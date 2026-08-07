# Benchmark Utilities

`utilities/benchmarks/` contains standalone Criterion programs used for exploratory client, server, and plugin measurements. They are separate from the maintained Cargo benchmark target in `benches/all_benchmarks.rs`.

The canonical suite for CI and regression checks is:

```bash
cargo bench --features bench,plugin,http --bench all_benchmarks
```

Run standalone utilities only when their Cargo targets are configured for the current checkout. Their source may contain experimental or simulated paths and should not be reported as end-to-end SDK performance.

For benchmark methodology, provenance requirements, and interpretation, see [Performance](../docs/guides/performance.md). Generated results live below `target/criterion/`; a dated Markdown snapshot can be generated with `scripts/ci/run-benchmarks.sh`.
