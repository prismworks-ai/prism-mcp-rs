# Contributing

Thank you for improving `prism-mcp-rs`. Keep changes focused, preserve compatibility unless a breaking release is intended, and describe security or behavior changes explicitly.

## Prerequisites

- Rust 1.85 or newer with `rustfmt` and `clippy`
- Git
- Optional: `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`, and Act

```bash
rustup component add rustfmt clippy
cargo install cargo-audit cargo-deny cargo-llvm-cov
```

Do not install tools you do not need for the change.

## Setup

```bash
git clone https://github.com/prismworks-ai/prism-mcp-rs
cd prism-mcp-rs
cargo check --all-features
cargo test --all-features
```

Use a topic branch. An issue is recommended for significant features or API changes so design and compatibility can be discussed before implementation; trivial fixes do not require ceremony.

## Development standards

- Prefer small, typed APIs and the smallest necessary feature/dependency surface.
- Keep transport-independent behavior in the shared server path.
- Preserve explicit trust boundaries: identity must be verified before creating `RequestContext`; native plugins are not sandboxed.
- Avoid blocking work on Tokio executor threads.
- Add regression tests for fixes and focused unit/integration tests for new behavior.
- Document public APIs and update maintained guides when user-visible behavior changes.
- Do not add latency, throughput, coverage, vulnerability, or compliance claims without a reproducible artifact and date.
- Keep unsafe code confined and justified. Plugin FFI changes require extra review.

## Required checks

Run checks proportional to the change. Before requesting review, the expected full set is:

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo build --examples --all-features
cargo doc --no-deps --all-features
python3 scripts/docs/check-docs-quality.py
```

For dependency, security, or release changes also run:

```bash
cargo audit --deny warnings
cargo deny check
cargo package --allow-dirty
```

For performance-sensitive changes:

```bash
cargo bench --features bench,plugin,http --bench all_benchmarks
```

Record hardware, toolchain, feature set, benchmark name, and before/after statistics. Do not optimize against one noisy sample.

The Makefile and scripts are conveniences; direct Cargo commands above are the source of truth. `./scripts/ci/run_ci_local.sh` can run native checks or use Act when installed.

## Tests

- Unit tests live near the code they cover.
- Cross-module and transport behavior belongs in `tests/`.
- Documentation examples should compile where practical; use `no_run` for examples that would block on a server loop and `ignore` only when external infrastructure or secrets are required.
- Tests must not assume common ports are unused or depend on the public internet.
- Security tests should include both allowed and denied behavior.

## Documentation

The canonical documentation index is [docs/README.md](docs/README.md). Contributor workflow belongs here rather than in a second development guide. Script details belong in [scripts/README.md](scripts/README.md). Generated reports and examples must state their provenance and should not be edited into timeless claims.

Use relative links for repository files. After moving or deleting documentation, search for old paths and run the documentation quality checker.

## Commits and pull requests

Use clear imperative commit subjects. Conventional prefixes such as `feat:`, `fix:`, `docs:`, `test:`, and `chore:` are welcome but not mandatory unless repository automation requires them.

A pull request should state:

- the problem and chosen approach;
- user-visible or compatibility impact;
- security and operational implications where relevant;
- the exact verification performed; and
- related issues or follow-up work.

Update `CHANGELOG.md` under `Unreleased` for meaningful user-facing changes. Review feedback should be addressed with new commits until the branch is ready; maintainers may squash at merge.

## Reporting security issues

Do not disclose a vulnerability in a public issue or pull request. Follow [SECURITY.md](SECURITY.md).

By contributing, you agree that your contribution is licensed under the repository's MIT license.
