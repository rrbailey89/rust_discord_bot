// services/mod.rs
pub mod api;
pub mod database;
pub mod logging;
pub mod metrics;
pub mod migrations;

// Re-export services
pub use logging::{LoggingService, TimedOperation};
pub use metrics::{MetricsService, TimedMetric, MetricType, format_metric_name};
