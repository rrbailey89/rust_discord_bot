// services/logging.rs
use crate::config::LoggingConfig;
use crate::error::Error;
use std::path::{Path, PathBuf};
use tracing::Level;
use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Metrics for performance tracking
static PERFORMANCE_METRICS: Lazy<Mutex<HashMap<String, Vec<u64>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// Counter for error occurrences
static ERROR_COUNTS: Lazy<Mutex<HashMap<String, u64>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Centralized logging service for the application
#[derive(Debug)]
pub struct LoggingService {
    config: LoggingConfig,
}

impl LoggingService {
    /// Create a new logging service with the given configuration
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Initialize the logging system
    pub fn init(&self) -> Result<(), Error> {
        // Parse the log level from config
        let level = match self.config.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };
        
        // Check if we need to log to both file and stdout
        let log_to_stdout = self.config.log_to_stdout;
        
        if let Some(file_path) = &self.config.file_path {
            let path = PathBuf::from(file_path);
            
            // Get directory and ensure it exists
            let log_dir = if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| Error::Io(e))?;
                }
                parent
            } else {
                Path::new(".")
            };
            
            // Get filename or use default
            let file_name = path.file_name()
                .map(|name| name.to_str().unwrap_or("bot.log"))
                .unwrap_or("bot.log");
            
            // Set up non-blocking file appender for writing to log files
            let file_appender = tracing_appender::rolling::daily(log_dir, file_name);
            let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);
            
            // Initialize subscriber based on whether we need to log to stdout too
            if log_to_stdout {
                // Create a subscriber that logs to both file and stdout
                tracing::info!("Initializing logging to both file and stdout");
                
                match tracing_subscriber::fmt()
                    .with_max_level(level)
                    .with_writer(non_blocking_file)
                    // This causes output to be duplicated to stdout as well
                    .with_writer(std::io::stdout)
                    .with_ansi(false)  // Disable ANSI colors in file
                    .with_target(true) // Include targets
                    .with_thread_ids(self.config.include_thread_ids)
                    .with_thread_names(self.config.include_thread_names)
                    .try_init() {
                        Ok(_) => {
                            // Store guard in static to keep it alive
                            std::mem::forget(file_guard);
                        },
                        Err(_) => {
                            return Err(Error::Unknown("Failed to initialize logging".into()));
                        }
                    }
            } else {
                // File logging only
                match tracing_subscriber::fmt()
                    .with_max_level(level)
                    .with_writer(non_blocking_file)
                    .with_ansi(false)  // Disable ANSI colors in file
                    .with_target(true) // Include targets
                    .with_thread_ids(self.config.include_thread_ids)
                    .with_thread_names(self.config.include_thread_names)
                    .try_init() {
                        Ok(_) => {
                            // Store guard in static to keep it alive
                            std::mem::forget(file_guard);
                        },
                        Err(_) => {
                            return Err(Error::Unknown("Failed to initialize logging".into()));
                        }
                    }
            }
        } else if log_to_stdout {
            // Initialize subscriber with stdout only
            match tracing_subscriber::fmt()
                .with_max_level(level)
                .with_target(true)
                .with_thread_ids(self.config.include_thread_ids)
                .with_thread_names(self.config.include_thread_names)
                .try_init() {
                    Ok(_) => {},
                    Err(_) => {
                        return Err(Error::Unknown("Failed to initialize logging".into()));
                    }
                }
        } else {
            // No logging destination specified
            return Err(Error::Unknown("No logging destination specified (neither file nor stdout)".into()));
        }

        tracing::info!(
            level = %self.config.level,
            file_path = ?self.config.file_path,
            rotation_days = self.config.rotation_days,
            rotation_size_mb = self.config.rotation_size_mb,
            "Logging initialized"
        );
        
        Ok(())
    }

    /// Log command execution with context
    pub fn log_command_execution(&self, command: &str, guild_id: Option<i64>, user_id: Option<i64>) {
        tracing::info!(
            command = command,
            guild_id = ?guild_id,
            user_id = ?user_id,
            "Command executed"
        );
    }

    /// Log external API call with duration
    pub fn log_api_call(&self, service: &str, endpoint: &str, duration_ms: u64) {
        tracing::debug!(
            service = service,
            endpoint = endpoint,
            duration_ms = duration_ms,
            "External API call"
        );
        
        // Store metrics for performance tracking
        let metric_key = format!("api_call.{}.{}", service, endpoint);
        self.record_metric(&metric_key, duration_ms);
    }
    
    /// Log database query with duration
    pub fn log_database_query(&self, query_type: &str, duration_ms: u64) {
        tracing::debug!(
            query_type = query_type,
            duration_ms = duration_ms,
            "Database query executed"
        );
        
        // Store metrics for performance tracking
        let metric_key = format!("db_query.{}", query_type);
        self.record_metric(&metric_key, duration_ms);
    }
    
    /// Record a performance metric
    fn record_metric(&self, key: &str, value: u64) {
        if let Ok(mut metrics) = PERFORMANCE_METRICS.lock() {
            let values = metrics.entry(key.to_string()).or_insert_with(Vec::new);
            values.push(value);
            
            // Keep only the last 100 values to avoid unbounded growth
            if values.len() > 100 {
                values.remove(0);
            }
        }
    }
    
    /// Log an error occurrence
    pub fn log_error_occurrence(&self, error_type: &str) {
        if let Ok(mut counts) = ERROR_COUNTS.lock() {
            let count = counts.entry(error_type.to_string()).or_insert(0);
            *count += 1;
        }
    }
    
    /// Get performance metrics summary
    pub fn get_performance_metrics(&self) -> HashMap<String, (u64, u64, u64)> {
        let mut result = HashMap::new();
        
        if let Ok(metrics) = PERFORMANCE_METRICS.lock() {
            for (key, values) in metrics.iter() {
                if !values.is_empty() {
                    // Calculate min, max, avg
                    let min = *values.iter().min().unwrap_or(&0);
                    let max = *values.iter().max().unwrap_or(&0);
                    let avg = values.iter().sum::<u64>() / values.len() as u64;
                    
                    result.insert(key.clone(), (min, avg, max));
                }
            }
        }
        
        result
    }
    
    /// Get error counts
    pub fn get_error_counts(&self) -> HashMap<String, u64> {
        if let Ok(counts) = ERROR_COUNTS.lock() {
            counts.clone()
        } else {
            HashMap::new()
        }
    }
    
    /// Log system health information
    pub fn log_system_health(&self) {
        // Log performance metrics
        let metrics = self.get_performance_metrics();
        for (key, (min, avg, max)) in metrics {
            tracing::info!(
                metric = %key,
                min_ms = min,
                avg_ms = avg, 
                max_ms = max,
                "Performance metric"
            );
        }
        
        // Log error counts
        let error_counts = self.get_error_counts();
        for (error_type, count) in error_counts {
            tracing::info!(
                error_type = %error_type,
                count = count,
                "Error count"
            );
        }
    }
}

/// Utility struct for measuring and logging execution time
pub struct TimedOperation {
    start: Instant,
    name: String,
    service: Option<String>,
    logging_service: Arc<LoggingService>,
    is_api_call: bool,
    is_db_query: bool,
}

impl TimedOperation {
    /// Create a new timed operation for generic measurement
    pub fn new(name: impl Into<String>, logging_service: Arc<LoggingService>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
            service: None,
            logging_service,
            is_api_call: false,
            is_db_query: false,
        }
    }
    
    /// Create a new timed operation for API call measurement
    pub fn for_api_call(
        service: impl Into<String>, 
        endpoint: impl Into<String>, 
        logging_service: Arc<LoggingService>
    ) -> Self {
        Self {
            start: Instant::now(),
            name: endpoint.into(),
            service: Some(service.into()),
            logging_service,
            is_api_call: true,
            is_db_query: false,
        }
    }
    
    /// Create a new timed operation for database query measurement
    pub fn for_db_query(
        query_type: impl Into<String>, 
        logging_service: Arc<LoggingService>
    ) -> Self {
        Self {
            start: Instant::now(),
            name: query_type.into(),
            service: None,
            logging_service,
            is_api_call: false,
            is_db_query: true,
        }
    }
}

impl Drop for TimedOperation {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        
        if self.is_api_call {
            if let Some(service) = &self.service {
                self.logging_service.log_api_call(service, &self.name, duration_ms);
            }
        } else if self.is_db_query {
            self.logging_service.log_database_query(&self.name, duration_ms);
        } else {
            // Generic operation
            tracing::debug!(
                operation = %self.name,
                duration_ms = duration_ms,
                "Operation completed"
            );
            
            // Store metrics
            let metric_key = format!("operation.{}", self.name);
            self.logging_service.record_metric(&metric_key, duration_ms);
        }
    }
}
