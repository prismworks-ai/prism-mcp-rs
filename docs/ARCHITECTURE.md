# Architecture

`prism-mcp-rs` is a Tokio-based library organized around protocol types, registries and handlers, a common server dispatch path, clients, and replaceable transports.

## Module map

| Module | Responsibility |
|--------|----------------|
| `protocol` | JSON-RPC/MCP methods, revision negotiation, request envelopes, capabilities, and wire types |
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
2. HTTP validates 2026 routing and schema-declared parameter headers against the body.
3. The shared dispatch path classifies the request as stateless 2026 or stateful 2025 and enforces `ProtocolMode`.
4. An authentication adapter may call `handle_request_with_context`; the compatibility path uses an anonymous context.
5. `RequestPolicy` authorizes the normalized method/resource target and applies the rate limiter.
6. Request and MCP parameters are validated when validation is enabled.
7. Dispatch routes to a built-in operation or registered tool/resource/prompt handler.
8. Modern results receive `resultType`, server identity, and conservative cache metadata; legacy results retain their 2025 shape.
9. Long-lived subscription streams route only explicitly selected notifications and stamp their request ID; task execution uses the caller-bound task registry outside the request lifetime.

All built-in transports install the same server handler. Custom authenticated transports should construct `RequestContext` only after validating credentials; caller-provided identity data is not trustworthy by itself.

## Protocol selection

The 2026 lifecycle is stateless: `server/discover` replaces initialization, and every request carries its protocol revision, client identity, and capabilities in `_meta`. The 2025 lifecycle retains `initialize`, connection-scoped capability negotiation, and initialized notifications.

In automatic mode, clients probe 2026 first. Only JSON-RPC `-32601` for `server/discover` permits a 2025 fallback. STDIO uses a disposable probe process so a legacy server receives initialization on a clean process. Other failures are returned to the caller. This narrow fallback rule prevents an authentication, routing, or version error from silently weakening protocol behavior.

## Concurrency and ownership

Registries are shared through `Arc` plus asynchronous or concurrent maps. Handlers must be `Send + Sync`, should avoid blocking the Tokio runtime, and should enforce their own downstream timeouts. CPU-heavy work belongs in `spawn_blocking` or a dedicated worker pool.

The in-process rate limiter stores independent token buckets keyed by principal and method. `prune_idle` is explicit so request processing never performs a full-map scan. It is not a distributed quota.

## Transport model

- `Transport` models client-side request/notification exchange.
- `ServerTransport` accepts a request-handler callback and owns server lifecycle.
- STDIO is the default and reserves stdout for frames.
- HTTP uses `/mcp` for standards-track requests, notifications, and request-scoped subscription SSE; `/mcp/notify` and `/mcp/events` remain legacy compatibility routes. TLS is opt-in.
- WebSocket provides bidirectional transport when enabled.
- `EndpointPoolTransport` selects healthy endpoints round-robin and keeps reactive circuit state.

The proprietary advanced chunked/compressed transport is a legacy Prism extension. Automatic selection never chooses it, and its client helpers reject modern protocol modes.

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
