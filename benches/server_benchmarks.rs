//! Server performance benchmarks
//!
//! Measures request handling, routing efficiency,
//! and concurrent request processing.

#![cfg(feature = "bench")]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prism_mcp_rs::protocol::{ErrorObject, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::collections::HashMap;

fn benchmark_server_creation(c: &mut Criterion) {
    c.bench_function("server_config_default", |b| {
        b.iter(|| {
            // Benchmark creating server configuration
            let config = HashMap::from([
                ("name".to_string(), json!("benchmark-server")),
                ("version".to_string(), json!("1.0.0")),
            ]);
            black_box(config);
        });
    });

    c.bench_function("server_config_with_capabilities", |b| {
        b.iter(|| {
            let config = HashMap::from([
                ("name".to_string(), json!("benchmark-server")),
                ("version".to_string(), json!("1.0.0")),
                (
                    "capabilities".to_string(),
                    json!({
                        "tools": {"listChanged": true},
                        "resources": {"subscribe": true},
                        "prompts": {"listChanged": true}
                    }),
                ),
            ]);
            black_box(config);
        });
    });
}

fn benchmark_request_routing(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_routing");

    // Simple request routing
    let simple_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "1.0.0",
            "capabilities": {}
        })),
        id: json!(1),
    };

    group.bench_function("route_simple", |b| {
        b.iter(|| {
            // Simulate routing logic
            let method = black_box(&simple_request.method);
            match method.as_str() {
                "initialize" => Some("handle_initialize"),
                "tools/list" => Some("handle_tools_list"),
                "tools/execute" => Some("handle_tools_execute"),
                _ => None,
            }
        });
    });

    // Complex nested routing
    let complex_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/execute".to_string(),
        params: Some(json!({
            "tool": "database.query",
            "args": {
                "query": "SELECT * FROM users",
                "params": []
            }
        })),
        id: json!(2),
    };

    group.bench_function("route_complex", |b| {
        b.iter(|| {
            let method = black_box(&complex_request.method);
            let parts: Vec<&str> = method.split('/').collect();
            match parts.as_slice() {
                ["tools", action] => match *action {
                    "list" => Some("list_handler"),
                    "execute" => Some("execute_handler"),
                    _ => None,
                },
                ["resources", action] => match *action {
                    "read" => Some("read_handler"),
                    "write" => Some("write_handler"),
                    _ => None,
                },
                _ => None,
            }
        });
    });

    group.finish();
}

fn benchmark_response_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_generation");

    // Success response
    group.bench_function("generate_success", |b| {
        b.iter(|| {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                result: Some(json!({
                    "status": "success",
                    "data": black_box("test_data"),
                })),
            };
            let _json = serde_json::to_string(&response).unwrap();
        });
    });

    // Error response
    group.bench_function("generate_error", |b| {
        b.iter(|| {
            let response = JsonRpcError {
                jsonrpc: "2.0".to_string(),
                id: json!(null),
                error: ErrorObject {
                    code: -32600,
                    message: "Invalid Request".to_string(),
                    data: Some(json!({
                        "details": black_box("Missing required field"),
                    })),
                },
            };
            let _json = serde_json::to_string(&response).unwrap();
        });
    });

    // Large response with tool results
    let tool_results: Vec<_> = (0..20)
        .map(|i| {
            json!({
                "tool_id": format!("tool_{}", i),
                "name": format!("Tool {}", i),
                "description": format!("Description for tool {}", i),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"},
                        "options": {"type": "object"}
                    }
                }
            })
        })
        .collect();

    group.bench_function("generate_large", |b| {
        b.iter(|| {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!("batch-request"),
                result: Some(json!({
                    "tools": black_box(&tool_results),
                    "total": tool_results.len(),
                })),
            };
            let _json = serde_json::to_string(&response).unwrap();
        });
    });

    group.finish();
}

fn benchmark_concurrent_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_handling");

    // Simulate concurrent request processing
    let requests: Vec<JsonRpcRequest> = (0..100)
        .map(|i| JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "echo".to_string(),
            params: Some(json!({"message": format!("test_{}", i)})),
            id: json!(i),
        })
        .collect();

    group.bench_function("process_100_sequential", |b| {
        b.iter(|| {
            for request in black_box(&requests) {
                // Simulate processing
                let _result = json!({
                    "echo": request.params.as_ref().unwrap()["message"].as_str()
                });
            }
        });
    });

    group.bench_function("process_100_parallel_sim", |b| {
        b.iter(|| {
            // Simulate parallel processing with chunks
            let chunk_size = 10;
            for chunk in black_box(&requests).chunks(chunk_size) {
                // Process chunk in "parallel"
                let _results: Vec<_> = chunk
                    .iter()
                    .map(|req| {
                        json!({
                            "echo": req.params.as_ref().unwrap()["message"].as_str()
                        })
                    })
                    .collect();
            }
        });
    });

    group.finish();
}

fn benchmark_middleware_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("middleware");

    // Simulate middleware chain processing
    group.bench_function("chain_3_middlewares", |b| {
        b.iter(|| {
            let mut context = json!({
                "request_id": "test-123",
                "timestamp": "2024-01-01T00:00:00Z",
                "data": {}
            });

            // Middleware 1: Authentication
            context["auth"] = json!({"user": "test", "validated": true});

            // Middleware 2: Logging
            context["log"] = json!({"level": "info", "message": "Processing request"});

            // Middleware 3: Rate limiting
            context["rate_limit"] = json!({"remaining": 99, "reset": 3600});

            black_box(context);
        });
    });

    group.bench_function("chain_5_middlewares", |b| {
        b.iter(|| {
            let mut context = json!({
                "request_id": "test-456",
                "timestamp": "2024-01-01T00:00:00Z",
                "data": {}
            });

            // Middleware 1: Authentication
            context["auth"] = json!({"user": "test", "validated": true});

            // Middleware 2: Logging
            context["log"] = json!({"level": "info", "message": "Processing request"});

            // Middleware 3: Rate limiting
            context["rate_limit"] = json!({"remaining": 99, "reset": 3600});

            // Middleware 4: Validation
            context["validation"] = json!({"schema": "v1", "valid": true});

            // Middleware 5: Metrics
            context["metrics"] = json!({"latency_ms": 5, "cpu_usage": 0.15});

            black_box(context);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_server_creation,
    benchmark_request_routing,
    benchmark_response_generation,
    benchmark_concurrent_handling,
    benchmark_middleware_chain
);

criterion_main!(benches);
