//! OpenTelemetry setup for exporting the request spans emitted by the SDK.
//!
//! Enable the `otel` feature and call [`init_otlp_tracing`] once at process
//! startup. The server request path is instrumented even when this feature is
//! disabled, so applications may also install their own tracing subscriber.

use crate::core::error::{McpError, McpResult};
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// OTLP trace export configuration.
#[derive(Debug, Clone)]
pub struct OtlpTracingConfig {
    pub service_name: String,
    pub endpoint: String,
    pub filter: String,
}

impl OtlpTracingConfig {
    pub fn new(service_name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            endpoint: endpoint.into(),
            filter: "info".to_string(),
        }
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = filter.into();
        self
    }
}

/// Flushes batched spans when dropped or explicitly shut down.
pub struct TelemetryGuard {
    provider: Option<SdkTracerProvider>,
}

impl TelemetryGuard {
    pub fn shutdown(mut self) -> McpResult<()> {
        self.provider
            .take()
            .expect("telemetry provider is present until shutdown")
            .shutdown()
            .map_err(|error| McpError::Internal(format!("OpenTelemetry shutdown failed: {error}")))
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take() {
            if let Err(error) = provider.shutdown() {
                tracing::warn!(%error, "OpenTelemetry provider did not shut down cleanly");
            }
        }
    }
}

/// Install an OTLP/gRPC exporter and a W3C Trace Context propagator.
///
/// This function must be called from within a Tokio runtime and before another
/// global tracing subscriber has been installed.
pub fn init_otlp_tracing(config: OtlpTracingConfig) -> McpResult<TelemetryGuard> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(config.endpoint)
        .build()
        .map_err(|error| McpError::Internal(format!("failed to build OTLP exporter: {error}")))?;

    let resource = Resource::builder()
        .with_service_name(config.service_name)
        .build();
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());
    let tracer = global::tracer("prism-mcp-rs");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let filter = EnvFilter::try_new(config.filter)
        .map_err(|error| McpError::Validation(format!("invalid tracing filter: {error}")))?;

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .try_init()
        .map_err(|error| {
            McpError::Internal(format!("failed to install tracing subscriber: {error}"))
        })?;

    Ok(TelemetryGuard {
        provider: Some(provider),
    })
}
