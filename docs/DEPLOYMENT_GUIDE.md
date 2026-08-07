# Deployment Guide

`prism-mcp-rs` is an SDK, not a standalone daemon. Deployment configuration belongs to the binary that embeds it. This guide separates the two common operating models.

## Choose a transport

### Local AI-tool integration: STDIO

Build a release binary and configure the AI tool to launch its absolute path:

```bash
cargo build --release --locked
```

STDIO is appropriate for one client launching one local server. Reserve stdout for MCP frames and write logs to stderr or a file. The launching client supplies the process environment, working directory, and lifecycle.

### Network service: HTTP

Enable only the required features, normally `http,tls,otel` plus application-specific auth support. Bind explicitly and put the service behind a managed ingress or load balancer unless direct mTLS is required.

```toml
prism-mcp-rs = {
    version = "2",
    default-features = false,
    features = ["http", "tls", "otel"]
}
```

The built-in HTTP server provides `/mcp`, `/mcp/notify`, `/mcp/events`, and `/health`. Its permissive CORS behavior and generic health response may not match a public deployment; constrain exposure with an ingress and add application readiness checks around dependencies.

## Recommended production profile

- Run as an unprivileged user on a read-only root filesystem where practical.
- Authenticate at the gateway or custom transport boundary, then construct `RequestContext`.
- Install deny-by-default RBAC and per-principal rate limits.
- Enforce body-size, header, connection, concurrency, and request-timeout limits at the ingress/host.
- Use mTLS for service-to-service identity or terminate TLS at a hardened managed proxy.
- Export structured traces and logs with credential/payload redaction.
- Disable native plugins unless every loaded artifact is trusted.
- Give the process explicit CPU and memory limits; tune after measuring the target workload.

The [Production Controls guide](PRODUCTION_CONTROLS.md) contains the SDK configuration examples.

## Container baseline

Use a multi-stage image: build with the locked dependency graph, copy only the binary and required CA material into a small runtime image, create a non-root user, and expose only the selected port. Do not bake private keys or bearer tokens into image layers.

The exact base image depends on the binary's native dependencies and organizational policy. Verify the final image with the same architecture and feature set used in production.

## Configuration and secrets

The SDK does not reserve environment-variable names or load a configuration file. The host application should document:

- every variable and its default;
- whether reload is supported;
- the precedence of flags, files, and environment variables;
- which fields are secrets; and
- validation performed at startup.

Load keys and tokens from a managed secret store or mounted secret volume. Fail startup on invalid certificates, impossible rate limits, or missing mandatory policy rather than silently weakening the service.

## Health and observability

Use separate signals:

- liveness: the process/event loop can respond;
- readiness: required dependencies and policy/configuration are usable;
- startup: slow initialization has completed.

The SDK HTTP `/health` route is a basic transport liveness check. Add application-level readiness outside it when database, queue, model, or plugin availability matters.

Keep the OTLP guard alive for the process lifetime and flush it during graceful shutdown. Set sampling and log retention in the observability platform. Never record authorization headers, private keys, tokens, or unredacted sensitive tool arguments.

## Scaling and failover

Scale stateless server instances behind a platform load balancer. Remember that SDK token buckets and endpoint circuit state are process-local. Use a shared or edge limiter for global quotas.

`EndpointPoolTransport` provides reactive round-robin selection and safe replay rules for an MCP client. It does not discover instances or probe them in the background. Feed it endpoints from the platform's discovery mechanism and rebuild/update client state when membership changes.

## Rollout and recovery

Before rollout:

1. run format, lint, full-feature tests, docs tests, security audit, and release build;
2. validate certificates and secret references in staging;
3. exercise allow and deny policy paths plus rate-limit behavior;
4. run representative load tests and set alerts from measured baselines;
5. verify graceful termination and in-flight request handling;
6. record the previous image digest and configuration for rollback.

Use rolling or canary deployment with automated health gates. Keep rollback under five minutes by retaining the prior artifact and avoiding irreversible schema/config changes in the same step. Back up application data where applicable; this crate itself stores no durable data.

## Verification commands

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo audit --deny warnings
cargo build --release --locked --features "http,tls,otel"
```

Run load tests on production-like hardware. Checked-in benchmark results are diagnostic snapshots, not capacity planning data.
