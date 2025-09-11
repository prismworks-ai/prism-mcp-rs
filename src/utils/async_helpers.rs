//! Async utility functions and helpers

use super::{UtilError, UtilResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Execute a future with a timeout
pub async fn with_timeout<F, T>(future: F, duration: Duration) -> UtilResult<T>
where
    F: Future<Output = T>,
{
    timeout(duration, future)
        .await
        .map_err(|_| UtilError::Timeout {
            duration_ms: duration.as_millis() as u64,
        })
}

/// Retry a future with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts - 1 => return Err(e),
            Err(_) => {
                sleep(delay).await;
                delay = std::cmp::min(delay * 2, max_delay);
            }
        }
    }

    unreachable!()
}

/// Execute multiple futures concurrently with a limit
pub async fn execute_with_concurrency_limit<F, Fut, T>(operations: Vec<F>, limit: usize) -> Vec<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
    T: Send + 'static,
{
    use futures::stream::{FuturesUnordered, StreamExt};
    use std::collections::VecDeque;

    let mut queue: VecDeque<_> = operations.into_iter().collect();
    let mut active = FuturesUnordered::new();
    let mut results = Vec::new();

    // Start initial batch
    for _ in 0..std::cmp::min(limit, queue.len()) {
        if let Some(op) = queue.pop_front() {
            active.push(op());
        }
    }

    // Process results and start new operations
    while let Some(result) = active.next().await {
        results.push(result);

        // Start next operation if available
        if let Some(op) = queue.pop_front() {
            active.push(op());
        }
    }

    results
}

/// Create a future that completes after a delay
pub async fn delay(duration: Duration) {
    sleep(duration).await;
}

/// Create a cancellable future
pub struct CancellableTask<T> {
    handle: tokio::task::JoinHandle<T>,
}

impl<T> CancellableTask<T> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self {
            handle: tokio::spawn(future),
        }
    }

    pub fn cancel(&self) {
        self.handle.abort();
    }

    pub async fn wait(self) -> Result<T, tokio::task::JoinError> {
        self.handle.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(async { "success" }, Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(result, "success");
    }

    #[tokio::test]
    async fn test_with_timeout_failure() {
        let result = with_timeout(
            async {
                sleep(Duration::from_secs(2)).await;
                "too slow"
            },
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let mut attempts = 0;

        let result = retry_with_backoff(
            || {
                attempts += 1;
                async move {
                    if attempts < 3 {
                        Err("not ready")
                    } else {
                        Ok("success")
                    }
                }
            },
            5,
            Duration::from_millis(10),
            Duration::from_millis(100),
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }
}
