//! Client performance benchmarks
//! 
//! Measures transport layer performance, serialization speed,
//! and request/response handling efficiency.

#![cfg(feature = "bench")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use prism_mcp_rs::client::McpClientBuilder;
use prism_mcp_rs::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::time::Duration;

fn benchmark_client_creation(c: &mut Criterion) {
    c.bench_function("client_create_default", |b| {
        b.iter(|| {
            let builder = McpClientBuilder::new();
            let _config = black_box(builder);
        });
    });

    c.bench_function("client_create_with_config", |b| {
        b.iter(|| {
            let builder = McpClientBuilder::new()
                .with_name("benchmark-client")
                .with_version("1.0.0")
                .with_timeout(Duration::from_secs(30));
            let _config = black_box(builder);
        });
    });
}

fn benchmark_request_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_serialization");
    
    // Small request
    let small_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "test".to_string(),
        params: Some(json!({"key": "value"})),
        id: json!(1),
    };
    
    group.bench_with_input(
        BenchmarkId::new("small", "50_bytes"),
        &small_request,
        |b, req| b.iter(|| {
            let _json = serde_json::to_string(black_box(req)).unwrap();
        })
    );
    
    // Medium request with nested data
    let medium_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/execute".to_string(),
        params: Some(json!({
            "tool": "calculator",
            "args": {
                "expression": "2 + 2",
                "variables": {
                    "x": 10,
                    "y": 20,
                    "z": 30
                }
            },
            "metadata": {
                "timestamp": "2024-01-01T00:00:00Z",
                "user_id": "test-user",
                "session_id": "test-session"
            }
        })),
        id: json!(42),
    };
    
    group.bench_with_input(
        BenchmarkId::new("medium", "500_bytes"),
        &medium_request,
        |b, req| b.iter(|| {
            let _json = serde_json::to_string(black_box(req)).unwrap();
        })
    );
    
    // Large request with array data
    let large_data: Vec<_> = (0..100).map(|i| json!({
        "id": i,
        "name": format!("item_{}", i),
        "value": i * 2,
        "metadata": {
            "created": "2024-01-01T00:00:00Z",
            "modified": "2024-01-01T00:00:00Z"
        }
    })).collect();
    
    let large_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "batch/process".to_string(),
        params: Some(json!({
            "items": large_data,
            "options": {
                "parallel": true,
                "timeout": 60
            }
        })),
        id: json!("batch-123"),
    };
    
    group.bench_with_input(
        BenchmarkId::new("large", "10KB"),
        &large_request,
        |b, req| b.iter(|| {
            let _json = serde_json::to_string(black_box(req)).unwrap();
        })
    );
    
    group.finish();
}

fn benchmark_response_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_deserialization");
    
    // Small response
    let small_response = r#"{
        "jsonrpc": "2.0",
        "result": {"status": "ok"},
        "id": 1
    }"#;
    
    group.bench_with_input(
        BenchmarkId::new("small", "50_bytes"),
        &small_response,
        |b, resp| b.iter(|| {
            let _parsed: JsonRpcResponse = serde_json::from_str(black_box(resp)).unwrap();
        })
    );
    
    // Medium response
    let medium_response = r#"{
        "jsonrpc": "2.0",
        "result": {
            "tools": [
                {"name": "calculator", "version": "1.0.0"},
                {"name": "weather", "version": "2.1.0"},
                {"name": "translate", "version": "3.0.0"}
            ],
            "capabilities": {
                "streaming": true,
                "batch": true,
                "async": true
            }
        },
        "id": "req-456"
    }"#;
    
    group.bench_with_input(
        BenchmarkId::new("medium", "500_bytes"),
        &medium_response,
        |b, resp| b.iter(|| {
            let _parsed: JsonRpcResponse = serde_json::from_str(black_box(resp)).unwrap();
        })
    );
    
    // Large response with structured data
    let large_response = format!(r#"{{
        "jsonrpc": "2.0",
        "result": {{
            "data": [{}],
            "timestamp": "2024-01-01T00:00:00Z",
            "request_id": "req-789"
        }},
        "id": 123
    }}"#, 
        (0..50).map(|i| format!(r#"{{"id": {}, "value": "item_{}"}},"#, i, i))
            .collect::<Vec<_>>().join(",")
    );
    
    group.bench_with_input(
        BenchmarkId::new("large", "5KB"),
        &large_response,
        |b, resp| b.iter(|| {
            let _parsed: JsonRpcResponse = serde_json::from_str(black_box(resp)).unwrap();
        })
    );
    
    group.finish();
}

fn benchmark_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    // Benchmark creating batch requests
    let requests: Vec<JsonRpcRequest> = (0..10).map(|i| JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: format!("method_{}", i),
        params: Some(json!({"index": i})),
        id: json!(i),
    }).collect();
    
    group.bench_function("create_batch_10", |b| {
        b.iter(|| {
            let _batch = black_box(&requests).clone();
        });
    });
    
    // Benchmark serializing batch
    group.bench_function("serialize_batch_10", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&requests)).unwrap();
        });
    });
    
    // Benchmark processing batch responses
    let responses: Vec<String> = (0..10).map(|i| 
        format!(r#"{{"jsonrpc":"2.0","result":{{"value":{}}},"id":{}}}"#, i, i)
    ).collect();
    
    let batch_response = format!("[{}]", responses.join(","));
    
    group.bench_function("deserialize_batch_10", |b| {
        b.iter(|| {
            let _parsed: Vec<JsonRpcResponse> = 
                serde_json::from_str(black_box(&batch_response)).unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_client_creation,
    benchmark_request_serialization,
    benchmark_response_deserialization,
    benchmark_batch_operations
);

criterion_main!(benches);