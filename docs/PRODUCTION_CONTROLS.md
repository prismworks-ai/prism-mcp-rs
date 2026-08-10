# Production Controls

This guide describes controls implemented by the SDK and the boundaries that remain the application's responsibility. All request policies are enforced in the common `McpServer` dispatch path, independent of transport.

## Recommended production profile

- Authenticate at the transport or gateway boundary and construct a `RequestContext`.
- Use deny-by-default `RbacAuthorizer` permissions.
- Apply a per-principal/per-method token bucket.
- Enable mTLS for service-to-service HTTP deployments.
- Export request spans using the optional `otel` feature.
- Use `EndpointPoolTransport` for balancing and conservative failover.
- Keep native plugins disabled for untrusted code. They execute in-process.

## Fine-grained RBAC and rate limiting

```rust
use prism_mcp_rs::prelude::*;

let rbac = RbacAuthorizer::new([
    Permission::new("operator", "tools/list"),
    Permission::new("operator", "tools/call").for_resource("report_*"),
    Permission::new("reader", "resources/read").for_resource("urn:public:*")
]);

let rate_limit = RateLimiter::new(RateLimitConfig::new(20, 10.0)?);
let policy = RequestPolicy::new(rbac).with_rate_limiter(rate_limit);
let server = McpServer::create("production-server", "3.0.0")
    .with_request_policy(policy);
# Ok::<(), prism_mcp_rs::McpError>(())
```

Permissions support exact matching or a final `*`. RBAC denies requests without a matching role, method, and optional resource pattern. Rate limits are keyed by principal ID and MCP method. The limiter is local to one process; clustered deployments should add an edge or distributed limiter when a global quota is required.

An authenticated adapter calls `handle_request_with_context`:

```rust
let context = RequestContext::new(
    Principal::new("service:billing")
        .with_role("operator")
        .with_authentication_method("mtls"),
)
.with_transport("http")
.with_peer_address("10.0.0.12:54321");

let response = server.handle_request_with_context(request, context).await?;
# Ok::<(), prism_mcp_rs::McpError>(())
```

The built-in `handle_request` remains backwards compatible and uses an anonymous principal. A deny-by-default policy will therefore reject it unless the anonymous identity is explicitly permitted.

## Mutual TLS

Enable `http,tls`. Client identity PEM must contain its certificate chain and private key.

```rust,ignore
use prism_mcp_rs::transport::{
    HttpClientTransport, HttpServerTransport, MtlsClientConfig, MtlsServerConfig,
    TransportConfig,
};

let server_transport = HttpServerTransport::new("0.0.0.0:8443").with_mtls(
    MtlsServerConfig::new(server_cert_chain, server_private_key, client_ca),
);

let client_transport = HttpClientTransport::with_mtls(
    "https://mcp.internal.example",
    None,
    TransportConfig::default(),
    MtlsClientConfig::new(client_identity, server_ca),
).await?;
```

The server requires a certificate chaining to the configured client CA and the client requires TLS 1.3 and validates the server against its configured CA. Certificate rotation, revocation policy, secret storage, and mapping certificate subjects to application principals remain deployment responsibilities.

## OpenTelemetry distributed tracing

Enable `otel` and initialize the exporter once, from inside the Tokio runtime:

```rust,ignore
use prism_mcp_rs::telemetry::{init_otlp_tracing, OtlpTracingConfig};

let telemetry = init_otlp_tracing(
    OtlpTracingConfig::new("billing-mcp", "http://otel-collector:4317")
        .with_filter("prism_mcp_rs=info"),
)?;

// Keep the guard alive, then flush before process exit.
telemetry.shutdown()?;
```

The HTTP client injects W3C Trace Context headers, the HTTP server extracts them, and common request dispatch emits `mcp.request` spans plus structured audit events. Applications that already own the global tracing subscriber should install their own `tracing-opentelemetry` layer instead of calling this initializer.

## Load balancing and safe failover

```rust,ignore
use prism_mcp_rs::transport::{
    EndpointPoolConfig, EndpointPoolTransport, HttpClientTransport,
};

let primary = HttpClientTransport::new("https://mcp-a.internal", None).await?;
let secondary = HttpClientTransport::new("https://mcp-b.internal", None).await?;
let transport = EndpointPoolTransport::new(EndpointPoolConfig::default())
    .add_endpoint("a", primary)
    .add_endpoint("b", secondary);
```

Healthy endpoints are selected round-robin. Recoverable failures open an endpoint circuit after the configured threshold. Read-only methods may be replayed on another endpoint. `tools/call` and other potentially mutating operations are attempted once unless `params._meta.idempotencyKey` contains a non-empty application-controlled key. Backends must implement deduplication for keyed operations; the SDK cannot guarantee exactly-once execution.

The pool is reactive and process-local. Service discovery and active health probes should update or rebuild pools in the host application until native discovery is added.

## Deliberate deferrals

### Sandboxed plugins

The current `plugin` feature loads trusted native dynamic libraries into the server process. It does not provide a security sandbox or enforce memory/CPU limits. The recommended next implementation is a separate Wasmtime-based WebAssembly plugin runtime with fuel, epoch interruption, store limits, capability-scoped host calls, and signed plugin manifests. This is intentionally separate from the native ABI so existing plugin behavior is not mislabeled as isolation.

### CPU affinity

The SDK does not pin threads. Affinity is workload-, topology-, runtime-, and container-specific, and can reduce performance or interfere with orchestration limits. First benchmark the new `server_request_dispatch_ping` and `endpoint_failover_read` paths under representative load. If pinning produces a repeatable material improvement, configure it in the host's Tokio runtime or deployment layer rather than making it a global SDK default.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo bench --features bench,plugin,http
```
