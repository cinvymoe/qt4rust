use super::StorageError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff
    pub base_delay: Duration,
    /// Maximum delay cap
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }
}

/// Execute an operation with exponential backoff retry
pub async fn with_retry<F, Fut, T>(config: &RetryConfig, operation: F) -> Result<T, StorageError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, StorageError>>,
{
    let mut attempt = 0;
    let mut rng = StdRng::from_entropy();

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                let _ = e;

                if attempt >= config.max_retries {
                    tracing::error!("Max retries ({}) exceeded", config.max_retries);
                    return Err(StorageError::MaxRetriesExceeded);
                }

                // Exponential backoff with jitter
                let exponential_delay =
                    config.base_delay.as_secs_f64() * 2.0_f64.powi(attempt as i32);
                let capped_delay = exponential_delay.min(config.max_delay.as_secs_f64());
                // Add jitter: -25% to +25%
                let jitter_factor = 1.0 + rng.gen_range(-0.25..0.25);
                let delay = Duration::from_secs_f64(capped_delay * jitter_factor);

                tracing::warn!(
                    "Retry attempt {}/{} after {:?}",
                    attempt + 1,
                    config.max_retries,
                    delay
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
