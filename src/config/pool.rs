use rig_mongodb::options::ClientOptions;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MongoPoolConfig {
    pub min_pool_size: u32,
    pub max_pool_size: u32,
    pub connect_timeout: Duration,
}

impl Default for MongoPoolConfig {
    fn default() -> Self {
        Self {
            min_pool_size: 5,
            max_pool_size: 10,
            connect_timeout: Duration::from_secs(20),
        }
    }
}

impl MongoPoolConfig {
    pub fn from_env() -> Self {
        Self {
            min_pool_size: std::env::var("MONGODB_MIN_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            max_pool_size: std::env::var("MONGODB_MAX_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            connect_timeout: Duration::from_millis(
                std::env::var("MONGODB_CONNECT_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20000)
            ),
        }
    }

    pub fn apply_to_options(&self, options: &mut ClientOptions) {
        options.min_pool_size = Some(self.min_pool_size);
        options.max_pool_size = Some(self.max_pool_size);
        options.connect_timeout = Some(self.connect_timeout);
    }
}