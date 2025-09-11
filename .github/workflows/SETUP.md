# GitHub Actions Setup Guide

## Nightly Dependency Updates

This repository includes an automated dependency update workflow that runs nightly to keep dependencies up-to-date and secure.

### Features

- **Nightly Updates**: Automatically runs at 2 AM UTC (configurable)
- **Security Focus**: Detects and fixes security vulnerabilities
- **Three Update Modes**:
  - `compatible`: Updates to latest compatible versions (default)
  - `latest`: Updates to latest versions, including breaking changes
  - `security`: Only updates packages with known vulnerabilities
- **Automatic Testing**: Runs tests after updates to ensure nothing breaks
- **Direct Commits**: Bypasses PRs for faster updates (configurable)
- **Failure Notifications**: Creates GitHub issues if updates fail

### Setting Up PAT Token (Required for CI Triggering)

To ensure the dependency updates trigger your CI workflows:

1. **Create a Personal Access Token**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Give it a descriptive name (e.g., "Dependency Update Bot")
   - Select scopes:
     - `repo` (full control of private repositories)
     - `workflow` (update GitHub Action workflows)
   - Generate and copy the token

2. **Add Token to Repository Secrets**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `PAT_TOKEN`
   - Value: Paste your personal access token
   - Click "Add secret"

3. **Verify Setup**:
   - The workflow will use `PAT_TOKEN` if available, otherwise fall back to `GITHUB_TOKEN`
   - With PAT_TOKEN, commits will trigger CI workflows
   - Without it, updates will still work but won't trigger other workflows

### Manual Triggering

You can manually trigger the workflow:

1. Go to Actions → "Nightly Dependency Update"
2. Click "Run workflow"
3. Select update type (compatible/latest/security)
4. Click "Run workflow"

### Configuration Options

#### Change Schedule

Edit `.github/workflows/dependency-update.yml`:

```yaml
schedule:
  - cron: '0 2 * * *'  # Daily at 2 AM UTC
  # Examples:
  # - cron: '0 0 * * 1'     # Weekly on Monday
  # - cron: '0 0 1 * *'     # Monthly on the 1st
  # - cron: '0 */6 * * *'   # Every 6 hours
```

#### Enable Pull Requests Instead of Direct Commits

Change `if: false` to `if: true` in the `create-pr` job:

```yaml
create-pr:
  name: Create Pull Request (Optional)
  runs-on: ubuntu-latest
  needs: update-dependencies
  if: true  # Changed from false
```

#### Skip CI on Updates

The workflow includes `[skip ci]` in commit messages by default. Remove this line from the commit message if you want CI to run:

```yaml
commit_message: |
  chore: automated dependency update
  ...
  # Remove the line below to enable CI
  [skip ci] remove this line if you want CI to run
```

### Monitoring

- **Success**: Dependencies are automatically updated and committed
- **Failure**: A GitHub issue is created with details
- **Security**: Warning annotations appear when vulnerabilities are fixed

### Troubleshooting

1. **Workflow not running**: Check Actions are enabled in repository settings
2. **CI not triggered**: Ensure PAT_TOKEN is properly configured
3. **Update failures**: Check the created GitHub issue for details
4. **cargo-vet failures**: The workflow attempts to auto-update audits

### Security Best Practices

- Regularly review automated updates
- Set up branch protection rules for `main`
- Consider using PR mode for production repositories
- Monitor security alerts in the Security tab
- Keep PAT token permissions minimal
- Rotate PAT token periodically

### Benefits

- **Reduced maintenance burden**: No manual dependency updates
- **Improved security**: Automatic vulnerability fixes
- **Better stability**: Regular small updates vs. large breaking changes
- **Time savings**: Automated testing and verification
- **Audit trail**: All updates tracked in git history