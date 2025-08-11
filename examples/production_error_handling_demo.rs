// ! Production Error Handling Demo
// !
// ! This example demonstrates the production-ready error handling capabilities
// ! of the MCP SDK, including smart retry logic, circuit breakers,
// ! complete health checking, and structured logging with metrics.

use prism_mcp_rs::core::{
    error::McpError,
    health::{HealthChecker, ProtocolHealthCheck, ResourceHealthCheck, TransportHealthCheck},
    logging::{ErrorContext, ErrorLogger},
    metrics::global_metrics,
    retry::{CircuitBreakerConfig, RetryConfig, RetryPolicy},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("# MCP SDK Production Error Handling Demo");

    // Demo 1: Error Categorization and Recoverability
    demo_error_categorization().await;

    // Demo 2: smart Retry Logic
    demo_smart_retry().await;

    // Demo 3: Circuit Breaker Pattern
    demo_circuit_breaker().await;

    // Demo 4: complete Health Checking
    demo_health_checking().await;

    // Demo 5: Structured Logging and Metrics
    demo_logging_and_metrics().await;

    // Demo 6: End-to-End Production Scenario
    demo_production_scenario().await;

    info!("[x] All demos completed successfully!");
    Ok(())
}

async fn demo_error_categorization() {
    info!("\n📊 Demo 1: Error Categorization and Recoverability");

    let errors = vec![
        McpError::connection("Network connection failed"),
        McpError::timeout("Request timed out"),
        McpError::validation("Invalid input parameters"),
        McpError::protocol("Malformed JSON-RPC message"),
        McpError::Authentication("Invalid credentials".to_string()),
        McpError::internal("System error occurred"),
    ];

    for error in errors {
        info!(
            "Error: {} | Category: {} | Recoverable: {}",
            error,
            error.category(),
            error.is_recoverable()
        );
    }
}

async fn demo_smart_retry() {
    info!("\n🔄 Demo 2: smart Retry Logic");

    let retry_config = RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 100,
        respect_recoverability: true,
        ..Default::default()
    };

    let policy = RetryPolicy::new(retry_config);
    let attempt_count = Arc::new(AtomicU32::new(0));

    info!("Testing recoverable error with retry...");

    let attempt_count_clone = attempt_count.clone();
    let context = ErrorContext::new("demo_retry")
        .with_transport("demo")
        .with_method("test_operation");

    let result = policy
        .execute(
            move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                info!("  Attempt {}", count);

                Box::pin(async move {
                    if count < 3 {
                        Err(McpError::connection("Temporary network issue"))
                    } else {
                        Ok("Operation successful!")
                    }
                })
            },
            context,
        )
        .await;

    match result {
        Ok(message) => info!("[x] Retry succeeded: {}", message),
        Err(e) => error!("[!] Retry failed: {}", e),
    }

    info!(
        "Total attempts made: {}",
        attempt_count.load(Ordering::SeqCst)
    );

    // Test non-recoverable error (should not retry)
    info!("\nTesting non-recoverable error (should not retry)...");
    attempt_count.store(0, Ordering::SeqCst);
    let attempt_count_clone = attempt_count.clone();

    let context = ErrorContext::new("demo_no_retry")
        .with_transport("demo")
        .with_method("validation_operation");

    let result = policy
        .execute(
            move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                info!("  Attempt {}", count);

                Box::pin(async move {
                    Err::<&str, McpError>(McpError::validation("Invalid input - will not retry"))
                })
            },
            context,
        )
        .await;

    match result {
        Ok(_) => info!("[x] Unexpected success"),
        Err(e) => info!("[x] Expected failure (no retry): {}", e),
    }

    info!(
        "Attempts made for non-recoverable error: {}",
        attempt_count.load(Ordering::SeqCst)
    );
}

async fn demo_circuit_breaker() {
    info!("\n* Demo 3: Circuit Breaker Pattern");

    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay_ms: 50,
        ..Default::default()
    };

    let circuit_config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout: Duration::from_millis(500),
        success_threshold: 2,
        ..Default::default()
    };

    let policy = RetryPolicy::with_circuit_breaker(retry_config, circuit_config);
    let context = ErrorContext::new("demo_circuit_breaker")
        .with_transport("demo")
        .with_component("circuit_breaker_demo");

    info!("Causing failures to open circuit breaker...");

    // Cause failures to open the circuit
    for i in 1..=4 {
        let result = policy
            .execute(
                || {
                    Box::pin(async {
                        Err::<(), McpError>(McpError::connection("Service unavailable"))
                    })
                },
                context.clone(),
            )
            .await;

        info!("  Failure attempt {}: {:?}", i, result.is_err());

        if let Some(stats) = policy.circuit_breaker_stats().await {
            info!(
                "    Circuit state: {:?}, Failures: {}",
                stats.state, stats.failure_count
            );
        }
    }

    // Try a request when circuit is open
    info!("\nTrying request when circuit is open...");
    let result = policy
        .execute(
            || Box::pin(async { Ok::<(), McpError>(()) }),
            context.clone(),
        )
        .await;

    match result {
        Ok(_) => warn!("Unexpected success when circuit should be open"),
        Err(e) => {
            if e.to_string().contains("Circuit breaker is open") {
                info!("[x] Circuit breaker correctly blocked request: {}", e);
            } else {
                info!("[!] Request failed for different reason: {}", e);
            }
        }
    }

    // Wait for recovery timeout
    info!("\nWaiting for circuit breaker recovery timeout...");
    sleep(Duration::from_millis(600)).await;

    // Try recovery
    info!("Attempting recovery...");
    let result = policy
        .execute(|| Box::pin(async { Ok::<(), McpError>(()) }), context)
        .await;

    match result {
        Ok(_) => info!("[x] Circuit breaker allowed request through during recovery"),
        Err(e) => info!("Warning: Recovery attempt failed: {}", e),
    }

    if let Some(stats) = policy.circuit_breaker_stats().await {
        info!("Final circuit state: {:?}", stats.state);
    }
}

async fn demo_health_checking() {
    info!("\n🏥 Demo 4: complete Health Checking");

    let health_checker = HealthChecker::new()
        .add_check(TransportHealthCheck::new(
            "http_transport",
            "http",
            || async {
                // Simulate HTTP transport health check
                sleep(Duration::from_millis(50)).await;
                true // Healthy
            },
        ))
        .add_check(TransportHealthCheck::new(
            "websocket_transport",
            "websocket",
            || async {
                // Simulate WebSocket transport with intermittent issues
                sleep(Duration::from_millis(30)).await;
                fastrand::f32() > 0.3 // 70% chance of being healthy
            },
        ))
        .add_check(ProtocolHealthCheck::new("mcp_protocol", || async {
            // Simulate MCP protocol health check
            sleep(Duration::from_millis(40)).await;
            Ok(()) // Always healthy for demo
        }))
        .add_check(ResourceHealthCheck::new(
            "database_resource",
            "sqlite_db",
            || async {
                // Simulate database resource check
                sleep(Duration::from_millis(60)).await;
                true // Healthy
            },
        ));

    info!("Running complete health check...");
    let overall_health = health_checker.check_all().await;

    info!(
        "Overall Health Status: {} (Operational: {})",
        overall_health.status,
        overall_health.is_operational()
    );
    info!(
        "Health Summary: {} healthy, {} degraded, {} unhealthy",
        overall_health.healthy_count(),
        overall_health.degraded_count(),
        overall_health.unhealthy_count()
    );

    info!("Individual Health Check Results:");
    for (name, result) in &overall_health.checks {
        info!(
            "  {}: {} - {} (took {:?})",
            name, result.status, result.message, result.duration
        );

        if !result.metadata.is_empty() {
            info!("    Metadata: {:?}", result.metadata);
        }
    }
}

async fn demo_logging_and_metrics() {
    info!("\n📊 Demo 5: Structured Logging and Metrics");

    let metrics = global_metrics();

    // Simulate various error scenarios with logging
    let errors_and_contexts = vec![
        (
            McpError::timeout("Request timeout after 30s"),
            ErrorContext::new("api_request")
                .with_transport("http")
                .with_method("tools/list")
                .with_component("client")
                .with_session_id("session_123")
                .with_extra("timeout_duration", serde_json::Value::from(30000)),
        ),
        (
            McpError::connection("Failed to establish WebSocket connection"),
            ErrorContext::new("websocket_connect")
                .with_transport("websocket")
                .with_component("transport_layer")
                .with_extra("retry_attempt", serde_json::Value::from(2)),
        ),
        (
            McpError::validation("Invalid tool parameters"),
            ErrorContext::new("tool_execution")
                .with_transport("stdio")
                .with_method("tools/call")
                .with_component("server")
                .with_extra(
                    "tool_name",
                    serde_json::Value::String("file_reader".to_string()),
                ),
        ),
    ];

    info!("Logging errors with structured context...");
    for (error, context) in errors_and_contexts {
        ErrorLogger::log_error(&error, context).await;
    }

    // Simulate some successful operations for metrics
    metrics.record_request("tools/list", "http").await;
    metrics.record_request("resources/read", "websocket").await;
    metrics.record_connection_attempt("http", true).await;
    metrics.record_connection_attempt("websocket", false).await;

    // Display metrics summary
    let summary = metrics.get_all_metrics().await;

    info!("\nMetrics Summary:");
    info!("Error Metrics ({} entries):", summary.errors.len());
    for (key, count) in summary.errors.iter().take(5) {
        info!("  {}: {}", key, count);
    }

    info!("Request Metrics ({} entries):", summary.requests.len());
    for (key, count) in &summary.requests {
        info!("  {}: {}", key, count);
    }

    info!(
        "Connection Metrics ({} entries):",
        summary.connections.len()
    );
    for (key, count) in &summary.connections {
        info!("  {}: {}", key, count);
    }
}

async fn demo_production_scenario() {
    info!("\n### Demo 6: End-to-End Production Scenario");
    info!("Simulating a realistic production error handling flow...");

    // Create production-ready configuration
    let retry_config = RetryConfig::network(); // Network-improved retry config
    let circuit_config = CircuitBreakerConfig::default();

    let policy = RetryPolicy::with_circuit_breaker(retry_config, circuit_config);
    let metrics = global_metrics();

    // Simulate a service operation that experiences intermittent failures
    let attempt_count = Arc::new(AtomicU32::new(0));
    let total_operations = 10;
    let mut successful_operations = 0;

    for operation_id in 1..=total_operations {
        let context = ErrorContext::new("production_api_call")
            .with_transport("http")
            .with_method("tools/execute")
            .with_component("production_service")
            .with_session_id("prod_session_456")
            .with_extra("operation_id", serde_json::Value::from(operation_id));

        let attempt_count_clone = attempt_count.clone();
        attempt_count.store(0, Ordering::SeqCst);

        let result = policy
            .execute(
                move || {
                    let _count = attempt_count_clone.fetch_add(1, Ordering::SeqCst) + 1;

                    Box::pin(async move {
                        // Simulate different failure scenarios
                        let failure_chance = fastrand::f32();

                        if failure_chance < 0.2 {
                            // 20% chance of timeout (recoverable)
                            Err(McpError::timeout("Service temporarily unavailable"))
                        } else if failure_chance < 0.3 {
                            // 10% chance of connection error (recoverable)
                            Err(McpError::connection("Network connectivity issue"))
                        } else if failure_chance < 0.35 {
                            // 5% chance of validation error (non-recoverable)
                            Err(McpError::validation("Invalid request format"))
                        } else {
                            // 65% chance of success
                            Ok(format!("Operation {operation_id} completed successfully"))
                        }
                    })
                },
                context,
            )
            .await;

        match result {
            Ok(message) => {
                successful_operations += 1;
                info!("[x] {}", message);
            }
            Err(e) => {
                error!("[!] Operation {} failed: {}", operation_id, e);
            }
        }

        // Small delay between operations
        sleep(Duration::from_millis(100)).await;
    }

    info!("\n📈 Production Scenario Results:");
    info!("Total operations: {}", total_operations);
    info!("Successful operations: {}", successful_operations);
    info!(
        "Success rate: {:.1}%",
        (successful_operations as f32 / total_operations as f32) * 100.0
    );

    // Display circuit breaker stats
    if let Some(stats) = policy.circuit_breaker_stats().await {
        info!("Circuit breaker state: {:?}", stats.state);
        info!("Total failures: {}", stats.failure_count);
    }

    // Display final metrics
    let final_metrics = metrics.get_all_metrics().await;
    info!(
        "Final metrics - Errors: {}, Requests: {}, Retries: {}",
        final_metrics.errors.len(),
        final_metrics.requests.len(),
        final_metrics.retries.len()
    );

    // Run final health check
    let health_checker = HealthChecker::new()
        .add_check(ProtocolHealthCheck::new("production_health", || async {
            Ok(())
        }));

    let health = health_checker.check_all().await;
    info!(
        "Final system health: {} (operational: {})",
        health.status,
        health.is_operational()
    );

    info!("\n Production scenario completed successfully!");
    info!("   The MCP SDK demonstrated resilient error handling with:");
    info!("   • smart retry logic based on error recoverability");
    info!("   • Circuit breaker protection against cascading failures");
    info!("   • complete structured logging and metrics");
    info!("   • Production-ready health monitoring");
}
