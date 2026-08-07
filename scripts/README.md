# Repository Scripts

Run scripts from the repository root. Read the script header before use; several utilities change GitHub metadata or delete generated files and are not part of routine development.

## Maintained entry points

| Path | Purpose |
|------|---------|
| `ci/run_ci_local.sh` | Run CI through Act when available, otherwise run native checks |
| `ci/generate-coverage-report.sh` | Generate LCOV and a Markdown coverage report |
| `ci/simple-coverage.sh` | Produce the fallback coverage summary used by CI |
| `ci/run-benchmarks.sh` | Run the maintained Criterion suite and write a benchmark snapshot |
| `ci/pre-push` | Optional Git pre-push hook |
| `dev/setup-dev.sh` | Install/configure development prerequisites |
| `dev/verify-environment.sh` | Check the local development environment |
| `docs/check-docs-quality.py` | Check maintained Markdown for duplicate content and broken local links |
| `install-pre-commit.sh` | Install repository Git hooks |
| `security-audit.sh` | Run dependency/supply-chain audit helpers |
| `update-dependencies.sh` | Update Rust dependencies with repository checks |
| `act-clean.sh` | Clean Act containers/resources |

The scripts under `test/` exercise the large examples collection. The scripts under `utils/` are repository-maintenance helpers for badges, labels, publication, and cleanup; inspect their arguments and targets before running them.

## Common commands

```bash
./scripts/dev/verify-environment.sh
./scripts/ci/run_ci_local.sh
python3 scripts/docs/check-docs-quality.py
./scripts/ci/run-benchmarks.sh
```

Install the optional pre-push hook:

```bash
cp scripts/ci/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

Contributor requirements and direct Cargo commands are canonical in [CONTRIBUTING.md](../CONTRIBUTING.md). CI-specific inputs and outputs are documented in [ci/README.md](ci/README.md).

## Adding or changing a script

- use strict error handling and quote paths/variables;
- work from the repository root or resolve it safely;
- avoid hidden network or destructive behavior;
- make destructive targets explicit and recoverable where possible;
- keep generated output deterministic or clearly timestamped;
- update this index and the relevant workflow; and
- test locally on the supported shell/platforms.
