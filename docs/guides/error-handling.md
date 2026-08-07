# Error Handling

Fallible SDK operations return `McpResult<T>`, an alias for `Result<T, McpError>`. Preserve the most specific error category available so callers, logs, and failover logic can respond correctly.

## Main categories

`McpError` includes transport, connection, protocol, serialization, validation, URI, not-found, authentication, OAuth authorization, forbidden, rate-limited, timeout, cancellation, I/O, URL, internal, and feature-specific HTTP/WebSocket errors.

Use tuple variants as defined by the API:

```rust
use prism_mcp_rs::{McpError, McpResult};

fn require_name(name: Option<&str>) -> McpResult<&str> {
    name.ok_or_else(|| McpError::InvalidParams("missing name".to_string()))
}
```

Convenience constructors such as `McpError::validation`, `connection`, `timeout`, and `internal` are useful when translating an application error.

## Protocol failure versus tool-domain failure

Return `Err(McpError::...)` when the request cannot be processed as an MCP operation: invalid input, unavailable dependency, authorization denial, timeout, or internal failure.

Return a `ToolResult` with `is_error: Some(true)` when the tool ran and produced an expected domain-level failure that the model should see as tool output:

```rust
use prism_mcp_rs::prelude::*;

let result = ToolResult {
    content: vec![ContentBlock::text("invoice is already closed")],
    is_error: Some(true),
    structured_content: None,
    meta: None,
};
```

Do not expose internal paths, queries, stack traces, credentials, or sensitive downstream responses in either error form.

## Retries and failover

`McpError::is_recoverable` is used by endpoint failover. Connection, timeout, I/O, rate-limit, and feature-specific network failures are generally classified as recoverable; validation, policy, protocol, and internal failures are not. Recoverable does not automatically mean safe to replay: the endpoint pool separately checks method idempotency.

When adding an error variant, update its display text, category, recovery classification, transport mapping, and tests together.

## Logging

Log a stable category, request/trace ID, MCP method, endpoint or transport, and elapsed time. Avoid duplicate logging at every propagation layer. Expected denials and throttling should be distinguishable from server faults without leaking policy details to untrusted clients.

## Handler guidance

- Validate required arguments before side effects.
- Add operational context while retaining the original source in application logs.
- Apply explicit downstream timeouts and cancellation.
- Do not retry non-idempotent actions unless the backend implements deduplication.
- Keep client-facing messages stable enough for automation, and use structured fields when callers must branch on details.

## Tests

Cover the success path, each meaningful category, redaction, timeout/cancellation, policy denial, throttling retry metadata, and idempotency behavior. Assert variants with `matches!` rather than comparing display strings unless the display text is itself a contract.
