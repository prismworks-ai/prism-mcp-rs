// ! smart retry logic for MCP operations
// !
// ! Module provides production-ready retry capabilities with:
// ! - Smart retry decisions based on error recoverability
// ! - Exponential backoff with jitter
// ! - Circuit breaker pattern for cascading failure protection
// ! - complete logging and metrics integration

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, warn};

use crate::core::error::{McpError, McpResult};
use crate::core::logging::{ErrorContext, ErrorLogger};
use crate::core::metrics::global_metrics;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to add random jitter to delays
    pub enable_jitter: bool,
    /// Maximum jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
    /// Whether to respect error recoverability
    pub respect_recoverability: bool,
    /// Custom timeout for individual attempts
    pub attempt_timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
            jitter_factor: 0.1,
            respect_recoverability: true,
            attempt_timeout: None,
        }
    }
}

impl RetryConfig {
    /// Create a conservative retry config for production
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 1.5,
            enable_jitter: true,
            jitter_factor: 0.05,
            respect_recoverability: true,
            attempt_timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create an aggressive retry config for high-availability scenarios
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 60000,
            backoff_multiplier: 2.5,
            enable_jitter: true,
            jitter_factor: 0.15,
            respect_recoverability: true,
            attempt_timeout: Some(Duration::from_secs(60)),
        }
    }

    /// Create a retry config for network operations
    pub fn network() -> Self {
        Self {
            max_attempts: 4,
            initial_delay_ms: 200,
            max_delay_ms: 15000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
            jitter_factor: 0.1,
            respect_recoverability: true,
            attempt_timeout: Some(Duration::from_secs(45)),
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through normally
    Closed,
    /// Circuit is open, requests fail immediately
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures to trigger circuit opening
    pub failure_threshold: u32,
    /// Time to wait before attempting recovery
    pub recovery_timeout: Duration,
    /// Number of successful requests needed to close circuit
    pub success_threshold: u32,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker for protecting against cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
    half_open_requests: AtomicU32,
    state: Arc<tokio::sync::RwLock<CircuitState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            half_open_requests: AtomicU32::new(0),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
        }
    }

    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Execute an operation through the circuit breaker
    pub async fn call<F, T>(&self, operation: F, context: &ErrorContext) -> McpResult<T>
    where
        F: std::future::Future<Output = McpResult<T>>,
    {
        // Check if circuit is open and if recovery timeout has passed
        let current_state = self.update_state_if_needed().await;

        match current_state {
            CircuitState::Open => {
                let error = McpError::connection("Circuit breaker is open");
                error.log_with_context(context.clone()).await;
                Err(error)
            }
            CircuitState::HalfOpen => {
                // Limit concurrent requests in half-open state
                let current_requests = self.half_open_requests.fetch_add(1, Ordering::SeqCst);
                if current_requests >= self.config.half_open_max_requests {
                    self.half_open_requests.fetch_sub(1, Ordering::SeqCst);
                    let error = McpError::connection(
                        "Circuit breaker is half-open with max concurrent requests",
                    );
                    error.log_with_context(context.clone()).await;
                    return Err(error);
                }

                let result = operation.await;
                self.half_open_requests.fetch_sub(1, Ordering::SeqCst);

                match &result {
                    Ok(_) => self.on_success().await,
                    Err(error) => {
                        if error.is_recoverable() {
                            self.on_failure().await;
                        }
                    }
                }

                result
            }
            CircuitState::Closed => {
                let result = operation.await;

                match &result {
                    Ok(_) => {
                        // Reset failure count on success
                        self.failure_count.store(0, Ordering::SeqCst);
                    }
                    Err(error) => {
                        if error.is_recoverable() {
                            self.on_failure().await;
                        }
                    }
                }

                result
            }
        }
    }

    /// Update circuit state based on time and failure count
    async fn update_state_if_needed(&self) -> CircuitState {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::Open => {
                let last_failure = self.last_failure_time.load(Ordering::SeqCst);
                let now = current_time_millis();

                if now.saturating_sub(last_failure)
                    >= self.config.recovery_timeout.as_millis() as u64
                {
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::SeqCst);
                    debug!("Circuit breaker transitioned to HalfOpen state");
                    CircuitState::HalfOpen
                } else {
                    CircuitState::Open
                }
            }
            _ => current_state,
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let current_state = *self.state.read().await;

        if current_state == CircuitState::HalfOpen {
            let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;

            if success_count >= self.config.success_threshold {
                let mut state = self.state.write().await;
                *state = CircuitState::Closed;
                self.failure_count.store(0, Ordering::SeqCst);
                self.success_count.store(0, Ordering::SeqCst);
                debug!(
                    "Circuit breaker transitioned to Closed state after {} successes",
                    success_count
                );
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure_time
            .store(current_time_millis(), Ordering::SeqCst);

        if failure_count >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            if *state == CircuitState::Closed {
                *state = CircuitState::Open;
                warn!(
                    "Circuit breaker opened after {} failures, recovery timeout: {:?}",
                    failure_count, self.config.recovery_timeout
                );
            } else if *state == CircuitState::HalfOpen {
                *state = CircuitState::Open;
                warn!("Circuit breaker reopened during half-open state");
            }
        }
    }

    /// Get circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state().await,
            failure_count: self.failure_count.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
            last_failure_time: self.last_failure_time.load(Ordering::SeqCst),
            half_open_requests: self.half_open_requests.load(Ordering::SeqCst),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: u64,
    pub half_open_requests: u32,
}

/// Retry policy with smart error-based decisions
#[derive(Debug)]
pub struct RetryPolicy {
    config: RetryConfig,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            circuit_breaker: None,
        }
    }

    /// Create a retry policy with circuit breaker
    pub fn with_circuit_breaker(
        config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Self {
        Self {
            config,
            circuit_breaker: Some(Arc::new(CircuitBreaker::new(circuit_breaker_config))),
        }
    }

    /// Execute an operation with smart retry logic
    pub async fn execute<F, T>(&self, mut operation: F, context: ErrorContext) -> McpResult<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = McpResult<T>> + Send>>,
    {
        let mut last_error = None;
        let start_time = Instant::now();

        for attempt in 1..=self.config.max_attempts {
            let attempt_start = Instant::now();

            // Execute through circuit breaker if available
            let result = if let Some(ref circuit_breaker) = self.circuit_breaker {
                circuit_breaker.call(operation(), &context).await
            } else {
                operation().await
            };

            match result {
                Ok(value) => {
                    // Success! Log if we had previous attempts
                    if attempt > 1 {
                        ErrorLogger::log_retry_success(
                            &context.operation,
                            attempt,
                            context.clone(),
                        )
                        .await;
                    }

                    // Record successful operation metrics
                    let metrics = global_metrics();
                    if let Some(ref method) = context.method {
                        metrics
                            .record_request(
                                method,
                                context.transport.as_deref().unwrap_or("unknown"),
                            )
                            .await;
                    }

                    return Ok(value);
                }
                Err(error) => {
                    let attempt_duration = attempt_start.elapsed();
                    last_error = Some(error.clone());

                    // Determine if we should retry
                    let should_retry = self.should_retry(&error, attempt).await;

                    // Log the retry attempt
                    ErrorLogger::log_retry_attempt(
                        &error,
                        attempt,
                        self.config.max_attempts,
                        should_retry,
                        context.clone(),
                    )
                    .await;

                    if !should_retry {
                        // Final failure, log and return error
                        error.log_with_context(context.clone()).await;
                        return Err(error);
                    }

                    // Calculate and apply retry delay
                    if attempt < self.config.max_attempts {
                        let delay = self.calculate_delay(attempt, attempt_duration);
                        debug!(
                            "Retrying {} in {:?} (attempt {}/{})",
                            context.operation, delay, attempt, self.config.max_attempts
                        );
                        sleep(delay).await;
                    }
                }
            }
        }

        // All retries exhausted
        let total_duration = start_time.elapsed();
        let final_error = last_error
            .unwrap_or_else(|| McpError::internal("Retry logic failed without capturing error"));

        error!(
            "Operation '{}' failed after {} attempts in {:?}",
            context.operation, self.config.max_attempts, total_duration
        );

        final_error.log_with_context(context).await;
        Err(final_error)
    }

    /// Determine if an error should trigger a retry
    async fn should_retry(&self, error: &McpError, attempt: u32) -> bool {
        // Don't retry if we've reached max attempts
        if attempt >= self.config.max_attempts {
            return false;
        }

        // Respect error recoverability if configured
        if self.config.respect_recoverability && !error.is_recoverable() {
            debug!(
                "Not retrying non-recoverable error: {} (category: {})",
                error,
                error.category()
            );
            return false;
        }

        true
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32, _last_attempt_duration: Duration) -> Duration {
        let base_delay = self.config.initial_delay_ms as f64
            * self.config.backoff_multiplier.powi(attempt as i32 - 1);

        let capped_delay = base_delay.min(self.config.max_delay_ms as f64);

        let final_delay = if self.config.enable_jitter {
            #[cfg(feature = "fastrand")]
            {
                let jitter_range = capped_delay * self.config.jitter_factor;
                let jitter = (fastrand::f64() - 0.5) * 2.0 * jitter_range;
                (capped_delay + jitter).max(0.0)
            }
            #[cfg(not(feature = "fastrand"))]
            {
                // No jitter without fastrand
                capped_delay
            }
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }

    /// Get circuit breaker statistics if available
    pub async fn circuit_breaker_stats(&self) -> Option<CircuitBreakerStats> {
        if let Some(ref circuit_breaker) = self.circuit_breaker {
            Some(circuit_breaker.stats().await)
        } else {
            None
        }
    }
}

/// Get current time in milliseconds since epoch
fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_retry_policy_success_immediate() {
        let policy = RetryPolicy::new(RetryConfig::default());
        let context = ErrorContext::new("test_operation");

        let result = policy
            .execute(|| Box::pin(async { Ok::<i32, McpError>(42) }), context)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_policy_success_after_retries() {
        let policy = RetryPolicy::new(RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            ..Default::default()
        });
        let context = ErrorContext::new("test_retry_operation");

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = policy
            .execute(
                move || {
                    let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                    Box::pin(async move {
                        if count < 3 {
                            Err(McpError::connection("Temporary failure"))
                        } else {
                            Ok::<i32, McpError>(42)
                        }
                    })
                },
                context,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_policy_non_recoverable_error() {
        let policy = RetryPolicy::new(RetryConfig {
            max_attempts: 3,
            respect_recoverability: true,
            ..Default::default()
        });
        let context = ErrorContext::new("test_non_recoverable");

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = policy
            .execute(
                move || {
                    attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Err::<i32, McpError>(McpError::validation("Invalid input")) })
                },
                context,
            )
            .await;

        assert!(result.is_err());
        // Should only attempt once for non-recoverable errors
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(100),
            ..Default::default()
        });

        let context = ErrorContext::new("test_circuit_breaker");

        // Cause failures to open the circuit
        for _ in 0..3 {
            let result = circuit_breaker
                .call(
                    async { Err::<(), McpError>(McpError::connection("Service down")) },
                    &context,
                )
                .await;
            assert!(result.is_err());
        }

        // Circuit should now be open
        assert_eq!(circuit_breaker.state().await, CircuitState::Open);

        // Next call should fail immediately
        let result = circuit_breaker
            .call(async { Ok::<(), McpError>(()) }, &context)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Circuit breaker is open")
        );
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(50),
            success_threshold: 2,
            ..Default::default()
        });

        let context = ErrorContext::new("test_recovery");

        // Open the circuit
        for _ in 0..2 {
            let _ = circuit_breaker
                .call(
                    async { Err::<(), McpError>(McpError::connection("Failure")) },
                    &context,
                )
                .await;
        }

        assert_eq!(circuit_breaker.state().await, CircuitState::Open);

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should transition to half-open and allow requests
        let result = circuit_breaker
            .call(async { Ok::<(), McpError>(()) }, &context)
            .await;
        assert!(result.is_ok());

        // After enough successes, should close
        let result = circuit_breaker
            .call(async { Ok::<(), McpError>(()) }, &context)
            .await;
        assert!(result.is_ok());

        assert_eq!(circuit_breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_retry_with_circuit_breaker() {
        let policy = RetryPolicy::with_circuit_breaker(
            RetryConfig {
                max_attempts: 2,
                initial_delay_ms: 10,
                ..Default::default()
            },
            CircuitBreakerConfig {
                failure_threshold: 2,
                recovery_timeout: Duration::from_millis(100),
                ..Default::default()
            },
        );

        let context = ErrorContext::new("test_combined");
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        // This should fail and open the circuit breaker
        let result = policy
            .execute(
                move || {
                    attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Err::<(), McpError>(McpError::connection("Service down")) })
                },
                context,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2); // Should retry once

        // Circuit breaker should have stats
        let stats = policy.circuit_breaker_stats().await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.failure_count, 2);
    }

    #[tokio::test]
    async fn test_retry_config_variations() {
        // Test conservative config
        let conservative = RetryConfig::conservative();
        assert_eq!(conservative.max_attempts, 2);
        assert_eq!(conservative.initial_delay_ms, 500);

        // Test aggressive config
        let aggressive = RetryConfig::aggressive();
        assert_eq!(aggressive.max_attempts, 5);
        assert_eq!(aggressive.initial_delay_ms, 100);

        // Test network config
        let network = RetryConfig::network();
        assert_eq!(network.max_attempts, 4);
        assert_eq!(network.initial_delay_ms, 200);
    }

    #[tokio::test]
    async fn test_delay_calculation() {
        let policy = RetryPolicy::new(RetryConfig {
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            enable_jitter: false,
            ..Default::default()
        });

        // Test exponential backoff without jitter
        let delay1 = policy.calculate_delay(1, Duration::from_millis(100));
        let delay2 = policy.calculate_delay(2, Duration::from_millis(100));
        let delay3 = policy.calculate_delay(3, Duration::from_millis(100));

        assert_eq!(delay1, Duration::from_millis(1000));
        assert_eq!(delay2, Duration::from_millis(2000));
        assert_eq!(delay3, Duration::from_millis(4000));
    }

    #[tokio::test]
    async fn test_delay_calculation_with_cap() {
        let policy = RetryPolicy::new(RetryConfig {
            initial_delay_ms: 1000,
            max_delay_ms: 3000,
            backoff_multiplier: 2.0,
            enable_jitter: false,
            ..Default::default()
        });

        let delay3 = policy.calculate_delay(3, Duration::from_millis(100));
        let delay4 = policy.calculate_delay(4, Duration::from_millis(100));

        // Should be capped at max_delay_ms
        assert_eq!(delay3, Duration::from_millis(3000));
        assert_eq!(delay4, Duration::from_millis(3000));
    }
}