// services/metrics.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::info;
use once_cell::sync::Lazy;
use crate::services::logging::LoggingService;

// Store for performance metrics
static METRICS_STORE: Lazy<Mutex<HashMap<String, MetricValues>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct MetricValues {
    pub count: u64,
    pub min: u64,
    pub max: u64,
    pub sum: u64,
    pub last_update: Instant,
}

impl MetricValues {
    fn new(value: u64) -> Self {
        Self {
            count: 1,
            min: value,
            max: value,
            sum: value,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self, value: u64) {
        self.count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.last_update = Instant::now();
    }

    pub fn average(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.sum / self.count
        }
    }
}

/// Service for tracking performance and error metrics
#[derive(Debug)]
pub struct MetricsService {
    logging_service: Arc<LoggingService>,
}

impl MetricsService {
    /// Create a new metrics service
    pub fn new(logging_service: Arc<LoggingService>) -> Self {
        Self { logging_service }
    }

    /// Record a metric value
    pub fn record(&self, name: &str, value: u64) {
        if let Ok(mut store) = METRICS_STORE.lock() {
            match store.get_mut(name) {
                Some(metric) => metric.update(value),
                None => {
                    store.insert(name.to_string(), MetricValues::new(value));
                }
            }
        }
    }

    /// Get a specific metric
    pub fn get(&self, name: &str) -> Option<MetricValues> {
        if let Ok(store) = METRICS_STORE.lock() {
            store.get(name).cloned()
        } else {
            None
        }
    }

    /// Get all metrics
    pub fn get_all(&self) -> HashMap<String, MetricValues> {
        if let Ok(store) = METRICS_STORE.lock() {
            store.clone()
        } else {
            HashMap::new()
        }
    }

    /// Start a background task to periodically log metrics
    pub fn start_metrics_logger(&self, period: Duration) {
        let logging_service = self.logging_service.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(period);
            
            loop {
                interval.tick().await;
                
                // Log all metrics
                logging_service.log_system_health();
                
                // Clean up old metrics (older than 1 hour)
                if let Ok(mut store) = METRICS_STORE.lock() {
                    store.retain(|_, values| {
                        values.last_update.elapsed() < Duration::from_secs(3600)
                    });
                }
            }
        });
    }
}

/// Helper function to measure and record operation duration
pub struct TimedMetric<'a> {
    name: String,
    start: Instant,
    metrics_service: &'a MetricsService,
}

impl<'a> TimedMetric<'a> {
    pub fn new(name: impl Into<String>, metrics_service: &'a MetricsService) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            metrics_service,
        }
    }
}

impl<'a> Drop for TimedMetric<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_millis() as u64;
        self.metrics_service.record(&self.name, duration);
    }
}

/// Enum to categorize performance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    Command,
    ApiCall,
    DatabaseQuery,
    CacheOperation,
    CustomOperation,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::ApiCall => "api_call",
            Self::DatabaseQuery => "db_query",
            Self::CacheOperation => "cache",
            Self::CustomOperation => "custom",
        }
    }
}

/// Format a metric name with proper categorization
pub fn format_metric_name(metric_type: MetricType, name: &str) -> String {
    format!("{}.{}", metric_type.as_str(), name)
}
