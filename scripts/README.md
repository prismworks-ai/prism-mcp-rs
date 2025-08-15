# Scripts Directory

Organized automation and utility scripts for the prism-mcp-rs project.

## Directory Structure

```
scripts/
├── ci/                 # CI/CD related scripts
│   └── pre-push                  # Git hook for pre-push validation
└── README.md          # This file
```

## Usage

All CI scripts can be accessed through the main `./ci` command in the project root:

```bash
# From project root
./ci run        # Start CI
./ci logs       # View logs
./ci monitor    # Monitor containers
./ci stop       # Clean up
./ci help       # Show all commands
```

Or run scripts directly:

```bash
# From project root
./scripts/ci/run-ci-with-network.sh
./scripts/ci/view-ci-logs.sh -f  # Follow logs
```

## Adding New Scripts

When adding new scripts:

1. Place them in the appropriate subdirectory
2. Make them executable: `chmod +x script-name.sh`
3. Add documentation in the script header
4. Update this README
5. Consider adding to the main `./ci` command if CI-related

## Script Categories

### CI Scripts (`ci/`)

Container and CI pipeline management:
- **pre-push**: Git hook that validates code before pushing

**Note:** Most CI scripts have been removed in favor of using Act to run GitHub Actions locally.
See DEVELOPMENT.md for instructions on setting up and using Act.

### Future Categories

Planned script categories:
- `build/` - Build and compilation scripts
- `test/` - Testing utilities
- `release/` - Release automation
- `dev/` - Development helpers