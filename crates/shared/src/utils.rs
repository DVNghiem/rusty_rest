use tracing::{info, instrument};
use uuid::Uuid;

/// Logging utilities
pub struct Logger;

impl Logger {
    /// Initialize structured logging
    pub fn init() {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("my_project=debug".parse().unwrap())
                    .add_directive("tower_http=debug".parse().unwrap())
                    .add_directive("axum=debug".parse().unwrap()),
            )
            .with_target(false)
            .compact()
            .init();
    }
}

/// Request ID utilities
pub struct RequestId;

impl RequestId {
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }
}

/// Retry utilities for external service calls
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub exponential_base: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(10),
            exponential_base: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> std::time::Duration {
        if attempt == 0 {
            return std::time::Duration::ZERO;
        }

        let delay = self.base_delay.as_millis() as f64
            * self.exponential_base.powi((attempt - 1) as i32);

        std::time::Duration::from_millis((delay as u64).min(self.max_delay.as_millis() as u64))
    }
}

/// Execute a function with retry logic
#[instrument(skip(operation, policy))]
pub async fn retry_async<F, Fut, T, E>(
    operation: F,
    policy: &RetryPolicy,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 1..=policy.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(attempt, "Operation succeeded after retry");
                }
                return Ok(result);
            }
            Err(error) => {
                last_error = Some(error);
                if attempt < policy.max_attempts {
                    let delay = policy.delay_for_attempt(attempt);
                    tracing::warn!(
                        attempt,
                        delay_ms = delay.as_millis(),
                        "Operation failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Environment utilities
pub fn get_env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

pub fn get_env_var_or_default(key: &str, default: &str) -> String {
    get_env_var(key).unwrap_or_else(|| default.to_string())
}

pub fn require_env_var(key: &str) -> anyhow::Result<String> {
    get_env_var(key).ok_or_else(|| anyhow::anyhow!("Required environment variable {} not set", key))
}

/// JSON serialization utilities
pub fn to_json_pretty<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

pub fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> anyhow::Result<T> {
    serde_json::from_str(json).map_err(Into::into)
}

/// Hash utilities for caching keys
pub fn hash_key(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Generate a cache key with namespace
pub fn cache_key(namespace: &str, key: &str) -> String {
    format!("{}:{}", namespace, key)
}

/// Metrics utilities for observability
pub struct Metrics;

impl Metrics {
    pub fn record_duration(operation: &str, duration: std::time::Duration) {
        tracing::info!(
            operation,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
    }

    pub fn record_counter(metric: &str, value: u64) {
        tracing::info!(metric, value, "Counter incremented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.delay_for_attempt(0), std::time::Duration::ZERO);
        assert_eq!(policy.delay_for_attempt(1), std::time::Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), std::time::Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), std::time::Duration::from_millis(400));
    }

    #[test]
    fn test_cache_key_generation() {
        let key = cache_key("products", "123");
        assert_eq!(key, "products:123");
    }

    #[test]
    fn test_hash_key() {
        let hash1 = hash_key("test");
        let hash2 = hash_key("test");
        let hash3 = hash_key("different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}