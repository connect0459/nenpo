use anyhow::{Context, Result};
use std::thread;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl RetryConfig {
    /// Creates a default retry configuration
    /// - Max retries: 3
    /// - Initial delay: 1000ms (1 second)
    /// - Backoff multiplier: 2.0 (exponential)
    pub fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }

    /// Creates a retry configuration with custom values
    #[allow(dead_code)]
    pub fn new(max_retries: u32, initial_delay_ms: u64, backoff_multiplier: f64) -> Self {
        Self {
            max_retries,
            initial_delay_ms,
            backoff_multiplier,
        }
    }
}

/// Executes an operation with retry logic
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - Operation to execute
///
/// # Returns
///
/// Result of the operation
pub fn with_retry<F, T>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                if attempt > config.max_retries {
                    return Err(e).context(format!(
                        "Operation failed after {} retries",
                        config.max_retries
                    ));
                }

                // Check if error is retryable (API rate limit)
                let error_msg = format!("{:?}", e);
                if error_msg.contains("API rate limit") || error_msg.contains("403") {
                    eprintln!(
                        "Rate limit error detected. Retrying in {}ms (attempt {}/{})",
                        delay, attempt, config.max_retries
                    );
                    thread::sleep(Duration::from_millis(delay));
                    delay = (delay as f64 * config.backoff_multiplier) as u64;
                } else {
                    // Non-retryable error
                    return Err(e).context("Non-retryable error occurred");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn completes_successful_operation_without_retry() {
        let config = RetryConfig::default();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = with_retry(&config, || {
            *call_count_clone.lock().unwrap() += 1;
            Ok::<i32, anyhow::Error>(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn fails_when_max_retry_count_reached() {
        let config = RetryConfig::new(2, 10, 1.0); // 短い遅延でテスト
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<()> = with_retry(&config, || {
            *call_count_clone.lock().unwrap() += 1;
            anyhow::bail!("API rate limit exceeded (403)")
        });

        assert!(result.is_err());
        assert_eq!(*call_count.lock().unwrap(), 3); // Initial + 2 retries
    }

    #[test]
    fn retries_on_retryable_error() {
        let config = RetryConfig::new(3, 10, 1.0);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = with_retry(&config, || {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            if *count < 3 {
                anyhow::bail!("API rate limit exceeded (403)")
            } else {
                Ok::<i32, anyhow::Error>(100)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[test]
    fn fails_immediately_on_non_retryable_error() {
        let config = RetryConfig::default();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<()> = with_retry(&config, || {
            *call_count_clone.lock().unwrap() += 1;
            anyhow::bail!("Invalid credentials")
        });

        assert!(result.is_err());
        assert_eq!(*call_count.lock().unwrap(), 1); // No retries
    }
}
