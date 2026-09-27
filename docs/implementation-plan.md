# Prism MCP Rust SDK implementation plan

Effective: 2026-09-24. Status: planned integration work, not a completed release. Governed by [product strategy](product-strategy.md). Owner: component maintainer; cross-product authorization changes also require review by the Databridle security owner. These are accountable roles, not assumed hires.

## Baseline

Source and manifest review only in this pass; SDK tests were not rerun. Downstream tools resolution was tested and fails against the local SDK because their manifests request ^0.1.0. Existing code and user changes are preserved. New checks below are required acceptance work, not claims that those checks already pass.

## Ordered backlog

| ID | Priority | Change | Deliverable | Dependency | Acceptance evidence |
| --- | --- | --- | --- | --- | --- |
| MCP-01 | P0 | Modify | Define generic extension points and narrow protected example scope. | G1; existing src/security.rs hooks. | No enterprise product dependency in default SDK; context authenticated at transport/host boundary. |
| MCP-02 | P1 | Add | Implement optional policy adapter and evidence hooks. | Databridle G2. | No execution after denial, timeout or mismatched request; stable action/attempt IDs; secrets excluded. |
| MCP-03 | P1 | Modify | Validate supported transport/protocol matrix and downstream examples. | MCP Tools manifest/API migration. | Fresh builds and conformance at pinned versions; protected examples never use permissive default policy. |
| MCP-04 | P2 | Defer | Plugin marketplace, new transport variants and broad platform features. | A supported customer workload requires them. | No claim of plugin isolation without an actual process/security boundary. |

P0 establishes correctness and scope; P1 completes a supported integration; P2 is conditional expansion. Work may run concurrently once interface dependencies are agreed. No calendar duration or production availability is implied.

## Contract acceptance

- Pin supported component, profile and transport versions; reject unsupported security semantics.
- Verify tenant/principal/resource binding, changed arguments, expiry, replay, revocation, retries and missing context on every advertised protected path.
- Keep authorization verdict, execution result and independent verification distinct; interrupted or unobserved effects remain unknown.
- Keep credentials out of prompts/logs/general memory; test redaction and controlled evidence access.
- Demonstrate independent use and supported replacement components; fail closed in protected mode when required controls are unavailable.

## Migration and removal

Before executable code removal, inventory consumers and stored data; add replacement/version migration and compatibility tests; communicate deprecation; ship export/restore and rollback instructions. Do not delete historical evidence, encrypted secrets or user data as documentation cleanup. No published package/API identifier or license changes without a separate compatibility and ownership decision.

## Release definition

Record exact commit/version, supported paths, test commands/results, known limits, installer/upgrade steps and accountable maintainer. Mock tests and schema checks are not production security evidence. Update README and compatibility documentation from the passing report, not from planned checkboxes. No universal integrated/production-ready claim until the relevant execution path passes the complete acceptance matrix.

## Dependency gate reference

G1 freezes validated integration contracts and fixtures. G2 implements Databridle’s protected action/approval/receipt path. G3 connects supervision. G4 validates RabbitLock credential delivery. G5 proves interoperability and replacement hosts/providers. G6 packages a supported release. References to these gates are dependencies, not claims of completion; standalone open-source use remains independent.
