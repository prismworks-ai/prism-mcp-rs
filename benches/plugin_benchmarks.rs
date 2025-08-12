//! Plugin system performance benchmarks
//! 
//! Measures tool registration, execution, and plugin lifecycle management.

#![cfg(feature = "bench")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prism_mcp_rs::plugin::{PluginConfig, PluginMetadata, PluginCapabilities};
use prism_mcp_rs::protocol::{Tool, ToolInputSchema, ContentBlock};
use serde_json::json;
use std::collections::HashMap;

fn benchmark_plugin_creation(c: &mut Criterion) {
    c.bench_function("plugin_config_empty", |b| {
        b.iter(|| {
            let config = PluginConfig {
                name: "benchmark-plugin".to_string(),
                enabled: true,
                path: None,
                config: None,
                env: HashMap::new(),
                auto_reload: false,
                priority: 100,
            };
            black_box(config);
        });
    });

    c.bench_function("plugin_config_with_env", |b| {
        b.iter(|| {
            let mut config = PluginConfig {
                name: "benchmark-plugin".to_string(),
                enabled: true,
                path: Some("/path/to/plugin.so".to_string()),
                config: Some(json!({
                    "timeout": 30,
                    "max_retries": 3
                })),
                env: HashMap::new(),
                auto_reload: false,
                priority: 100,
            };
            
            // Add environment variables
            for i in 0..10 {
                config.env.insert(
                    format!("VAR_{}", i),
                    format!("value_{}", i),
                );
            }
            
            black_box(config);
        });
    });
}

fn benchmark_tool_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_registration");
    
    group.bench_function("register_single_tool", |b| {
        b.iter(|| {
            let mut tools = HashMap::new();
            let tool = Tool {
                name: "calculator".to_string(),
                description: Some("Perform calculations".to_string()),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(HashMap::from([
                        ("expression".to_string(), json!({"type": "string"})),
                    ])),
                    required: Some(vec!["expression".to_string()]),
                    additional_properties: HashMap::new(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            };
            tools.insert(black_box("calculator".to_string()), tool);
        });
    });
    
    group.bench_function("register_10_tools", |b| {
        b.iter(|| {
            let mut tools = HashMap::new();
            for i in 0..10 {
                let tool = Tool {
                    name: format!("tool_{}", i),
                    description: Some(format!("Tool number {}", i)),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(HashMap::from([
                            ("input".to_string(), json!({"type": "string"})),
                            ("config".to_string(), json!({"type": "object"})),
                        ])),
                        required: None,
                        additional_properties: HashMap::new(),
                    },
                    output_schema: None,
                    annotations: None,
                    title: None,
                    meta: None,
                };
                tools.insert(format!("tool_{}", i), tool);
            }
            black_box(tools);
        });
    });
    
    group.bench_function("register_100_tools", |b| {
        b.iter(|| {
            let mut tools = HashMap::new();
            for i in 0..100 {
                let tool = Tool {
                    name: format!("tool_{}", i),
                    description: Some(format!("Tool number {}", i)),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(HashMap::from([
                            ("input".to_string(), json!({"type": "string"})),
                        ])),
                        required: None,
                        additional_properties: HashMap::new(),
                    },
                    output_schema: None,
                    annotations: None,
                    title: None,
                    meta: None,
                };
                tools.insert(format!("tool_{}", i), tool);
            }
            black_box(tools);
        });
    });
    
    group.finish();
}

fn benchmark_tool_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_lookup");
    
    // Create registry with various sizes
    let small_registry: HashMap<String, Tool> = (0..10)
        .map(|i| (
            format!("tool_{}", i),
            Tool {
                name: format!("Tool {}", i),
                description: Some(format!("Description {}", i)),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: None,
                    required: None,
                    additional_properties: HashMap::new(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            }
        ))
        .collect();
    
    let medium_registry: HashMap<String, Tool> = (0..100)
        .map(|i| (
            format!("tool_{}", i),
            Tool {
                name: format!("Tool {}", i),
                description: Some(format!("Description {}", i)),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: None,
                    required: None,
                    additional_properties: HashMap::new(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            }
        ))
        .collect();
    
    let large_registry: HashMap<String, Tool> = (0..1000)
        .map(|i| (
            format!("tool_{}", i),
            Tool {
                name: format!("Tool {}", i),
                description: Some(format!("Description {}", i)),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: None,
                    required: None,
                    additional_properties: HashMap::new(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            }
        ))
        .collect();
    
    group.bench_function("lookup_in_10", |b| {
        b.iter(|| {
            small_registry.get(black_box("tool_5"))
        });
    });
    
    group.bench_function("lookup_in_100", |b| {
        b.iter(|| {
            medium_registry.get(black_box("tool_50"))
        });
    });
    
    group.bench_function("lookup_in_1000", |b| {
        b.iter(|| {
            large_registry.get(black_box("tool_500"))
        });
    });
    
    group.bench_function("lookup_missing", |b| {
        b.iter(|| {
            large_registry.get(black_box("nonexistent_tool"))
        });
    });
    
    group.finish();
}

fn benchmark_tool_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_execution");
    
    // Simple tool call parameters
    let simple_params = json!({"message": "Hello, World!"});
    
    group.bench_function("execute_simple", |b| {
        b.iter(|| {
            // Simulate tool execution
            let params = black_box(&simple_params);
            let result = json!({
                "output": params["message"].as_str().unwrap(),
                "timestamp": "2024-01-01T00:00:00Z"
            });
            black_box(result);
        });
    });
    
    // Complex tool parameters with validation
    let complex_params = json!({
        "query": "SELECT * FROM users WHERE age > ?",
        "params": [25],
        "options": {
            "timeout": 30,
            "cache": true,
            "format": "json"
        }
    });
    
    group.bench_function("execute_complex", |b| {
        b.iter(|| {
            // Simulate parameter validation
            let params = black_box(&complex_params);
            let has_query = params.get("query").is_some();
            let has_params = params.get("params").is_some();
            
            if has_query && has_params {
                // Simulate execution
                let result = json!({
                    "rows": [
                        {"id": 1, "name": "User 1", "age": 30},
                        {"id": 2, "name": "User 2", "age": 28},
                    ],
                    "count": 2,
                    "execution_time_ms": 15
                });
                black_box(result);
            }
        });
    });
    
    // Batch tool execution
    let batch_params: Vec<_> = (0..10).map(|i| json!({
        "tool": format!("tool_{}", i % 3),
        "input": format!("data_{}", i),
        "index": i
    })).collect();
    
    group.bench_function("execute_batch_10", |b| {
        b.iter(|| {
            let results: Vec<_> = batch_params.iter().map(|params| {
                // Simulate execution for each tool
                json!({
                    "tool": params["tool"].as_str().unwrap(),
                    "result": format!("Processed: {:?}", params["input"]),
                    "success": true
                })
            }).collect();
            black_box(results);
        });
    });
    
    group.finish();
}

fn benchmark_plugin_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_lifecycle");
    
    // Plugin metadata creation
    group.bench_function("create_metadata", |b| {
        b.iter(|| {
            let metadata = PluginMetadata {
                id: "lifecycle-plugin".to_string(),
                name: "Lifecycle Plugin".to_string(),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                description: Some("Test plugin for benchmarking".to_string()),
                homepage: None,
                license: Some("MIT".to_string()),
                mcp_version: "2025-06-18".to_string(),
                capabilities: PluginCapabilities {
                    hot_reload: true,
                    configurable: true,
                    health_check: true,
                    thread_safe: true,
                    multi_instance: false,
                    custom: json!({}),
                },
                dependencies: vec![],
            };
            black_box(metadata);
        });
    });
    
    // Plugin state management
    group.bench_function("state_update", |b| {
        let mut state = json!({
            "counter": 0,
            "history": [],
            "metadata": {}
        });
        
        b.iter(|| {
            // Simulate state update
            state["counter"] = json!(state["counter"].as_i64().unwrap() + 1);
            if let Some(history) = state["history"].as_array_mut() {
                history.push(json!({
                    "action": "increment",
                    "timestamp": "2024-01-01T00:00:00Z"
                }));
                
                // Keep history limited
                if history.len() > 100 {
                    history.remove(0);
                }
            }
            black_box(&state);
        });
    });
    
    // Plugin result generation
    group.bench_function("generate_result", |b| {
        b.iter(|| {
            // Simulate generating a CallToolResult
            let result = prism_mcp_rs::protocol::CallToolResult {
                content: vec![
                    ContentBlock::text("Operation completed successfully"),
                ],
                is_error: Some(false),
                structured_content: Some(json!({
                    "status": "success",
                    "metrics": {
                        "duration_ms": 42,
                        "operations": 10
                    }
                })),
                meta: None,
            };
            black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_plugin_creation,
    benchmark_tool_registration,
    benchmark_tool_lookup,
    benchmark_tool_execution,
    benchmark_plugin_lifecycle
);

criterion_main!(benches);