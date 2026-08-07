# GitHub Actions Setup

Most workflows require no repository secrets. Review workflow permissions and branch protection before enabling automation that writes to the repository.

## Dependency update workflow

`.github/workflows/dependency-update.yml` runs daily at 02:00 UTC and supports manual `compatible`, `latest`, and `security` modes. It updates dependencies, runs tests/checks when the lockfile changes, and commits selected files directly to the triggering branch with a skip-CI marker.

That direct-write model is convenient but bypasses normal review. The recommended production setup is branch protection plus a pull-request-based dependency tool or enabling and validating the workflow's optional PR job before use.

## Optional PAT

The workflow falls back to `GITHUB_TOKEN`. Define `PAT_TOKEN` only if the desired automation cannot be achieved with scoped workflow permissions. Prefer a fine-grained token limited to this repository and the minimum contents/pull-request permissions; avoid classic `repo` scope when it is unnecessary. Store it under repository Actions secrets and rotate it according to organizational policy.

Never print a token in workflow logs or use a personal token where a GitHub App or `GITHUB_TOKEN` can provide narrower, auditable access.

## Manual run

In GitHub, open Actions → Nightly Dependency Update → Run workflow and choose an update type. Review the resulting commit and workflow logs. The current workflow reports failure in the run summary; it does not create a GitHub issue automatically.

## Changing behavior

- Edit the cron under `on.schedule` to change frequency.
- To adopt PR mode, disable the direct commit step and enable/test the `create-pr` job as one coherent change.
- Remove the skip-CI marker only after checking that workflow concurrency cannot create an update loop.
- Keep third-party actions pinned to reviewed versions and review their permissions.

Validate workflow changes with syntax tooling and, where practical, Act. GitHub-hosted behavior remains authoritative.

## Operational checklist

- Require reviews/status checks on protected branches.
- Grant the workflow only the permissions it uses.
- Confirm Dependabot/security alerts and failed-run notifications reach maintainers.
- Review automated updates rather than assuming successful tests eliminate supply-chain risk.
- Revoke unused tokens and audit workflow changes like application code.
