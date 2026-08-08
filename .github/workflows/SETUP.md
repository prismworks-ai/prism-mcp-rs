# GitHub automation

Repository automation is review-first: dependency changes arrive as pull requests, while security workflows have read-only repository access.

## Dependency updates

`.github/dependabot.yml` checks Rust dependencies daily at 03:00 UTC and GitHub Actions weekly on Monday at 04:00 UTC. Compatible Rust minor and patch releases are grouped to reduce pull-request noise; major releases remain separate for focused review.

Dependabot updates version requirements when a change is required. This crate intentionally does not commit `Cargo.lock`, because downstream library consumers resolve their own compatible dependency graph. Maintainers must review Dependabot pull requests and require the normal CI and Security checks before merging.

GitHub creates security-update pull requests independently of the version-update schedule when Dependabot security updates are enabled in the repository settings. Enable Dependabot alerts, security updates, and failed-run notifications for maintainers.

## Security workflow

`.github/workflows/security.yml` is the single dependency-security workflow:

- daily and dependency-related changes: resolve the current compatible graph, fail on cargo-audit vulnerabilities or warnings, and enforce cargo-deny bans, licenses, and sources;
- weekly and relevant pull requests: run cargo-vet as an advisory supply-chain review until the attestation baseline is complete;
- pull requests: run GitHub's dependency review and reject newly introduced vulnerabilities of moderate severity or higher.

Every audit run uploads the resolved `Cargo.lock` and JSON report for 30 days. Weekly supply-chain runs upload their resolved lockfile and cargo-vet report. These artifacts make a scheduled result reproducible without imposing a repository lockfile on library consumers.

The workflow deliberately has no SARIF job: cargo-audit does not emit SARIF natively, and uploading an empty conversion creates a misleading Security tab. Audit failures remain visible as required checks, workflow summaries, and retained JSON artifacts.

## Manual run

Open **Actions → Security → Run workflow**. The optional `run_supply_chain` input controls whether the advisory cargo-vet job runs; vulnerability and dependency-policy checks always run.

## Repository settings

- Protect `main` and require CI plus the blocking Security jobs.
- Enable Dependabot alerts and Dependabot security updates.
- Allow GitHub Actions read access by default; grant elevated permissions only to a specific workflow that needs them.
- Review third-party action updates like application-code changes.
- Remove `PAT_TOKEN` if it was used only by the retired direct-write updater.
