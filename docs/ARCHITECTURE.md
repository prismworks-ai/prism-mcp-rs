# Architecture

`prism-mcp-rs` is a Tokio-based library organized around protocol types, registries and handlers, a common server dispatch path, clients, and replaceable transports.

## Module map

| Module | Responsibility |
|--------|----------------|
| `protocol` | JSON-RPC/MCP methods, messages, capabilities, and wire types |
| `core` | Tools, resources, prompts, completion, errors, resilience, and logging primitives |
| `server` | Registration, validation, policy enforcement, routing, and lifecycle |
| `client` | MCP client operations, sessions, and request handling |
| `transport` | STDIO, HTTP, WebSocket, SSE, custom transport traits, and endpoint pools |
| `security` | Principal/context types, authorization, and in-process rate limiting |
| `auth` | OAuth-oriented client primitives; available with HTTP support |
| `telemetry` | Optional OTLP exporter setup and tracing integration |
| `plugin` | Optional trusted native dynamic-plugin loading and tool registration |

## Request path

1. A `ServerTransport` receives and decodes a JSON-RPC request.
2. The transport handler invokes the shared `McpServer` dispatch path.
3. An authentication adapter may call `handle_request_with_context`; the compatibility path uses an anonymous context.
4. `RequestPolicy` authorizes the normalized method/resource target and applies the rate limiter.
5. Request and MCP parameters are validated when validation is enabled.
6. Dispatch routes to a built-in operation or registered tool/resource/prompt handler.
7. The response is serialized by the transport.

All built-in transports install the same server handler. Custom authenticated transports should construct `RequestContext` only after validating credentials; caller-provided identity data is not trustworthy by itself.

## Concurrency and ownership

Registries are shared through `Arc` plus asynchronous or concurrent maps. Handlers must be `Send + Sync`, should avoid blocking the Tokio runtime, and should enforce their own downstream timeouts. CPU-heavy work belongs in `spawn_blocking` or a dedicated worker pool.

The in-process rate limiter stores independent token buckets keyed by principal and method. `prune_idle` is explicit so request processing never performs a full-map scan. It is not a distributed quota.

## Transport model

- `Transport` models client-side request/notification exchange.
- `ServerTransport` accepts a request-handler callback and owns server lifecycle.
- STDIO is the default and reserves stdout for frames.
- HTTP exposes MCP, notification, SSE, and health routes; TLS is opt-in.
- WebSocket provides bidirectional transport when enabled.
- `EndpointPoolTransport` selects healthy endpoints round-robin and keeps reactive circuit state.

Failover is intentionally conservative: naturally read-only MCP methods can be retried at another endpoint; mutations are attempted once unless an application-controlled idempotency key is present. The SDK does not provide backend deduplication, service discovery, active probes, or exactly-once execution.

## Security boundaries

| Boundary | SDK behavior | Host responsibility |
|----------|--------------|---------------------|
| Identity | Carries a `Principal` in `RequestContext` | Verify credentials and create the context |
| Authorization | Optional deny-by-default RBAC | Define and test least-privilege policy |
| Rate limiting | Optional process-local token bucket | Add global/edge limits and concurrency bounds |
| Network | Optional TLS 1.3 mTLS for HTTP | Issue/rotate/revoke certificates and protect keys |
| Observability | Structured tracing and optional OTLP | Operate collector, sampling, retention, and redaction |
| Plugins | Loads trusted native libraries in process | Verify provenance or isolate in a separate process |

The default policy remains allow-all for compatibility. Installing `RbacAuthorizer` changes authorization to deny by default.

## Extensibility

Prefer normal Rust handlers for application code. Use custom transports when a platform has its own framing or identity boundary. Use native plugins only when hot-loaded, trusted code is a requirement; they share the process address space and have no resource sandbox.

## Performance model

The maintained Criterion suite measures selected registry operations, common server dispatch, and recoverable endpoint failover. It does not establish production throughput or tail latency. Network topology, handler work, payloads, enabled features, logging, and allocator behavior dominate real deployments. See [Performance](guides/performance.md).

## Planned, not implemented

- sandboxed WebAssembly plugins with enforceable CPU/memory limits;
- active endpoint health checks and service-discovery adapters;
- cluster-wide policy or rate-limit state;
- SDK-managed CPU affinity;
- exactly-once mutation semantics.

Keeping these items explicit prevents deployment architecture from depending on roadmap claims.
