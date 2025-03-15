// config/logging.rs
use serde::Deserialize;
use std::env;
use std::collections::HashMap;
use crate::error::Error;

/// Format options for log output
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable text format
    Text,
    /// JSON format for machine processing
    Json,
    /// Compact format with less verbosity
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Text
    }
}

impl LogFormat {
    /// Parse from string
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "compact" => Self::Compact,
            _ => Self::Text,
        }
    }
}

/// Configuration for the logging system
#[derive(Clone, Deserialize, Debug)]
pub struct LoggingConfig {
    /// Global log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Module-specific log levels
    pub module_levels: HashMap<String, String>,
    
    /// Path to log file (if None, logs to stdout only)
    pub file_path: Option<String>,
    
    /// Log file rotation size in megabytes
    pub rotation_size_mb: u64,
    
    /// Number of days to keep log files
    pub rotation_days: u32,
    
    /// Log format (text, json, compact)
    pub format: LogFormat,
    
    /// Whether to log to stdout in addition to file
    pub log_to_stdout: bool,
    
    /// Whether to include thread IDs in logs
    pub include_thread_ids: bool,
    
    /// Whether to include thread names in logs
    pub include_thread_names: bool,
    
    /// Whether to include file and line information
    pub include_file_line: bool,
}

impl LoggingConfig {
    /// Load logging configuration from environment variables
    pub fn load() -> Result<Self, Error> {
        // Initialize module_levels map
        let mut module_levels = HashMap::new();
        
        // Look for LOG_LEVEL_* environment variables
        for (key, value) in env::vars() {
            if key.starts_with("LOG_LEVEL_") {
                let module = key.trim_start_matches("LOG_LEVEL_").to_lowercase();
                module_levels.insert(module, value);
            }
        }
        
        // Parse log format
        let format_str = env::var("LOG_FORMAT").unwrap_or_else(|_| String::from("text"));
        let format = LogFormat::from_str(&format_str);
        
        Ok(Self {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| String::from("info")),
            module_levels,
            file_path: env::var("LOG_FILE_PATH").ok(),
            rotation_size_mb: env::var("LOG_ROTATION_SIZE_MB")
                .unwrap_or_else(|_| String::from("10"))
                .parse()
                .unwrap_or(10),
            rotation_days: env::var("LOG_ROTATION_DAYS")
                .unwrap_or_else(|_| String::from("7"))
                .parse()
                .unwrap_or(7),
            format,
            log_to_stdout: env::var("LOG_TO_STDOUT")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            include_thread_ids: env::var("LOG_INCLUDE_THREAD_IDS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            include_thread_names: env::var("LOG_INCLUDE_THREAD_NAMES")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            include_file_line: env::var("LOG_INCLUDE_FILE_LINE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }
    
    /// Build a filter string from the configuration
    pub fn build_filter_string(&self) -> String {
        let mut parts = Vec::new();
        
        // Add global level
        parts.push(format!("info,{}={}", env!("CARGO_PKG_NAME"), self.level));
        
        // Add module-specific levels
        for (module, level) in &self.module_levels {
            parts.push(format!("{}={}", module, level));
        }
        
        parts.join(",")
    }
}
