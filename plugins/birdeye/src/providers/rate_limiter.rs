use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug)]
struct RateLimiterState {
    tokens: f64,
    last_update: Instant,
    rate: f64,
    burst: f64,
}

#[derive(Debug)]
pub struct RateLimiter {
    state: Mutex<RateLimiterState>,
}

impl RateLimiter {
    pub fn new(rate: f64, burst: f64) -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                tokens: burst,
                last_update: Instant::now(),
                rate,
                burst,
            }),
        }
    }

    pub async fn acquire(&self) -> Duration {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update).as_secs_f64();
        
        // Add new tokens based on elapsed time
        state.tokens = (state.tokens + elapsed * state.rate).min(state.burst);
        state.last_update = now;

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            Duration::from_secs(0)
        } else {
            let wait_time = (1.0 - state.tokens) / state.rate;
            state.tokens = 0.0;
            Duration::from_secs_f64(wait_time)
        }
    }
}
