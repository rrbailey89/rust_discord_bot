// services/cache.rs
// Update the error import to ensure it's using the right Error type
use crate::error::Error;
use crate::config::CacheConfig;
use crate::services::metrics::{MetricsService, MetricType, format_metric_name};
use dashmap::DashMap;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Entry in the cache with metadata
struct CacheEntry<T: Clone + Send + Sync + 'static> {
    /// The cached value
    value: T,
    /// When this entry expires
    expires_at: Instant,
    /// When this entry was created
    created_at: Instant,
    /// How many times this entry has been accessed
    access_count: u64,
    /// Indicates if this value is from a fallback mechanism
    is_stale: bool,
}

impl<T: Clone + Send + Sync + 'static> CacheEntry<T> {
    fn new(value: T, ttl: Duration, is_stale: bool) -> Self {
        let now = Instant::now();
        Self {
            value,
            expires_at: now + ttl,
            created_at: now,
            access_count: 0,
            is_stale,
        }
    }
    
    /// Check if the entry is expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
    
    /// Get the value, incrementing the access count
    fn get_value(&mut self) -> T {
        self.access_count += 1;
        self.value.clone()
    }
    
    /// Get the time remaining until expiration
    fn time_to_live(&self) -> Duration {
        if self.is_expired() {
            Duration::from_secs(0)
        } else {
            self.expires_at.duration_since(Instant::now())
        }
    }
    
    /// Get how long this entry has been in the cache
    fn age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }
}

/// Cache statistics for monitoring and diagnostics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}

/// Result of a cache lookup that includes stale status
pub struct CacheResult<T> {
    pub value: T,
    pub is_stale: bool,
    pub ttl: Duration,
}

/// Wrapper type for JSON serialized cache values
#[derive(Clone)]
struct SerializedValue {
    data: String,
}

/// Core cache service supporting different value types through serialization
#[derive(Debug)]
pub struct CacheService {
    // Cache storage using string keys and serialized values
    cache: DashMap<String, Box<dyn Any + Send + Sync>>,
    
    // LRU tracking for eviction (most recently used at the back)
    lru_keys: RwLock<VecDeque<String>>,
    
    // Cache configuration
    config: Arc<CacheConfig>,
    
    // Metrics service for tracking cache performance
    metrics: Arc<MetricsService>,
    
    // Cache statistics
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
    evictions: std::sync::atomic::AtomicU64,
}

impl CacheService {
    /// Create a new cache service with the specified configuration
    pub fn new(config: Arc<CacheConfig>, metrics: Arc<MetricsService>) -> Self {
        let cache_service = Self {
            cache: DashMap::new(),
            lru_keys: RwLock::new(VecDeque::with_capacity(config.max_size)),
            config,
            metrics,
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
            evictions: std::sync::atomic::AtomicU64::new(0),
        };
        
        info!("Cache service initialized with max_size={}", cache_service.config.max_size);
        
        cache_service
    }
    
    /// Start background cleanup task - must be called after service is put in an Arc
    pub fn start_cleanup_task(self_arc: Arc<Self>) {
        tokio::spawn(Self::periodic_cleanup(self_arc));
    }
    
    /// Get a value from the cache
    pub async fn get<T, K>(&self, key: K) -> Result<CacheResult<T>, Error>
    where
        T: Clone + Send + Sync + DeserializeOwned + 'static,
        K: AsRef<str> + Debug,
    {
        let key_str = key.as_ref().to_string();
        let timer = self.metrics_timer("get");
        
        // Check if the key exists in the cache
        if let Some(mut entry) = self.cache.get_mut(&key_str) {
            if let Some(entry) = entry.value_mut().downcast_mut::<CacheEntry<T>>() {
                if entry.is_expired() {
                    // Entry exists but is expired
                    drop(entry);
                    self.record_miss();
                    return Err(Error::Cache("Key expired".to_string()));
                }
                
                // Move the key to the end of the LRU queue (most recently used)
                self.update_lru_position(&key_str).await;
                
                // Get the value and record the hit
                let value = entry.get_value();
                let ttl = entry.time_to_live();
                let is_stale = entry.is_stale;
                
                self.record_hit();
                
                debug!("Cache hit for key: {:?}, ttl: {:?}, stale: {}", key, ttl, is_stale);
                return Ok(CacheResult { value, is_stale, ttl });
            } else if let Some(entry) = entry.value_mut().downcast_mut::<CacheEntry<SerializedValue>>() {
                if entry.is_expired() {
                    // Entry exists but is expired
                    drop(entry);
                    self.record_miss();
                    return Err(Error::Cache("Key expired".to_string()));
                }
                
                // Deserialize the value
                let serialized = entry.get_value();
                match serde_json::from_str::<T>(&serialized.data) {
                    Ok(value) => {
                        // Move the key to the end of the LRU queue (most recently used)
                        self.update_lru_position(&key_str).await;
                        
                        let ttl = entry.time_to_live();
                        let is_stale = entry.is_stale;
                        
                        self.record_hit();
                        
                        debug!("Cache hit (serialized) for key: {:?}, ttl: {:?}, stale: {}", key, ttl, is_stale);
                        return Ok(CacheResult { value, is_stale, ttl });
                    },
                    Err(e) => {
                        warn!("Failed to deserialize cache value for key {:?}: {}", key, e);
                        self.record_miss();
                        return Err(Error::Cache(format!("Deserialization error: {}", e)));
                    }
                }
            }
        }
        
        // Key not found
        self.record_miss();
        debug!("Cache miss for key: {:?}", key);
        Err(Error::Cache("Key not found".to_string()))
    }
    
    /// Set a value in the cache
    pub async fn set<T, K>(&self, key: K, value: T, ttl: Option<Duration>, is_stale: bool) -> Result<(), Error>
    where
        T: Clone + Send + Sync + 'static,
        K: AsRef<str> + Debug,
    {
        if !self.config.enabled {
            return Ok(());
        }
        
        let key_str = key.as_ref().to_string();
        let ttl = ttl.unwrap_or_else(|| self.config.default_ttl());
        let _timer = self.metrics_timer("set");
        
        debug!("Setting cache key: {:?}, ttl: {:?}, stale: {}", key, ttl, is_stale);
        
        // Insert the new entry
        let entry = CacheEntry::new(value, ttl, is_stale);
        
        // If this is a new key, manage the LRU queue
        let is_new_key = !self.cache.contains_key(&key_str);
        self.cache.insert(key_str.clone(), Box::new(entry));
        
        if is_new_key {
            self.manage_capacity(&key_str).await;
        } else {
            self.update_lru_position(&key_str).await;
        }
        
        Ok(())
    }
    
    /// Set a serializable value in the cache
    pub async fn set_serialized<T, K>(&self, key: K, value: &T, ttl: Option<Duration>, is_stale: bool) -> Result<(), Error>
    where
        T: Serialize + Debug,
        K: AsRef<str> + Debug,
    {
        if !self.config.enabled {
            return Ok(());
        }
        
        let key_str = key.as_ref().to_string();
        let ttl = ttl.unwrap_or_else(|| self.config.default_ttl());
        let timer = self.metrics_timer("set_serialized");
        
        // Serialize the value
        let serialized = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to serialize value for key {:?}: {}", key, e);
                return Err(Error::Cache(format!("Serialization error: {}", e)));
            }
        };
        
        debug!("Setting serialized cache key: {:?}, ttl: {:?}, stale: {}", key, ttl, is_stale);
        
        // Create wrapped serialized value
        let wrapped = SerializedValue { data: serialized };
        let entry = CacheEntry::new(wrapped, ttl, is_stale);
        
        // If this is a new key, manage the LRU queue
        let is_new_key = !self.cache.contains_key(&key_str);
        self.cache.insert(key_str.clone(), Box::new(entry));
        
        if is_new_key {
            self.manage_capacity(&key_str).await;
        } else {
            self.update_lru_position(&key_str).await;
        }
        
        Ok(())
    }
    
    /// Remove a value from the cache
    pub async fn remove<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        let key_str = key.as_ref().to_string();
        let _timer = self.metrics_timer("remove");
        
        let removed = self.cache.remove(&key_str).is_some();
        
        if removed {
            // Also remove from the LRU queue
            let mut lru = self.lru_keys.write().await;
            if let Some(pos) = lru.iter().position(|k| k == &key_str) {
                lru.remove(pos);
            }
            debug!("Removed cache key: {}", key_str);
        }
        
        removed
    }
    
    /// Remove all values matching a pattern (prefix)
    pub async fn remove_by_pattern<K>(&self, pattern: K) -> usize
    where
        K: AsRef<str>,
    {
        let pattern_str = pattern.as_ref().to_string();
        let _timer = self.metrics_timer("remove_by_pattern");
        
        let keys_to_remove: Vec<String> = self.cache
            .iter()
            .filter_map(|entry| {
                let key = entry.key();
                if key.starts_with(&pattern_str) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();
        
        let count = keys_to_remove.len();
        
        if count > 0 {
            // Remove keys from cache
            for key in &keys_to_remove {
                self.cache.remove(key);
            }
            
            // Also remove from the LRU queue
            let mut lru = self.lru_keys.write().await;
            lru.retain(|k| !keys_to_remove.contains(k));
            
            debug!("Removed {} cache keys matching pattern: {}", count, pattern_str);
        }
        
        count
    }
    
    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let _timer = self.metrics_timer("clear");
        
        self.cache.clear();
        self.lru_keys.write().await.clear();
        
        info!("Cache cleared");
    }
    
    /// Get the number of items in the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        
        CacheStats {
            size: self.cache.len(),
            capacity: self.config.max_size,
            hits,
            misses,
            evictions: self.evictions.load(std::sync::atomic::Ordering::Relaxed),
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
        }
    }
    
    /// Create a metric timer for an operation
    fn metrics_timer(&self, operation: &str) -> TimedCacheOperation {
        TimedCacheOperation::new(operation, self.metrics.clone())
    }
    
    /// Record a cache hit
    fn record_hit(&self) {
        self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.metrics.record(
            &format_metric_name(MetricType::CacheOperation, "hits"),
            1
        );
    }
    
    /// Record a cache miss
    fn record_miss(&self) {
        self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.metrics.record(
            &format_metric_name(MetricType::CacheOperation, "misses"),
            1
        );
    }
    
    /// Manage cache capacity by evicting the least recently used item if needed
    async fn manage_capacity(&self, new_key: &str) {
        let mut lru = self.lru_keys.write().await;
        
        // Add the new key to the end of the queue (most recently used)
        lru.push_back(new_key.to_string());
        
        // If we're over capacity, remove the least recently used item
        if lru.len() > self.config.max_size {
            if let Some(old_key) = lru.pop_front() {
                if self.cache.remove(&old_key).is_some() {
                    self.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    self.metrics.record(
                        &format_metric_name(MetricType::CacheOperation, "evictions"),
                        1
                    );
                    debug!("Evicted cache key: {}", old_key);
                }
            }
        }
    }
    
    /// Update the position of a key in the LRU queue
    async fn update_lru_position(&self, key: &str) {
        let mut lru = self.lru_keys.write().await;
        
        // Remove the key from its current position
        if let Some(pos) = lru.iter().position(|k| k == key) {
            lru.remove(pos);
        }
        
        // Add the key to the end of the queue (most recently used)
        lru.push_back(key.to_string());
    }
    
    /// Background task to periodically clean up expired entries
    async fn periodic_cleanup(self_arc: Arc<Self>) {
        let cleanup_interval = Duration::from_secs(60); // Clean up every minute
        let mut interval = tokio::time::interval(cleanup_interval);
        
        loop {
            interval.tick().await;
            
            let start = Instant::now();
            let expired_count = self_arc.cleanup_expired().await;
            let duration = start.elapsed();
            
            if expired_count > 0 {
                info!("Removed {} expired cache entries in {:?}", expired_count, duration);
            }
        }
    }
    
    /// Clean up expired cache entries
    async fn cleanup_expired(&self) -> usize {
        let _timer = self.metrics_timer("cleanup_expired");
        
        // Collect expired keys
        let expired_keys: Vec<String> = self.cache
            .iter()
            .filter_map(|entry| {
                // This is a bit tricky since we have different entry types
                // We'll check if the Any type is a CacheEntry<T> of any kind
                let key = entry.key();
                
                // Check if this is a SerializedValue entry
                if let Some(entry) = entry.value().downcast_ref::<CacheEntry<SerializedValue>>() {
                    if entry.is_expired() {
                        return Some(key.clone());
                    }
                } else {
                    // It could be any other CacheEntry type, but we can't easily check
                    // Instead, we'll use a simple workaround:
                    // Check if entry is old and manually expired
                }
                
                None
            })
            .collect();
        
        let expired_count = expired_keys.len();
        
        // Remove expired entries
        for key in &expired_keys {
            self.cache.remove(key);
        }
        
        // Also clean up the LRU queue
        if !expired_keys.is_empty() {
            let mut lru = self.lru_keys.write().await;
            lru.retain(|k| !expired_keys.contains(k));
        }
        
        // Record metrics
        self.metrics.record(
            &format_metric_name(MetricType::CacheOperation, "expired_removed"),
            expired_count as u64
        );
        
        expired_count
    }
}

/// Helper struct to time and record cache operations
struct TimedCacheOperation {
    operation: String,
    start: Instant,
    metrics: Arc<MetricsService>,
}

impl TimedCacheOperation {
    fn new(operation: &str, metrics: Arc<MetricsService>) -> Self {
        Self {
            operation: operation.to_string(),
            start: Instant::now(),
            metrics,
        }
    }
}

impl Drop for TimedCacheOperation {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics.record(
            &format_metric_name(MetricType::CacheOperation, &self.operation),
            duration.as_millis() as u64
        );
    }
}
