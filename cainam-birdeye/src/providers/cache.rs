use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

#[derive(Debug)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

#[derive(Debug)]
struct CacheState<T> {
    inner: Arc<T>,
    cache: HashMap<String, CacheEntry<Vec<u8>>>,
}

#[derive(Debug, Clone)]
pub struct CachedClient<T> {
    state: Arc<RwLock<CacheState<T>>>,
}

impl<T: Send + Sync + 'static> CachedClient<T> {
    pub fn new(inner: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(CacheState {
                inner: Arc::new(inner),
                cache: HashMap::new(),
            })),
        }
    }

    pub async fn execute<F, Fut, R>(&self, key: &str, f: F, ttl: u64) -> R
    where
        F: FnOnce(Arc<T>) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send,
        R: DeserializeOwned + Serialize + Send + 'static,
    {
        // First try to get from cache with a read lock
        {
            let state = self.state.read().await;
            if let Some(entry) = state.cache.get(key) {
                if !entry.is_expired() {
                    if let Ok(data) = serde_json::from_slice(&entry.data) {
                        return data;
                    }
                }
            }
        }

        // Get the inner reference
        let inner = Arc::clone(&self.state.read().await.inner);
        
        // Execute the function
        let result = f(inner).await;

        // Cache the result with a write lock
        if let Ok(data) = serde_json::to_vec(&result) {
            let mut state = self.state.write().await;
            state.cache.insert(
                key.to_string(),
                CacheEntry::new(data, Duration::from_secs(ttl)),
            );
        }

        result
    }
}
