// config/cache.rs
use std::env;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size: usize,
    
    // Default TTL (Time-To-Live) for different cache types
    pub default_ttl_secs: u64,
    pub api_ttl_secs: u64,
    pub weather_ttl_secs: u64,
    pub openai_ttl_secs: u64,
    pub guild_config_ttl_secs: u64,
    pub user_data_ttl_secs: u64,
    
    // Feature flags for different cache types
    pub api_cache_enabled: bool,
    pub db_cache_enabled: bool,
}

impl CacheConfig {
    pub fn load() -> Result<Self, crate::error::Error> {
        Ok(Self {
            enabled: env::var("CACHE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
                
            max_size: env::var("CACHE_MAX_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
                
            default_ttl_secs: env::var("CACHE_DEFAULT_TTL_SECS")
                .unwrap_or_else(|_| "300".to_string())  // 5 minutes
                .parse()
                .unwrap_or(300),
                
            api_ttl_secs: env::var("CACHE_API_TTL_SECS")
                .unwrap_or_else(|_| "600".to_string())  // 10 minutes
                .parse()
                .unwrap_or(600),
                
            weather_ttl_secs: env::var("CACHE_WEATHER_TTL_SECS")
                .unwrap_or_else(|_| "1800".to_string())  // 30 minutes
                .parse()
                .unwrap_or(1800),
                
            openai_ttl_secs: env::var("CACHE_OPENAI_TTL_SECS")
                .unwrap_or_else(|_| "86400".to_string())  // 24 hours
                .parse()
                .unwrap_or(86400),
                
            guild_config_ttl_secs: env::var("CACHE_GUILD_CONFIG_TTL_SECS")
                .unwrap_or_else(|_| "3600".to_string())  // 1 hour
                .parse()
                .unwrap_or(3600),
                
            user_data_ttl_secs: env::var("CACHE_USER_DATA_TTL_SECS")
                .unwrap_or_else(|_| "1800".to_string())  // 30 minutes
                .parse()
                .unwrap_or(1800),
                
            api_cache_enabled: env::var("CACHE_API_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
                
            db_cache_enabled: env::var("CACHE_DB_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
    
    pub fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.default_ttl_secs)
    }
    
    pub fn api_ttl(&self) -> Duration {
        Duration::from_secs(self.api_ttl_secs)
    }
    
    pub fn weather_ttl(&self) -> Duration {
        Duration::from_secs(self.weather_ttl_secs)
    }
    
    pub fn openai_ttl(&self) -> Duration {
        Duration::from_secs(self.openai_ttl_secs)
    }
    
    pub fn guild_config_ttl(&self) -> Duration {
        Duration::from_secs(self.guild_config_ttl_secs)
    }
    
    pub fn user_data_ttl(&self) -> Duration {
        Duration::from_secs(self.user_data_ttl_secs)
    }
}
