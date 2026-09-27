# Prism MCP Rust SDK product strategy

Effective: 2026-09-24. Active direction for new work. Implementation is tracked in the [plan](implementation-plan.md); proposed additions are not shipped features.

## Purpose and boundary

**The reusable Rust connectivity and generic request-control substrate for MCP clients and servers.**

Own protocol correctness, transport lifecycle and generic authorization/context hooks. Databridle supplies enterprise policy through an adapter. Remain a general MIT-licensed SDK usable without any other PrismWorks product.

## Current repository evidence

Review of the local working tree, including pre-existing changes; source presence is not deployment evidence.

| Source | Observation |
| --- | --- |
| [Cargo.toml](../Cargo.toml) | Local package version 3.0.1 with explicit transport/security feature flags. |
| [src/security.rs](../src/security.rs) | RequestContext, Authorizer, RequestPolicy and deny-by-default RBAC exist; default RequestPolicy uses AllowAllAuthorizer for compatibility. |
| [src/auth/client.rs](../src/auth/client.rs) | HTTP authorization client logic exists. |
| [docs/PROTOCOL_VERSIONS.md](PROTOCOL_VERSIONS.md) | Protocol-version behavior is documented; support claims need matching version tests. |
| [README.md](../README.md) | Native plugins are explicitly not a security boundary. |

Verification this review: Source and manifest review only in this pass; SDK tests were not rerun. Downstream tools resolution was tested and fails against the local SDK because their manifests request ^0.1.0.

## Retain

Retain working behavior, user data, existing integrations, tests, licensing and independent use within the boundary above. Maintain existing support obligations. Preserve local changes from other work; this review does not release or deploy code.

## Remove or defer

- Keep tenant administration, hosted policy storage, billing, agent orchestration and credential-provider business logic outside the SDK.
- Remove any implication that native dynamic plugins provide isolation or that an SDK default constitutes production authorization.

## Modify

- Expose generic trusted-context and action-policy hooks without a compulsory Databridle or UAICP dependency.
- Pin and test each advertised MCP protocol/transport combination; distinguish existing custom transports from standard wire compatibility.
- Provide protected-service examples that explicitly install authentication and restrictive policy while preserving semver compatibility for general SDK users.

## Add

- Optional adapter example for Databridle decision/receipt mapping with tenant/audience/request binding.
- Conformance fixtures for denied calls, cancellation, retries and transport identity propagation.
- A published compatibility manifest for SDK, example crates and optional adapter versions.

## Integration rules

This is the target integration design, not a claim of an existing Databridle adapter. Components communicate through versioned contracts and keep independent storage. No shared database, forced cloud account or mandatory all-product installation.

The host proposes work; Databridle decides protected enterprise actions; the credential provider enforces its own access conditions; the executor performs only the bound action. Local restrictions can deny but cannot widen an enterprise grant. Human software acceptance and credential approval remain distinct from exact-action security approval.

Carry tenant/principal/delegation/task/run/action identifiers, policy revision, exact request digest, expiry and decision/receipt references through authenticated adapters. Treat client-supplied identity and trace fields as untrusted until bound by the trusted host. Keep secrets and raw sensitive payloads out of default logs, prompts and project memory. Distinguish allow/deny from executed/failed/unknown and from independent verification.

Protected mode stops on missing authority or unavailable required controls; it cannot silently invoke an unprotected path. Retries need idempotency or reconciliation. Revocation of future access does not undo completed work or erase credentials already delivered. Record actual transport, version and bypass coverage before describing an integration as supported.

## Investment and success

Prioritize a supported, reusable protected workflow over feature breadth. Measure integration effort, correctly completed work, denied unauthorized operations, evidence completeness and ongoing maintenance. This component's role does not create a new license, transfer IP or approve a pricing/partnership claim.
