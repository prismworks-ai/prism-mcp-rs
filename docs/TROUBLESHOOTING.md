# Troubleshooting

Start with the smallest command that reproduces the failure and record the crate version, Rust version, enabled features, transport, OS, and complete error chain.

## Build failures

### Unsupported compiler

The minimum supported Rust version is 1.85.

```bash
rustc --version
rustup update stable
```

### Missing type or module

Most transports and integrations are feature-gated. Inspect the feature table in [the documentation index](README.md), then enable the required feature explicitly. `auth` is currently exposed with HTTP support, while mTLS requires both `http` and `tls` in practical use.

```bash
cargo tree -e features -p prism-mcp-rs
cargo check --all-features
```

### Duplicate or ambiguous imports

The prelude is convenient for applications, but broad globs can collide with application or plugin types. Replace a glob with explicit imports around the ambiguous name.

## STDIO problems

### Client starts the process and immediately disconnects

- Run the exact configured absolute command in a terminal.
- Check execute permissions and host architecture.
- Replace relative paths in arguments and environment-derived paths.
- Ensure the client is launching the release binary you just built.
- Inspect stderr; never add diagnostic `println!` calls.

### Invalid JSON or framing errors

STDOUT must contain only newline-delimited MCP/JSON-RPC messages. Move banners, debugging, and panic diagnostics to stderr or `tracing`. Confirm the process does not emit a UTF-8 BOM or shell wrapper output.

## HTTP problems

### Connection refused

Confirm the bind address is reachable from the caller and that the server remains alive. Container `127.0.0.1` refers to that container, not the host. Check port mappings and firewalls.

```bash
curl -v http://127.0.0.1:8080/health
```

### HTTP 404

The MCP request endpoint is `/mcp`; health is `/health`, notifications are `/mcp/notify`, and SSE events are `/mcp/events`.

### TLS or mTLS handshake failure

- Verify the server chain and private key form one identity.
- Verify the client identity PEM contains the client certificate chain and private key.
- Verify each side trusts the intended CA, not merely a leaf certificate.
- Check DNS names/SANs and certificate validity dates.
- Confirm both ends support TLS 1.3.

Use `openssl s_client` or the platform's TLS diagnostics without exposing private keys in logs.

## Authorization and throttling

### Every request is forbidden

`RbacAuthorizer` denies by default. Check that:

- authentication created the expected principal and role set;
- the transport calls `handle_request_with_context` rather than the anonymous compatibility path;
- the method pattern matches the normalized MCP method; and
- resource/tool patterns match the target extracted from request parameters.

Pattern matching supports exact values or a final `*`; it is not a general regular expression.

### Requests are unexpectedly allowed

The default `RequestPolicy` is allow-all. Verify that the configured server instance received `with_request_policy` or `set_request_policy` before starting its transport.

### Rate limited too early

Buckets are keyed by principal ID and method. Shared or anonymous principal IDs intentionally share a bucket. Validate `burst` and `requests_per_second`, and call `prune_idle` from a maintenance loop if high-cardinality identities churn. For multi-instance quotas, inspect the edge/distributed limiter as well.

## OpenTelemetry

### Subscriber initialization fails

Only one global tracing subscriber can be installed. If the application or framework already owns it, do not call `init_otlp_tracing`; compose a `tracing-opentelemetry` layer in the host subscriber instead.

### Traces do not connect across HTTP

Enable `otel` on both participating builds, initialize the W3C propagator before requests, and verify that proxies preserve `traceparent`/`tracestate`. Check collector endpoint, protocol, sampling, and export errors.

## Endpoint pool

### Mutating request does not fail over

This is the safe default. Potentially mutating methods are attempted once. Supply `params._meta.idempotencyKey` only when the backend deduplicates that key correctly.

### All endpoints unavailable

The pool's circuits open after consecutive recoverable errors and reset after cooldown. It has no active health probe. Check endpoint membership, cooldown, and the underlying transport errors; recreate/update the pool when discovery membership changes.

## Plugins

### Library cannot be loaded

Confirm the native library path, platform extension, architecture, exported symbols, and compatible plugin contract. Loader failures can also come from missing transitive native libraries.

### Plugin crashes or consumes excessive resources

Native plugins share the process and are not sandboxed. Disable the plugin and recover the service. Run untrusted or failure-prone extensions in a separate OS process/container with explicit resource limits.

## Collecting useful diagnostics

```bash
RUST_BACKTRACE=1 RUST_LOG=prism_mcp_rs=debug cargo test --all-features -- --nocapture
cargo metadata --format-version 1
cargo tree -e features
```

Redact tokens, authorization headers, private keys, personal data, and sensitive tool arguments before sharing logs.

## Before filing an issue

Provide a minimal reproduction, expected and actual behavior, exact commands, complete error output, version/feature information, and whether the issue reproduces on the current 2.x release. Use the private process in [SECURITY.md](../SECURITY.md) for suspected vulnerabilities.
