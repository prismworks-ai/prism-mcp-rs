# CI Scripts

This directory contains continuous integration scripts used by GitHub Actions.

## Scripts

### generate-coverage-report.sh

Generates code coverage reports using `cargo-llvm-cov` and prepares data for Codecov upload.

**Features:**
- Generates LCOV format coverage data (`lcov.info`) for Codecov
- Creates markdown summary report (`reports/coverage-report.md`)
- Calculates overall and per-module coverage percentages
- Generates HTML coverage report (optional)

**Usage:**
```bash
./scripts/ci/generate-coverage-report.sh
```

**Output:**
- `lcov.info` - Coverage data in LCOV format (for Codecov)
- `reports/coverage-report.md` - Markdown summary report
- `target/llvm-cov/html/` - HTML coverage report (optional)

### simple-coverage.sh

Fallback script for basic coverage reporting when the main script fails.

**Usage:**
```bash
./scripts/ci/simple-coverage.sh
```

### run-benchmarks.sh

Runs Cargo benchmarks and generates performance reports.

**Features:**
- Runs benchmarks with Criterion
- Parses benchmark results into markdown report
- Includes system information
- Formats timing data appropriately (ns/µs/ms)

**Usage:**
```bash
./scripts/ci/run-benchmarks.sh
```

**Output:**
- `reports/benchmark-report.md` - Benchmark results in markdown
- `target/criterion/` - Detailed Criterion output

### run_ci_local.sh

Runs CI checks locally before pushing to GitHub.

**Usage:**
```bash
./scripts/ci/run_ci_local.sh
```

### pre-push

Git pre-push hook to run checks before pushing.

**Installation:**
```bash
cp scripts/ci/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

## Coverage Setup

### Prerequisites

1. **Install cargo-llvm-cov:**
   ```bash
   cargo install cargo-llvm-cov
   ```

2. **Install system dependencies (for report generation):**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install bc jq
   
   # macOS
   brew install bc jq
   ```

### Codecov Integration

The coverage workflow automatically uploads coverage data to Codecov:

1. Coverage is generated in LCOV format (`lcov.info`)
2. The `codecov/codecov-action@v3` GitHub Action uploads the data
3. Results appear on the Codecov dashboard
4. PR comments show coverage changes

### Local Coverage Testing

To generate coverage locally:

```bash
# Generate coverage with HTML report
cargo llvm-cov --all-features --workspace --html

# Open HTML report
open target/llvm-cov/html/index.html

# Generate LCOV format for Codecov
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate markdown report
./scripts/ci/generate-coverage-report.sh
```

## Benchmark Setup

### Prerequisites

Benchmarks use Criterion.rs and require:
- The `bench` feature to be enabled
- Benchmark files in `benches/` directory

### Running Benchmarks Locally

```bash
# Run all benchmarks
cargo bench --features bench

# Run specific benchmark
cargo bench --features bench -- <benchmark_name>

# Generate report
./scripts/ci/run-benchmarks.sh
```

## Troubleshooting

### Coverage Issues

1. **cargo-llvm-cov not found:**
   ```bash
   cargo install cargo-llvm-cov
   ```

2. **bc or jq not found:**
   Install system dependencies as shown above.

3. **Coverage appears low:**
   - Check `codecov.yml` for ignored paths
   - Ensure tests are actually running
   - Verify feature flags are enabled

### Benchmark Issues

1. **No benchmarks found:**
   - Check that `benches/` directory exists
   - Verify `bench` feature is defined in Cargo.toml
   - Ensure benchmark targets are configured

2. **Benchmarks fail to compile:**
   ```bash
   cargo build --benches --features bench
   ```

## CI Workflow Integration

These scripts are called by `.github/workflows/ci.yml`:

- **Coverage job:** Uses `generate-coverage-report.sh` with fallback to `simple-coverage.sh`
- **Benchmark job:** Uses `run-benchmarks.sh`
- Both jobs upload artifacts and reports to GitHub
- Coverage data is sent to Codecov for tracking

## Contributing

When modifying CI scripts:

1. Test locally first
2. Ensure scripts are executable (`chmod +x`)
3. Use proper error handling (`set -e`)
4. Add fallbacks for missing dependencies
5. Document any new dependencies
