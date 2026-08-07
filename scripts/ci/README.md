# CI Scripts

These scripts support local checks and GitHub Actions. Workflow YAML remains authoritative for hosted CI.

## Local CI

```bash
./scripts/ci/run_ci_local.sh
```

If Act is installed and `USE_ACT` is not `false`, the script runs `.github/workflows/ci.yml` with Act. Otherwise it runs formatting, strict all-feature Clippy, build, tests, docs, examples, and an optional security audit natively.

To force native execution:

```bash
USE_ACT=false ./scripts/ci/run_ci_local.sh
```

Act logs appear in the terminal. For more detail, set `ACTIONS_STEP_DEBUG=true`, run Act with `-v`, or execute the failing Cargo command directly with `RUST_BACKTRACE=1` and an appropriate `RUST_LOG` filter. `scripts/act-clean.sh` removes reusable Act resources when a cached container is unhealthy.

## Coverage

Prerequisites:

```bash
cargo install cargo-llvm-cov
# macOS
brew install bc jq
# Debian/Ubuntu
sudo apt-get install bc jq
```

Generate LCOV plus Markdown/HTML output:

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
./scripts/ci/generate-coverage-report.sh
```

Outputs are `lcov.info`, `reports/coverage-report.md`, and (when supported) `target/llvm-cov/html/`. The hosted workflow uploads report artifacts and LCOV to Codecov. A checked-in report is a dated snapshot, not the current branch's coverage.

## Benchmarks

```bash
./scripts/ci/run-benchmarks.sh
# equivalent suite without regenerating Markdown
cargo bench --features bench,plugin,http --bench all_benchmarks
```

Outputs are `reports/benchmark-report.md` and Criterion data below `target/criterion/`. Report generation fails if no benchmark estimates are produced; it does not substitute fabricated values.

## Pre-push hook

```bash
cp scripts/ci/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

Hooks are local safeguards and do not replace hosted CI.

## Troubleshooting

- Confirm the toolchain with `rustc --version` and `cargo --version`.
- Reproduce a failing workflow command directly before debugging Act.
- Use `cargo tree -e features` for feature mismatches.
- Delete only the relevant generated output or Act resource; avoid broad cleanup commands.
- Check workflow and script feature flags together when benchmarks/examples fail to compile.
