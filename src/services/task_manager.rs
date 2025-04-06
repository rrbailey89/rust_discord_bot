// services/task_manager.rs
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, info};
use crate::error::Error;

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority, can be delayed or cancelled if resources are constrained
    Low = 0,
    /// Normal priority, standard task execution
    Normal = 1,
    /// High priority, important for system functioning
    High = 2,
    /// Critical priority, should not be delayed or cancelled
    Critical = 3,
}

/// Status of a running task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is queued but not yet running
    Queued,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Metrics for task manager
#[derive(Debug, Clone, Default)]
pub struct TaskManagerMetrics {
    /// Number of tasks by priority
    pub tasks_by_priority: [usize; 4],
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Number of cancelled tasks
    pub cancelled_tasks: usize,
    /// Average task duration in milliseconds
    pub avg_duration_ms: f64,
}

type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Task information
struct TaskInfo {
    /// Task name
    name: String,
    /// Task priority
    priority: TaskPriority,
    /// When the task was created
    created_at: Instant,
    /// When the task started running
    started_at: Option<Instant>,
    /// When the task completed
    completed_at: Option<Instant>,
    /// Current status
    status: TaskStatus,
    /// Join handle for the task
    handle: Option<JoinHandle<()>>,
}

// Manually implement Debug for TaskInfo to handle the JoinHandle which might not be Debug
impl std::fmt::Debug for TaskInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskInfo")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("created_at", &self.created_at)
            .field("started_at", &self.started_at)
            .field("completed_at", &self.completed_at)
            .field("status", &self.status)
            .field("handle", &format!("{:?}", self.handle.is_some()))
            .finish()
    }
}

/// Task manager for efficient background task management
#[derive(Debug)]
pub struct TaskManager {
    /// Active tasks
    tasks: Arc<RwLock<HashMap<String, TaskInfo>>>,
    /// Total tasks stats
    metrics: Arc<Mutex<TaskManagerMetrics>>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(TaskManagerMetrics::default())),
        }
    }

    /// Spawn a new task with normal priority
    pub async fn spawn_task<F>(&self, name: &str, future: F) -> Result<(), Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn_task_with_priority(name, TaskPriority::Normal, future).await
    }

    /// Spawn a new task with the specified priority
    pub async fn spawn_task_with_priority<F>(&self, name: &str, priority: TaskPriority, future: F) -> Result<(), Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Check if task with this name already exists
        {
            let tasks = self.tasks.read().await;
            if tasks.contains_key(name) {
                return Err(Error::Unknown(format!("Task with name '{}' already exists", name)));
            }
        }

        // Create task info
        let task_info = TaskInfo {
            name: name.to_string(),
            priority,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Queued,
            handle: None,
        };

        // Add task to map
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(name.to_string(), task_info);
        }

        // Update metrics for queued task
        {
            let mut metrics = self.metrics.lock().await;
            metrics.tasks_by_priority[priority as usize] += 1;
        }

        // Clone references for the task closure
        let task_name = name.to_string();
        let tasks_ref = self.tasks.clone();
        let metrics_ref = self.metrics.clone();

        // Spawn the task
        let handle = tokio::spawn(async move {
            // Update task to running
            {
                let mut tasks = tasks_ref.write().await;
                if let Some(task) = tasks.get_mut(&task_name) {
                    task.status = TaskStatus::Running;
                    task.started_at = Some(Instant::now());
                }
            }

            debug!("Starting task: {}", task_name);

            // Run the actual future
            let result = match future.await {
                () => {
                    debug!("Task completed successfully: {}", task_name);
                    TaskStatus::Completed
                }
                // No error case in this specific future's Output type
                // If we wanted to handle errors, we could change the Future type
            };

            // Update task status
            let duration = {
                let mut tasks = tasks_ref.write().await;
                if let Some(task) = tasks.get_mut(&task_name) {
                    task.status = result;
                    task.completed_at = Some(Instant::now());
                    task.completed_at.unwrap().duration_since(task.started_at.unwrap())
                } else {
                    // Should never happen
                    debug!("Task not found after execution: {}", task_name);
                    Duration::from_secs(0)
                }
            };

            // Update metrics
            {
                let mut metrics = metrics_ref.lock().await;
                match result {
                    TaskStatus::Completed => {
                        metrics.completed_tasks += 1;
                    }
                    TaskStatus::Failed => {
                        metrics.failed_tasks += 1;
                    }
                    TaskStatus::Cancelled => {
                        metrics.cancelled_tasks += 1;
                    }
                    _ => {}
                }

                // Update average duration
                let duration_ms = duration.as_millis() as f64;
                let total_completed = metrics.completed_tasks as f64;
                if total_completed > 0.0 {
                    metrics.avg_duration_ms = (metrics.avg_duration_ms * (total_completed - 1.0) + duration_ms) / total_completed;
                }
            }
        });

        // Store the handle
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(name) {
                task.handle = Some(handle);
            }
        }

        info!("Spawned task: {} with priority {:?}", name, priority);
        Ok(())
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, name: &str) -> Result<(), Error> {
        let handle = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(name).ok_or_else(|| Error::NotFound(format!("Task not found: {}", name)))?;

            if task.status != TaskStatus::Running && task.status != TaskStatus::Queued {
                return Err(Error::Unknown(format!("Task is not running or queued: {}", name)));
            }

            task.status = TaskStatus::Cancelled;
            task.handle.take()
        };

        // Abort the task if we have a handle
        if let Some(handle) = handle {
            handle.abort();
            debug!("Cancelled task: {}", name);

            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.cancelled_tasks += 1;
        }

        Ok(())
    }

    /// Get the status of a task
    pub async fn get_task_status(&self, name: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(name).map(|task| task.status)
    }

    /// Get the current metrics
    pub async fn get_metrics(&self) -> TaskManagerMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// Clean up completed tasks older than the specified duration
    pub async fn cleanup_old_tasks(&self, older_than: Duration) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        // Find tasks to remove
        {
            let tasks = self.tasks.read().await;
            for (name, task) in tasks.iter() {
                if let Some(completed_at) = task.completed_at {
                    if now.duration_since(completed_at) > older_than {
                        to_remove.push(name.clone());
                    }
                }
            }
        }

        // Remove tasks
        if !to_remove.is_empty() {
            let mut tasks = self.tasks.write().await;
            for name in to_remove {
                tasks.remove(&name);
                debug!("Cleaned up old task: {}", name);
            }
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
