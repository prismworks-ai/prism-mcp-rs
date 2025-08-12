# Scripts Directory

> **Organized automation scripts for the Prism MCP SDK**

This directory contains all automation and utility scripts for the project, organized by purpose.

## Directory Structure

```
scripts/
├── ci/ # Continuous Integration scripts
├── docs/ # Documentation generation and management
├── dev/ # Development environment setup
├── utils/ # Utility scripts for various tasks
└── doc-templates/ # Documentation templates
```

## Categories

## CI Scripts (`ci/`)

Scripts for continuous integration and testing:

- `local-ci.sh` - Run CI pipeline locally (mirrors GitHub Actions)
- `ci-check.sh` - Quick CI validation checks
- `integration_test.sh` - Run integration tests
- `validate-workflows.sh` - Validate GitHub workflow files
- `pre-push` - Git pre-push hook for automatic CI

**Usage:**
```bash
# Run full CI locally./scripts/ci/local-ci.sh

# Quick CI check./scripts/ci/ci-check.sh

# Install pre-push hook
cp scripts/ci/pre-push.git/hooks/
```

## Documentation Scripts (`docs/`)

Improved documentation generation with v2 headers:

- `generate-docs-v2.sh` - Generate docs with improved headers
- `add-doc-headers-v2.py` - Add v2 headers with 2-click issue reporting
- `check-docs-quality.py` - Check documentation quality
- `rustdoc-to-markdown.py` - Convert Rust docs to Markdown
- `update-docs-index.py` - Update documentation index
- `restructure-docs.py` - Restructure documentation files
- `fix-final-docs.py` - Final documentation fixes

**Features of v2 Documentation System:**
- [x] Clear manual/auto-generated labels
- [x] 2-click issue reporting (click button → describe → submit)
- [x] clean badge-based UI
- [x] Metadata tracking for changes
- [x] Pre-filled GitHub issue templates

**Usage:**
```bash
# Generate docs with v2 headers./scripts/docs/generate-docs-v2.sh

# Update headers only
python3 scripts/docs/add-doc-headers-v2.py

# Or use Makefile
make docs
make docs-headers # Headers only
```

## Development Scripts (`dev/`)

Development environment setup and verification:

- `setup-dev.sh` - Set up development environment
- `verify-environment.sh` - Verify development environment

**Usage:**
```bash
# Set up development environment./scripts/dev/setup-dev.sh

# Verify environment./scripts/dev/verify-environment.sh
```

## Utility Scripts (`utils/`)

Various utility scripts for project maintenance:

- `cleanup-root.sh` - Clean up root folder (follows Rust SDK best practices)
- `badge-manager.sh` - Manage README badges
- `diagnose-badges.sh` - Diagnose badge issues
- `fix-badges.sh` - Fix broken badges
- `refresh-badges.sh` - Refresh all badges
- `create-labels.sh` - Create GitHub issue labels
- `verify-publication.sh` - Verify crate publication

**Usage:**
```bash
# Clean up project root./scripts/utils/cleanup-root.sh

# Manage badges./scripts/utils/badge-manager.sh

# Create GitHub labels./scripts/utils/create-labels.sh
```

## Documentation Templates (`doc-templates/`)

Templates for documentation headers:

- `header-manual.md` - Template for manually written docs
- `header-generated.md` - Template for auto-generated docs

## Best Practices

### Script Organization

1. **Keep scripts focused** - Each script should do one thing well
2. **Use appropriate categories** - Place scripts in the correct subdirectory
3. **Document scripts** - Include usage comments at the top of each script
4. **Make scripts executable** - Use `chmod +x script.sh` for shell scripts

### Documentation Generation Workflow

1. **Write/update source code** with proper documentation comments
2. **Run `make docs`** to generate documentation with v2 headers
3. **Check quality** with `make docs-check`
4. **Commit changes** including updated documentation

### 2-Click Issue Reporting System

All documentation now includes a 2-click issue reporting system:

1. **Click** the "Report Issue" button in any documentation file
2. **Describe** the issue in the pre-filled GitHub issue form
3. **Submit** - that's it!

The system automatically:
- Pre-fills issue title with document name
- Adds appropriate labels (documentation, good first issue)
- Includes document metadata (path, type, URL)
- Provides a structured template for the issue description

## Integration with Build System

The documentation scripts are integrated with the build system:

- **build.rs** - Calls documentation scripts during release builds
- **Makefile** - Provides convenient targets for documentation tasks
- **GitHub Actions** - Runs documentation generation in CI/CD

## Environment Requirements

- **Python 3.6+** - For documentation scripts
- **Rust toolchain** - For cargo doc generation
- **Bash** - For shell scripts
- **Git** - For hooks and version control

## Troubleshooting

### Permission Denied
```bash
# Make scripts executable
chmod +x scripts/**/*.sh
chmod +x scripts/**/*.py
```

### Python Module Not Found
```bash
# Install required Python packages
pip3 install --user -r scripts/requirements.txt
```

### Documentation Not Generating
```bash
# Check script permissions
ls -la scripts/docs/

# Run with verbose output
bash -x scripts/docs/generate-docs-v2.sh

# Check Python version
python3 --version # Should be 3.6+
```

## Contributing

When adding new scripts:

1. Place in appropriate category directory
2. Add documentation at the top of the script
3. Update this README with usage information
4. Make script executable: `chmod +x script.sh`
5. Test completely before committing

## License

All scripts in this directory are part of the Prism MCP SDK and are licensed under the same terms as the main project.