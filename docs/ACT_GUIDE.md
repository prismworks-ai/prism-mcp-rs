# Act - Local GitHub Actions Runner Guide

Act allows you to run GitHub Actions workflows locally, ensuring your changes pass CI before pushing to GitHub.

## Installation

### macOS
```bash
brew install act
```

### Linux
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

### Windows
```powershell
choco install act-cli
# or
scoop install act
# or
winget install nektos.act
```

## Quick Start

```bash
# List available workflows and jobs
act -l

# Run the default push event workflows
act push

# Run pull request workflows
act pull_request
```

## Common Commands

### Running Specific Jobs

```bash
# Run only the test job
act -j test

# Run only the clippy linting job
act -j clippy

# Run coverage generation
act -j coverage

# Run security audit
act -j security
```

### Running Specific Workflows

```bash
# Run a specific workflow file
act -W .github/workflows/ci.yml
act -W .github/workflows/security.yml
act -W .github/workflows/benchmarks.yml
```

### Debugging

```bash
# Verbose output to see what's happening
act -v push

# Dry run - see what would execute without running
act -n push

# Run with specific runner image
act -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

### Event Simulation

```bash
# Simulate different GitHub events
act push              # Push event
act pull_request      # Pull request event
act workflow_dispatch # Manual workflow trigger
act schedule         # Scheduled event
```

## Configuration

The project includes an `.actrc` file with optimized settings:
- Container reuse for faster subsequent runs
- Proper Docker network configuration
- macOS-specific optimizations

## Workflows Available

This project includes the following GitHub Actions workflows:

| Workflow | File | Description | Trigger |
|----------|------|-------------|----------|
| CI | `ci.yml` | Main CI pipeline (test, lint, coverage) | Push, PR |
| Security | `security.yml` | Security audits and checks | Push, Schedule |
| Benchmarks | `benchmarks.yml` | Performance benchmarks | Push to main |
| Dependencies | `dependencies.yml` | Dependency management | Schedule |
| Release | `release.yml` | Release preparation and publishing | Tag push |

## Tips and Best Practices

1. **First Run**: The first run will download Docker images, which may take a few minutes.

2. **Container Reuse**: The `.actrc` configuration enables container reuse for better performance.

3. **Secrets**: For workflows requiring secrets, create a `.secrets` file (gitignored):
   ```bash
   echo "MY_SECRET=value" > .secrets
   act --secret-file .secrets
   ```

4. **Architecture**: On Apple Silicon Macs, you might need to specify architecture:
   ```bash
   act --container-architecture linux/amd64
   ```

5. **Resource Limits**: For resource-intensive workflows, you can adjust Docker settings in Docker Desktop or OrbStack.

## Comparison with GitHub Actions

| Feature | GitHub Actions | Act |
|---------|---------------|-----|
| Cost | Uses GitHub minutes | Free (local resources) |
| Speed | Network latency | Local execution |
| Secrets | Repository/org secrets | Local `.secrets` file |
| Artifacts | GitHub storage | Local filesystem |
| Matrix builds | Full support | Full support |
| Services | Full support | Docker compose |
| Actions | GitHub Marketplace | Most work, some limitations |

## Troubleshooting

### "Error: Cannot connect to Docker"
- Ensure Docker Desktop or OrbStack is running
- Check Docker permissions: `docker ps`

### "Workflow not found"
- Ensure you're in the project root directory
- Check workflow files exist: `ls -la .github/workflows/`

### "Container architecture mismatch"
- Add `--container-architecture linux/amd64` for M1/M2 Macs
- Or use native ARM images when available

### Slow Performance
- Enable container reuse in `.actrc` (already configured)
- Increase Docker resources in Docker Desktop settings
- Use OrbStack for better macOS performance

## Learn More

- [Act Documentation](https://github.com/nektos/act)
- [Act Runner Images](https://github.com/catthehacker/docker_images)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
