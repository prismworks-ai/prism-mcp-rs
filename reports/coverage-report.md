# Coverage Report Snapshot

Generated: 2025-08-20 01:51:13 UTC.

This is a historical snapshot, not the coverage of the current branch. Regenerate from the exact commit and feature set before making a coverage decision.

## Recorded summary

- Line coverage: 66.40%
- Lines found: 19,506
- Lines hit: 12,953

The original module excerpt was incomplete and has been removed to avoid implying comprehensive per-module coverage.

## Regenerate

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
./scripts/ci/generate-coverage-report.sh
```

Local HTML output is written below `target/llvm-cov/html/` when available. Hosted CI uploads LCOV and report artifacts and may send LCOV to Codecov.
