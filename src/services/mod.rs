// services/mod.rs
pub mod api;
pub mod database;
pub mod logging;
pub mod metrics;
pub mod migrations;
pub mod cache;
pub mod task_manager;
pub mod rate_limiter;
pub mod command_registry;
pub mod command_cooldown;

// Re-export services
pub use logging::{LoggingService, TimedOperation};
pub use metrics::{MetricsService, MetricType, format_metric_name};
pub use cache::{CacheService, CacheStats, CacheResult};
pub use task_manager::{TaskManager, TaskPriority, TaskStatus, TaskManagerMetrics};
pub use rate_limiter::{RateLimiter, RateLimit, RateLimiterMetric};
pub use command_registry::CommandRegistryService;
pub use command_cooldown::CommandCooldownService;
