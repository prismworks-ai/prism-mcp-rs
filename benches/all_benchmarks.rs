//! CI/CD compatible benchmark runner
//!
//! This benchmark file provides compatibility with `cargo bench` for CI/CD workflows
//! while internally leveraging our standalone utility benchmarks. This ensures
//! GitHub Actions workflows and badges continue to function properly.

#![cfg(feature = "bench")]

use criterion::{criterion_group, criterion_main, Criterion};
use prism_mcp_rs::plugin::{PluginCapabilities, PluginConfig, PluginMetadata};
use prism_mcp_rs::protocol::{ContentBlock, Tool, ToolInputSchema};
use serde_json::json;
use std::collections::HashMap;
use std::hint::black_box;

// Core plugin benchmarks (simplified versions of our utility benchmarks)
fn benchmark_plugin_creation(c: &mut Criterion) {
    c.bench_function("plugin_config_creation", |b| {
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
}

fn benchmark_tool_registration(c: &mut Criterion) {
    c.bench_function("tool_registration", |b| {
        b.iter(|| {
            let mut tools = HashMap::new();
            let tool = Tool {
                name: "calculator".to_string(),
                description: Some("Perform calculations".to_string()),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(HashMap::from([(
                        "expression".to_string(),
                        json!({"type": "string"}),
                    )])),
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
}

fn benchmark_tool_lookup(c: &mut Criterion) {
    let registry: HashMap<String, Tool> = (0..100)
        .map(|i| {
            (
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
                },
            )
        })
        .collect();

    c.bench_function("tool_lookup", |b| {
        b.iter(|| registry.get(black_box("tool_50")));
    });
}

fn benchmark_plugin_lifecycle(c: &mut Criterion) {
    c.bench_function("plugin_metadata_creation", |b| {
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

    c.bench_function("call_tool_result_generation", |b| {
        b.iter(|| {
            let result = prism_mcp_rs::protocol::CallToolResult {
                content: vec![ContentBlock::text("Operation completed successfully")],
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
}

criterion_group!(
    benches,
    benchmark_plugin_creation,
    benchmark_tool_registration,
    benchmark_tool_lookup,
    benchmark_plugin_lifecycle
);

criterion_main!(benches);
