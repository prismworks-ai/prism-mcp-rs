# Contributing to Prism MCP SDK

Thank you for your interest in contributing to the Prism MCP SDK. This guide outlines our contribution process using GitHub Issues as the primary communication channel.

## Before you start

### Required reading

1. **[DEVELOPMENT.md](DEVELOPMENT.md)** - Development setup and build process
2. **[README.md](README.md)** - Project overview and architecture
3. **[docs/PLUGIN_GUIDE.md](docs/PLUGIN_GUIDE.md)** - If contributing plugins

### Issue-first approach

**All contributions must start with a GitHub Issue.** This ensures:
- No duplicate work
- Alignment with project goals
- Proper tracking and attribution
- Clear communication

## How to contribute

### 1. Reporting bugs

**Use Issue Label:** `bug`

Before reporting:
1. Search [existing issues](https://github.com/prismworks-ai/prism-mcp-rs/issues?q=is%3Aissue+label%3Abug)
2. Verify the bug exists in the latest version
3. Collect reproduction information

Create an issue with:
```markdown
**Description:**
Clear description of the bug

**Steps to reproduce:**
1. Step one
2. Step two
3. Step three

**Expected behavior:**
What should happen

**Actual behavior:**
What actually happens

**Environment:**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.85.0]
- SDK version: [e.g., 0.1.0]

**Code sample:**
```rust
// Minimal reproduction code
```
```

### 2. Requesting features

**Use Issue Label:** `feature-request`

Before requesting:
1. Search [existing feature requests](https://github.com/prismworks-ai/prism-mcp-rs/issues?q=is%3Aissue+label%3Afeature-request)
2. Check the [roadmap](https://github.com/prismworks-ai/prism-mcp-rs/projects)
3. Consider if it aligns with project goals

Create an issue with:
```markdown
**Problem:**
What problem does this solve?

**Proposed solution:**
How would this work?

**Alternatives considered:**
What other approaches exist?

**Use cases:**
Who benefits and how?

**Implementation notes:**
Any technical considerations?
```

**Note:** For major features, consider discussing your approach in the issue comments before implementing.

### 3. Fixing documentation

**Use Issue Label:** `documentation`

For documentation fixes:
1. Small fixes (typos, grammar): Create issue and PR together
2. Large changes: Discuss in issue first

Create an issue with:
```markdown
**Location:**
File path and line numbers

**Current text:**
What it says now

**Proposed text:**
What it should say

**Reason:**
Why this change improves documentation
```

### 4. Asking questions

**Use Issue Label:** `question`

For questions about:
- How to use the SDK
- Architecture decisions
- Implementation details
- Best practices

Create an issue with:
```markdown
**Context:**
What are you trying to do?

**Question:**
Specific question

**What I've tried:**
Research/attempts made

**Related docs:**
Links to relevant documentation
```

### 5. Making suggestions

**Use Issue Label:** `enhancement`

For improvements that aren't new features:
- Performance optimizations
- Code refactoring
- Build process improvements
- Testing enhancements

Create an issue with:
```markdown
**Current situation:**
How it works now

**Suggested improvement:**
How it could be better

**Benefits:**
Why this matters

**Trade-offs:**
Any downsides?
```

## Issue labels

We use these labels to categorize issues:

| Label | Purpose |
|-------|---------|
| `bug` | Something isn't working |
| `documentation` | Documentation improvements |
| `feature-request` | New feature proposal |
| `enhancement` | Improvement to existing code |
| `question` | Questions about the project |
| `good-first-issue` | Good for newcomers |
| `help-wanted` | Extra attention needed |
| `blocked` | Waiting on something else |
| `wontfix` | Will not be worked on |
| `duplicate` | This issue already exists |

## Pull request process

### Prerequisites

1. **Issue exists:** Every PR must reference an issue
2. **Issue is unassigned:** Check that nobody else is working on it
3. **Development setup:** Complete setup from [DEVELOPMENT.md](DEVELOPMENT.md)

### Step-by-step process

#### 1. Self-assign the issue

If an issue is unassigned and you want to work on it:
- Self-assign it to yourself (if you have permissions)
- Or comment "I'm working on this" to claim it
- For complex features, consider posting your approach as a comment for feedback

#### 2. Fork and branch

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/prism-mcp-rs
cd prism-mcp-rs
git remote add upstream https://github.com/prismworks-ai/prism-mcp-rs
git checkout -b issue-123-description
```

#### 3. Make changes

Follow the development workflow:
```bash
# Regular development cycle
make quick          # Quick checks during development
make test           # Run tests
make docs           # Update documentation if needed

# Before committing
make commit-ready   # Full validation
```

#### 4. Commit your changes

Use conventional commits:
```bash
# Format: type(scope): description
git commit -m "fix(transport): resolve websocket connection timeout"
git commit -m "feat(plugin): add hot reload support"
git commit -m "docs(readme): clarify installation steps"
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance tasks

#### 5. Create pull request

```bash
git push origin issue-123-description
```

PR title format: `type(scope): description (#issue-number)`

PR description template:
```markdown
## Summary
Brief description of changes

## Related Issue
Fixes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Testing
- [ ] Tests pass locally (`make test-all`)
- [ ] Added new tests for changes
- [ ] Coverage maintained/improved

## Checklist
- [ ] Code follows project style (`make fmt`)
- [ ] Clippy passes (`make clippy`)
- [ ] Documentation updated
- [ ] CHANGELOG.md entry added (if applicable)
```

#### 6. Address review feedback

- Respond to all comments
- Push additional commits (don't force-push during review)
- Request re-review when ready

#### 7. Final steps

Once approved:
- Squash commits if requested
- Ensure CI passes
- Maintainer will merge

## CI/CD and reporting

### Automatic reports

Our CI pipeline automatically generates:
- **Coverage reports** (`reports/coverage-report.md`) - Code coverage metrics
- **Benchmark reports** (`reports/benchmark-report.md`) - Performance metrics

These reports are:
- Generated on every PR as downloadable artifacts
- Automatically committed to the repository on main branch pushes
- Viewable directly on GitHub in markdown format

### For contributors

**No tokens or secrets needed!** All CI features work automatically:
- ✅ Testing and validation
- ✅ Coverage report generation
- ✅ Benchmark execution
- ✅ PR status checks

The `GITHUB_TOKEN` used for committing reports is automatically provided by GitHub Actions for every workflow run.

### For maintainers only

**Publishing to crates.io** requires the `CRATES_IO_TOKEN` secret:
- Only repository owners have this token
- Contributors cannot publish releases
- Fork owners need their own token to publish their fork

### Running reports locally

```bash
# Generate both coverage and benchmark reports
./scripts/ci/local-ci-enhanced.sh --reports

# Run full CI with reports
./scripts/ci/local-ci-enhanced.sh --full

# Quick coverage report only
./scripts/ci/simple-coverage.sh
```

Reports will be saved in the `reports/` directory.

## Development guidelines

### Code quality standards

| Requirement | Check command |
|-------------|---------------|
| Formatting | `make fmt` |
| Linting | `make clippy` |
| Tests pass | `make test-all` |
| Coverage >65% | `make coverage` |
| Documentation | `cargo doc` |

### Testing requirements

Every PR must include:
- Unit tests for new functions
- Integration tests for new features
- Updated existing tests if behavior changes
- Test coverage report showing >65% coverage

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_functionality() {
        // Arrange
        let input = TestData::new();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Documentation requirements

Update documentation for:
- New public APIs (inline rustdoc)
- Changed behavior (update existing docs)
- New features (add to relevant guides)
- Breaking changes (update migration guide)

## What we're looking for

### High-priority contributions

- **Bug fixes** with tests
- **Documentation improvements** 
- **Performance optimizations** with benchmarks
- **Test coverage** increases
- **Example code** for complex features

### What benefits from discussion

- **Architecture changes** - Post your design approach in the issue
- **New dependencies** - Explain why they're needed
- **Breaking changes** - Describe migration path
- **Large features** - Share your implementation plan for feedback

### What we won't accept

- Changes without issues
- Features without tests
- Breaking changes without strong justification
- Code using `unsafe` without exceptional reason
- PRs that decrease test coverage

## Recognition

Contributors are recognized through:
- GitHub contributors graph
- Release notes mentions
- CHANGELOG.md credits

## Getting help

If you need help:

1. **Check documentation first:**
   - [DEVELOPMENT.md](DEVELOPMENT.md) for setup
   - [API docs](https://docs.rs/prism-mcp-rs) for usage
   - [Examples](examples/) for patterns

2. **Search existing issues:**
   - Someone may have asked already
   - Look at closed issues too

3. **Create a question issue:**
   - Use the `question` label
   - Be specific about what you need

## Code of conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct):
- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive criticism
- Assume good intentions

Violations can be reported to the maintainers through GitHub Issues.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.