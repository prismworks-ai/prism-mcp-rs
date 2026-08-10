# Protocol Versions

Prism MCP Rust SDK 3.0 is dual-stack. MCP 2026-07-28 is the preferred native protocol, and MCP 2025-11-25 remains available for deployed clients and servers that use the stateful initialization lifecycle.

## Compatibility matrix

| Behavior | MCP 2026-07-28 | MCP 2025-11-25 |
|----------|----------------|----------------|
| Lifecycle | Stateless `server/discover` | Stateful `initialize` plus initialized notification |
| Request context | Revision, client identity, and capabilities in each request `_meta` | Negotiated once during initialization |
| Successful results | Required `resultType`; server identity in result `_meta` | Existing 2025 result shapes |
| Cacheable operations | Explicit `ttlMs` and `cacheScope`; Prism defaults to private/no-cache | No 2026 cache fields added |
| HTTP routing | `MCP-Protocol-Version`, `Mcp-Method`, and applicable `Mcp-Name` headers validated against the body | Existing HTTP behavior retained |
| Tool parameter headers | `x-mcp-header` schemas generate and validate `Mcp-Param-*` headers | Annotation is ignored for wire compatibility |
| Multi-round-trip requests | Client automatically fulfills and retries `input_required`, with a configurable round bound | Not applicable |
| Notification subscriptions | `subscriptions/listen`; request-scoped SSE on HTTP and cancellable long-lived requests on STDIO | Legacy transport notification behavior retained |
| Resource updates | URI filters in `subscriptions/listen`; replaces `resources/subscribe` | `resources/subscribe` and `resources/unsubscribe` retained |
| Durable tasks | Opt-in `io.modelcontextprotocol/tasks` extension | Not advertised by the 2025 compatibility surface |
| Removed stateful methods | Rejected with `Method not found` | Retained |

Prism exposes `InputRequiredResult` and `OperationResult<T>` for typed modern result processing. The client continuation loop preserves opaque `requestState`, issues a fresh JSON-RPC ID on each retry, and defaults to at most ten rounds. Servers register continuation-aware tools with `add_multi_round_tool` or `add_multi_round_tool_detailed`; the handler receives input responses, opaque request state, client identity, and request-scoped capabilities. Before emitting `input_required`, the server validates that every input method is supported and that the client declared the required elicitation, sampling, or roots capability.

## Protocol modes

`ProtocolMode::Auto` is the best choice for most applications:

- clients attempt 2026 discovery first;
- fallback occurs only after JSON-RPC `-32601` (`Method not found`);
- a mutually supported 2026 revision is retried once after `UnsupportedProtocolVersion`;
- validation, authentication, HTTP, timeout, and other failures are returned without downgrade;
- STDIO probes with a disposable child process and starts a clean legacy process if fallback is required; and
- servers accept either supported request shape.

Use `ProtocolMode::ModernOnly` when every peer is known to support 2026 and downgrade must be impossible. Use `ProtocolMode::LegacyOnly` for a deliberately pinned 2025 deployment.

```rust,no_run
use prism_mcp_rs::prelude::*;

let mut client = McpClient::new("example-client".into(), "3.0.0".into());
client.set_protocol_mode(ProtocolMode::Auto);

let server = ServerBuilder::new()
    .name("example-server")
    .version("3.0.0")
    .protocol_mode(ProtocolMode::Auto)
    .build();
```

After connecting, inspect `ConnectResult.protocol` or `McpClient::negotiated_protocol()` instead of inferring the revision from server capabilities.

## Capability discipline

The 2026 server advertises catalog change and resource subscription filters because HTTP and STDIO implement `subscriptions/listen`. Filters are strictly opt-in, the acknowledgement is always the first stream message, and every delivered notification carries the originating subscription ID. Resource URI filters replace the removed modern `resources/subscribe` RPC.

The server advertises `io.modelcontextprotocol/tasks` only when at least one task tool is registered. Task methods and task-status subscriptions require the client to declare the same extension on the request. The runtime provides strong task creation consistency, unguessable IDs, caller binding, TTL enforcement, cooperative cancellation, polling hints, status notifications, and partial multi-round input delivery. Use `add_task_tool_with_fallback` when the same tool can run synchronously for clients that do not opt in; omit a fallback when Tasks is required. Use `add_composed_task_tool` when a tool must first gather request-scoped MRTR input and then escalate its final round into a durable task.

The 2025 capability response remains unchanged for interoperability. Application-specific extension capabilities can be carried in the `extensions` maps.

## HTTP header integrity

For modern HTTP requests, standard routing headers must match the JSON-RPC body. A mismatch returns HTTP 400 and MCP error `-32020`. Tool input schemas may annotate nested string, integer, or boolean properties with `x-mcp-header`. Prism validates header tokens, rejects case-insensitive duplicates, encodes unsafe values with the protocol base64 sentinel, excludes invalid discovered tools on clients, and validates received parameter headers before dispatch on servers.

Tool schemas are installed into the HTTP transport when the server starts. Register HTTP tools before startup so their parameter-header rules are enforced.

The standards-track 2026 HTTP implementation is `HttpClientTransport`/`HttpServerTransport`, and it is always selected by the recommended transport helper. Ordinary POST results may arrive as JSON or as a finite SSE stream; the client accepts both standard response forms. The optional advanced chunked/compressed transport uses Prism-specific endpoints, is restricted to `ProtocolMode::LegacyOnly`, and is not part of the MCP conformance surface. Applications must select it explicitly against a deployment that implements the same proprietary endpoints.

## Verification boundary

The repository tests modern discovery, exact legacy initialization, safe fallback, per-request metadata, result envelopes, cache fields, HTTP routing integrity, subscriptions, Tasks, schema-declared parameter headers, removed methods, and bounded multi-round client retries. A pinned upstream referee runs `server-stateless` plus selected client metadata, tool, header, and JSON Schema scenarios in CI with no failure baseline. The upstream Tasks semantic scenarios also exercise the local diagnostic adapter; their status-notification case is currently skipped by the upstream harness pending its `subscriptions/listen` rewrite, and its core wire-schema checker does not yet validate extension `CreateTaskResult` messages. This is an implementation compatibility statement, not certification by the MCP project; test every production peer and preserve captured wire fixtures for upgrades.
