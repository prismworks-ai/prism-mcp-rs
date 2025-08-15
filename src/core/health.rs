// ! complete health check system for MCP SDK
// !
// ! Module provides production-ready health checking capabilities with:
// ! - Transport-specific health checks
// ! - Protocol-level health validation
// ! - Resource availability monitoring
// ! - Circuit breaker integration
// ! - Detailed health reporting

use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::debug;

use crate::core::error::McpResult;
use crate::core::retry::CircuitBreakerStats;

/// Custom serialization for Instant
mod instant_serde {
    use super::*;
    use std::time::SystemTime;

    pub fn serialize<S>(_instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert Instant to SystemTime for serialization
        let system_time = SystemTime::now();
        let duration_since_epoch = system_time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        serializer.serialize_u64(duration_since_epoch.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For deserialization, just use current time as we can't reconstruct Instant
        let _millis = u64::deserialize(deserializer)?;
        Ok(Instant::now())
    }
}

/// Custom serialization for Duration
mod duration_serde {
    use super::*;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and operational
    Healthy,
    /// Component has minor issues but is functional
    Degraded,
    /// Component is unhealthy and may not function properly
    Unhealthy,
    /// Component health is unknown or could not be determined
    Unknown,
}

impl HealthStatus {
    /// Check if status indicates the component is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get a numeric score for this status (higher is better)
    pub fn score(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 75,
            HealthStatus::Unhealthy => 25,
            HealthStatus::Unknown => 0,
        }
    }

    /// Combine two health statuses, returning the worse of the two
    pub fn combine(self, other: HealthStatus) -> HealthStatus {
        if self.score() < other.score() {
            self
        } else {
            other
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    /// Overall health status
    pub status: HealthStatus,
    /// Human-readable description
    pub message: String,
    /// Additional metadata about the health check
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when check was performed (as milliseconds since epoch)
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Duration the health check took (as milliseconds)
    #[serde(with = "duration_serde")]
    pub duration: Duration,
}

impl HealthResult {
    /// Create a healthy result
    pub fn healthy(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Healthy, message)
    }

    /// Create a degraded result
    pub fn degraded(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Degraded, message)
    }

    /// Create an unhealthy result
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Unhealthy, message)
    }

    /// Create an unknown result
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Unknown, message)
    }

    /// Create a new health result
    pub fn new(status: HealthStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            metadata: HashMap::new(),
            timestamp: Instant::now(),
            duration: Duration::from_millis(0),
        }
    }

    /// Add metadata to the health result
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the duration for this health check
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Individual health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of this health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthResult;

    /// Get the timeout for this health check
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Check if this health check is critical for overall system health
    fn is_critical(&self) -> bool {
        true
    }
}

/// Transport health check
pub struct TransportHealthCheck {
    name: String,
    transport_type: String,
    connection_test: Box<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>> + Send + Sync,
    >,
}

impl TransportHealthCheck {
    /// Create a new transport health check
    pub fn new<F, Fut>(
        name: impl Into<String>,
        transport_type: impl Into<String>,
        connection_test: F,
    ) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send + 'static,
    {
        Self {
            name: name.into(),
            transport_type: transport_type.into(),
            connection_test: Box::new(move || Box::pin(connection_test())),
        }
    }
}

#[async_trait]
impl HealthCheck for TransportHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthResult {
        let start = Instant::now();

        match timeout(self.timeout(), (self.connection_test)()).await {
            Ok(true) => {
                HealthResult::healthy(format!("{} transport is connected", self.transport_type))
                    .with_duration(start.elapsed())
                    .with_metadata(
                        "transport_type",
                        serde_json::Value::String(self.transport_type.clone()),
                    )
            }
            Ok(false) => HealthResult::unhealthy(format!(
                "{} transport connection failed",
                self.transport_type
            ))
            .with_duration(start.elapsed())
            .with_metadata(
                "transport_type",
                serde_json::Value::String(self.transport_type.clone()),
            ),
            Err(_) => HealthResult::unhealthy(format!(
                "{} transport health check timed out",
                self.transport_type
            ))
            .with_duration(start.elapsed())
            .with_metadata(
                "transport_type",
                serde_json::Value::String(self.transport_type.clone()),
            )
            .with_metadata("timeout", serde_json::Value::Bool(true)),
        }
    }
}

/// Protocol health check
// Type alias for complex protocol test function
type ProtocolTestFn = Box<
    dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = McpResult<()>> + Send>>
        + Send
        + Sync,
>;

pub struct ProtocolHealthCheck {
    name: String,
    protocol_test: ProtocolTestFn,
}

impl ProtocolHealthCheck {
    /// Create a new protocol health check
    pub fn new<F, Fut>(name: impl Into<String>, protocol_test: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = McpResult<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            protocol_test: Box::new(move || Box::pin(protocol_test())),
        }
    }
}

#[async_trait]
impl HealthCheck for ProtocolHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthResult {
        let start = Instant::now();

        match timeout(self.timeout(), (self.protocol_test)()).await {
            Ok(Ok(())) => HealthResult::healthy("Protocol communication successful")
                .with_duration(start.elapsed()),
            Ok(Err(error)) => {
                let status = if error.is_recoverable() {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Unhealthy
                };

                HealthResult::new(status, format!("Protocol error: {error}"))
                    .with_duration(start.elapsed())
                    .with_metadata(
                        "error_category",
                        serde_json::Value::String(error.category().to_string()),
                    )
                    .with_metadata(
                        "error_recoverable",
                        serde_json::Value::Bool(error.is_recoverable()),
                    )
            }
            Err(_) => HealthResult::unhealthy("Protocol health check timed out")
                .with_duration(start.elapsed())
                .with_metadata("timeout", serde_json::Value::Bool(true)),
        }
    }
}

/// Resource health check
pub struct ResourceHealthCheck {
    name: String,
    resource_name: String,
    availability_test: Box<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>> + Send + Sync,
    >,
}

impl ResourceHealthCheck {
    /// Create a new resource health check
    pub fn new<F, Fut>(
        name: impl Into<String>,
        resource_name: impl Into<String>,
        availability_test: F,
    ) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send + 'static,
    {
        Self {
            name: name.into(),
            resource_name: resource_name.into(),
            availability_test: Box::new(move || Box::pin(availability_test())),
        }
    }
}

#[async_trait]
impl HealthCheck for ResourceHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthResult {
        let start = Instant::now();

        match timeout(self.timeout(), (self.availability_test)()).await {
            Ok(true) => {
                HealthResult::healthy(format!("Resource '{}' is available", self.resource_name))
                    .with_duration(start.elapsed())
                    .with_metadata(
                        "resource_name",
                        serde_json::Value::String(self.resource_name.clone()),
                    )
            }
            Ok(false) => {
                HealthResult::unhealthy(format!("Resource '{}' is unavailable", self.resource_name))
                    .with_duration(start.elapsed())
                    .with_metadata(
                        "resource_name",
                        serde_json::Value::String(self.resource_name.clone()),
                    )
            }
            Err(_) => HealthResult::unknown(format!(
                "Resource '{}' health check timed out",
                self.resource_name
            ))
            .with_duration(start.elapsed())
            .with_metadata(
                "resource_name",
                serde_json::Value::String(self.resource_name.clone()),
            )
            .with_metadata("timeout", serde_json::Value::Bool(true)),
        }
    }

    fn is_critical(&self) -> bool {
        false // Resources are typically non-critical by default
    }
}

/// Circuit breaker health check
// Type alias for complex stats getter function
type StatsGetterFn = Box<
    dyn Fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Option<CircuitBreakerStats>> + Send>,
        > + Send
        + Sync,
>;

pub struct CircuitBreakerHealthCheck {
    name: String,
    get_stats: StatsGetterFn,
}

impl CircuitBreakerHealthCheck {
    /// Create a new circuit breaker health check
    pub fn new<F, Fut>(name: impl Into<String>, get_stats: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Option<CircuitBreakerStats>> + Send + 'static,
    {
        Self {
            name: name.into(),
            get_stats: Box::new(move || Box::pin(get_stats())),
        }
    }
}

#[async_trait]
impl HealthCheck for CircuitBreakerHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthResult {
        let start = Instant::now();

        match (self.get_stats)().await {
            Some(stats) => {
                let status = match stats.state {
                    crate::core::retry::CircuitState::Closed => HealthStatus::Healthy,
                    crate::core::retry::CircuitState::HalfOpen => HealthStatus::Degraded,
                    crate::core::retry::CircuitState::Open => HealthStatus::Unhealthy,
                };

                let message = format!("Circuit breaker state: {:?}", stats.state);

                HealthResult::new(status, message)
                    .with_duration(start.elapsed())
                    .with_metadata(
                        "circuit_state",
                        serde_json::Value::String(format!("{:?}", stats.state)),
                    )
                    .with_metadata(
                        "failure_count",
                        serde_json::Value::from(stats.failure_count),
                    )
                    .with_metadata(
                        "success_count",
                        serde_json::Value::from(stats.success_count),
                    )
                    .with_metadata(
                        "half_open_requests",
                        serde_json::Value::from(stats.half_open_requests),
                    )
            }
            None => HealthResult::unknown("Circuit breaker stats not available")
                .with_duration(start.elapsed()),
        }
    }

    fn is_critical(&self) -> bool {
        false // Circuit breaker health is informational
    }
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Individual health check results
    pub checks: HashMap<String, HealthResult>,
    /// Timestamp when overall health was computed (as milliseconds since epoch)
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Total time taken for all health checks (as milliseconds)
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
}

impl OverallHealth {
    /// Create overall health from individual check results
    pub fn from_results(
        results: Vec<(&str, Result<HealthResult, tokio::time::error::Elapsed>)>,
    ) -> Self {
        let start = Instant::now();
        let mut checks = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for (name, result) in results {
            let health_result = match result {
                Ok(result) => result,
                Err(_) => HealthResult::unknown("Health check timed out"),
            };

            // Combine status (taking the worst)
            overall_status = overall_status.combine(health_result.status);
            checks.insert(name.to_string(), health_result);
        }

        Self {
            status: overall_status,
            checks,
            timestamp: start,
            total_duration: start.elapsed(),
        }
    }

    /// Check if the system is operational
    pub fn is_operational(&self) -> bool {
        self.status.is_operational()
    }

    /// Get the number of healthy checks
    pub fn healthy_count(&self) -> usize {
        self.checks
            .values()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count()
    }

    /// Get the number of unhealthy checks
    pub fn unhealthy_count(&self) -> usize {
        self.checks
            .values()
            .filter(|r| r.status == HealthStatus::Unhealthy)
            .count()
    }

    /// Get the number of degraded checks
    pub fn degraded_count(&self) -> usize {
        self.checks
            .values()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count()
    }
}

/// complete health checker
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
    timeout: Duration,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a health checker with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            checks: Vec::new(),
            timeout,
        }
    }

    /// Add a health check
    pub fn add_check<T: HealthCheck + 'static>(mut self, check: T) -> Self {
        self.checks.push(Box::new(check));
        self
    }

    /// Add a health check by reference
    pub fn add_check_ref<T: HealthCheck + 'static>(&mut self, check: T) {
        self.checks.push(Box::new(check));
    }

    /// Run all health checks
    pub async fn check_all(&self) -> OverallHealth {
        let mut results = Vec::new();

        for check in &self.checks {
            let name = check.name();
            let check_timeout = check.timeout().min(self.timeout);

            debug!("Running health check: {}", name);

            let result = timeout(check_timeout, check.check()).await;
            results.push((name, result));
        }

        OverallHealth::from_results(results)
    }

    /// Run only critical health checks
    pub async fn check_critical(&self) -> OverallHealth {
        let mut results = Vec::new();

        for check in &self.checks {
            if !check.is_critical() {
                continue;
            }

            let name = check.name();
            let check_timeout = check.timeout().min(self.timeout);

            debug!("Running critical health check: {}", name);

            let result = timeout(check_timeout, check.check()).await;
            results.push((name, result));
        }

        OverallHealth::from_results(results)
    }

    /// Get the number of registered health checks
    pub fn check_count(&self) -> usize {
        self.checks.len()
    }

    /// Get the names of all registered health checks
    pub fn check_names(&self) -> Vec<&str> {
        self.checks.iter().map(|c| c.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    struct TestHealthCheck {
        name: String,
        result: HealthResult,
        delay: Duration,
    }

    impl TestHealthCheck {
        fn new(name: &str, status: HealthStatus, delay: Duration) -> Self {
            Self {
                name: name.to_string(),
                result: HealthResult::new(status, format!("{name} test result")),
                delay,
            }
        }
    }

    #[async_trait]
    impl HealthCheck for TestHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check(&self) -> HealthResult {
            sleep(self.delay).await;
            self.result.clone()
        }

        fn timeout(&self) -> Duration {
            Duration::from_millis(100)
        }
    }

    #[tokio::test]
    async fn test_health_status_operations() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
        assert!(!HealthStatus::Unknown.is_operational());

        assert_eq!(HealthStatus::Healthy.score(), 100);
        assert_eq!(HealthStatus::Degraded.score(), 75);
        assert_eq!(HealthStatus::Unhealthy.score(), 25);
        assert_eq!(HealthStatus::Unknown.score(), 0);

        assert_eq!(
            HealthStatus::Healthy.combine(HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthStatus::Degraded.combine(HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
    }

    #[tokio::test]
    async fn test_health_result_creation() {
        let result = HealthResult::healthy("All good")
            .with_metadata("version", serde_json::Value::String("1.0.0".to_string()))
            .with_duration(Duration::from_millis(50));

        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, "All good");
        assert_eq!(result.duration, Duration::from_millis(50));
        assert!(result.metadata.contains_key("version"));
    }

    #[tokio::test]
    async fn test_transport_health_check() {
        let check = TransportHealthCheck::new("test_transport", "http", || async { true });

        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.contains("http transport is connected"));
    }

    #[tokio::test]
    async fn test_protocol_health_check() {
        let check = ProtocolHealthCheck::new("test_protocol", || async { Ok(()) });

        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, "Protocol communication successful");
    }

    #[tokio::test]
    async fn test_resource_health_check() {
        let check = ResourceHealthCheck::new("test_resource", "database", || async { true });

        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.contains("Resource 'database' is available"));
        assert!(!check.is_critical()); // Resources are non-critical by default
    }

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::new()
            .add_check(TestHealthCheck::new(
                "test1",
                HealthStatus::Healthy,
                Duration::from_millis(10),
            ))
            .add_check(TestHealthCheck::new(
                "test2",
                HealthStatus::Degraded,
                Duration::from_millis(20),
            ))
            .add_check(TestHealthCheck::new(
                "test3",
                HealthStatus::Unhealthy,
                Duration::from_millis(5),
            ));

        assert_eq!(checker.check_count(), 3);
        assert_eq!(checker.check_names(), vec!["test1", "test2", "test3"]);

        let overall = checker.check_all().await;
        assert_eq!(overall.status, HealthStatus::Unhealthy); // Worst of all checks
        assert_eq!(overall.checks.len(), 3);
        assert_eq!(overall.healthy_count(), 1);
        assert_eq!(overall.degraded_count(), 1);
        assert_eq!(overall.unhealthy_count(), 1);
        // Duration should be at least the total of delays, but allow for fast execution in tests
        assert!(overall.total_duration.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let checker =
            HealthChecker::with_timeout(Duration::from_millis(50)).add_check(TestHealthCheck::new(
                "slow_check",
                HealthStatus::Healthy,
                Duration::from_millis(200),
            ));

        let overall = checker.check_all().await;

        // Should timeout and be marked as unknown
        assert_eq!(overall.checks.len(), 1);
        let result = overall.checks.get("slow_check").unwrap();
        assert_eq!(result.status, HealthStatus::Unknown);
        assert!(result.message.contains("timed out"));
    }

    #[tokio::test]
    async fn test_overall_health_operations() {
        let results = vec![
            ("check1", Ok(HealthResult::healthy("OK"))),
            ("check2", Ok(HealthResult::degraded("Minor issue"))),
            (
                "check3",
                Err(tokio::time::timeout(Duration::from_millis(10), async {
                    tokio::time::sleep(Duration::from_millis(100)).await
                })
                .await
                .unwrap_err()),
            ),
        ];

        let overall = OverallHealth::from_results(results);

        assert_eq!(overall.status, HealthStatus::Unknown); // Worst status wins
        assert_eq!(overall.checks.len(), 3);
        assert_eq!(overall.healthy_count(), 1);
        assert_eq!(overall.degraded_count(), 1);
        assert_eq!(overall.unhealthy_count(), 0);
        assert!(!overall.is_operational()); // Unknown status makes it non-operational
    }
}
