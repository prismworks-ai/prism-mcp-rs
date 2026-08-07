# Performance

Performance is workload- and deployment-specific. The project does not publish universal throughput, memory, or latency guarantees.

## Maintained benchmarks

Run the Criterion suite with the features required by its targets:

```bash
cargo bench --features bench,plugin,http --bench all_benchmarks
```

It currently exercises selected plugin/registry operations, common server request dispatch, and a recoverable endpoint-failover path. Criterion writes detailed output below `target/criterion/`. The checked-in [benchmark report](../../reports/benchmark-report.md) records one dated development-machine run and is not a production baseline.

## Establish a useful baseline

Record:

- commit, Rust version, target triple, build profile, and enabled features;
- CPU model, memory, OS/kernel, container limits, and allocator;
- transport, connection reuse, TLS, payload distribution, and concurrency;
- handler/downstream behavior and injected failure conditions; and
- p50/p95/p99 latency, throughput, error rate, allocations, and resident memory.

Warm up the system, use enough samples, isolate background load, and compare distributions or confidence intervals. A single median from a developer laptop is only a smoke signal.

## Optimization order

1. Profile end-to-end and identify the dominant component.
2. Remove blocking work from Tokio executor threads.
3. Reuse HTTP connections and clients; avoid rebuilding transports per request.
4. Bound downstream calls with deadlines and concurrency limits.
5. Reduce payload size or enable compression only when measured network savings exceed CPU cost.
6. Reduce unnecessary cloning/serialization in hot handlers.
7. Tune Tokio worker counts, allocator, CPU requests/limits, and optional affinity in the host after application-level fixes.

CPU affinity is intentionally not an SDK default. Pinning can fight an orchestrator or worsen imbalance; keep it only when repeatable target-hardware measurements justify it.

## Load and failure testing

Criterion isolates code paths but does not model a production network. Add a separate load test for:

- representative payloads and handler work;
- cold and reused TLS connections;
- steady state, bursts, and overload;
- rate-limit and authorization paths;
- slow dependencies, endpoint loss, and circuit cooldown; and
- graceful shutdown during in-flight requests.

For mutations, do not benchmark replay unless the backend deduplicates the supplied idempotency key.

## Regression policy

Use a stable runner for automated comparisons. Fail CI only on a threshold supported by observed variance, and retain raw results. Investigate changes in compiler, dependencies, feature flags, or machine class before attributing a regression to application code.

Performance documentation should always name the environment and date. Never translate a microbenchmark into an SLA without an end-to-end capacity test.
