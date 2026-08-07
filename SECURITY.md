# Security Policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Email `security@prismworks.ai` with:

- the affected version and feature set;
- impact and realistic attack scenario;
- reproduction steps or a minimal proof of concept;
- any suggested mitigation; and
- a safe way to contact you.

The project aims to acknowledge reports within 48 hours, provide an initial assessment within seven days, and coordinate disclosure after a fix or mitigation is available. These are response targets, not contractual guarantees.

## Supported versions

| Version | Support |
|---------|---------|
| 2.x | Security fixes |
| 1.x and earlier | Upgrade required |

## Security model

The crate is a library, so security depends on enabled features and host integration.

### Implemented controls

- Core request handling is written in safe Rust. Dynamic native plugin loading has a narrow `unsafe` FFI boundary.
- `RbacAuthorizer` is deny by default once installed and can match MCP methods plus resource/tool patterns.
- `RateLimiter` enforces an in-process token bucket per principal and MCP method.
- HTTP mTLS requires TLS 1.3, verifies client certificates against the configured client CA, and lets clients verify servers against a configured CA.
- HTTP tracing can propagate W3C Trace Context and export spans through OTLP.
- Endpoint failover replays naturally idempotent methods. Potentially mutating methods are not replayed unless the caller provides an idempotency key.
- JSON-RPC and MCP validation can reject malformed input before handlers run.

### Important boundaries

- Authentication is not enabled by default. The host must validate credentials and build a trustworthy `RequestContext`.
- The default `RequestPolicy` allows requests for backward compatibility. Production services should install explicit RBAC and rate limiting.
- The rate limiter is process-local. Use an edge or distributed limiter for a cluster-wide quota or volumetric denial-of-service protection.
- Native plugins are trusted in-process code. The crate does not sandbox plugins or enforce plugin CPU/memory limits.
- TLS is opt-in. Plain HTTP remains available for trusted local networks or deployments terminated by a secure proxy.
- Certificate issuance, rotation, revocation, key storage, and certificate-to-principal mapping are deployment responsibilities.
- The crate does not provide MFA, API-key storage, encryption at rest, data retention policy, active endpoint health checks, or service discovery.
- CPU affinity and OS/container resource limits belong to the host runtime and deployment platform.

See [Production Controls](docs/PRODUCTION_CONTROLS.md) for concrete configuration guidance.

## Deployment checklist

- Authenticate before dispatch and never trust caller-supplied identity fields.
- Use least-privilege RBAC; test permitted and denied paths.
- Apply request, body-size, concurrency, and time limits at the gateway or host as well as SDK rate limits.
- Use mTLS or a hardened TLS-terminating proxy for network deployments.
- Keep secrets out of source control and logs; rotate them through a managed secret store.
- Disable native plugins unless their provenance and build pipeline are trusted.
- Preserve request IDs and trace IDs in security logs without recording credentials or sensitive payloads.
- Run dependency and policy checks against the exact release lockfile.
- Define rollback, incident response, and certificate-expiry alerts before production rollout.

## Security verification

```bash
cargo audit --deny warnings
cargo deny check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
```

`cargo audit` reports the state of the dependency graph at the time it runs. A clean result must not be presented as a permanent guarantee.

## Unsafe code and plugins

Unsafe code should remain confined to the native plugin loading boundary. Changes to FFI types, symbol loading, ownership, or unloading require focused review and tests. Do not load untrusted native libraries; process isolation is the safe choice until a sandboxed runtime exists.

Last reviewed: 2026-08-06.
