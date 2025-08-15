# Migration Guide

This guide helps you migrate between different versions of the Prism MCP SDK.

## Version Migration Matrix

| From Version | To Version | Breaking Changes | Migration Effort |
|--------------|------------|------------------|------------------|
| - | 0.1.0 | N/A - Initial Release | N/A |
| 0.1.x | 0.2.0 | See v0.2.0 section | Low |

## Migrating to v0.1.0 (Initial Setup)

If you're starting fresh with v0.1.0:

### 1. Add to Cargo.toml

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
```

### 2. Choose Your Features

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["plugin", "websocket", "http"]
}
```

### 3. Import the Prelude

```rust
use prism_mcp_rs::prelude::*;
```

## Future Migrations

### Migrating to v0.2.0 (Planned)

*This section will be updated when v0.2.0 is released.*

#### Expected Changes
- Enhanced plugin API
- Additional transport options
- Performance improvements

#### Migration Steps
1. Update Cargo.toml version
2. Review breaking changes in CHANGELOG.md
3. Update plugin implementations if needed
4. Run tests to verify compatibility

## Plugin Migration

### Plugin API Versioning

Plugins should specify compatible SDK versions:

```rust
impl Plugin for MyPlugin {
    fn min_sdk_version(&self) -> &str {
        "0.1.0"
    }
    
    fn max_sdk_version(&self) -> &str {
        "1.0.0"
    }
}
```

### Handling Breaking Changes

When plugin APIs change:

1. **Check Compatibility**
   ```rust
   if !plugin.is_compatible(SDK_VERSION) {
       return Err(McpError::IncompatiblePlugin);
   }
   ```

2. **Update Implementations**
   - Review trait changes in the documentation
   - Update method signatures
   - Test with new SDK version

3. **Maintain Backwards Compatibility**
   - Use feature flags for version-specific code
   - Provide migration utilities where possible

## Transport Migration

### Switching Transport Types

Transports are designed to be interchangeable:

```rust
// v0.1.0 - STDIO Transport
let transport = StdioTransport::new();
server.run(transport).await?;

// Migration to WebSocket
let transport = WebSocketTransport::bind("127.0.0.1:8080").await?;
server.run(transport).await?;
```

### Feature Flag Changes

If feature flags change between versions:

```toml
# Old (example)
features = ["websocket"]

# New (example)
features = ["websocket-server", "websocket-client"]
```

## API Deprecations

### Handling Deprecated APIs

When APIs are deprecated:

1. **Compiler Warnings**
   ```rust
   #[deprecated(since = "0.2.0", note = "Use new_method instead")]
   fn old_method() { }
   ```

2. **Migration Path**
   ```rust
   // Old API (deprecated)
   server.add_tool_handler(handler);
   
   // New API
   server.add_tool(Tool::new("name", handler));
   ```

3. **Grace Period**
   - Deprecated APIs are maintained for at least one minor version
   - Removal happens in major version updates only

## Testing Your Migration

### Migration Checklist

- [ ] Update Cargo.toml version
- [ ] Review CHANGELOG.md for breaking changes
- [ ] Update feature flags if needed
- [ ] Fix any compilation errors
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Test examples: `cargo build --examples`
- [ ] Verify plugin compatibility
- [ ] Update your documentation

### Automated Migration Testing

```bash
# Before migration
git checkout -b migration-backup
git commit -am "Backup before migration"

# Perform migration
cargo update -p prism-mcp-rs
cargo test --all-features

# If issues arise
git diff migration-backup
```

## Getting Help

### Resources
- [CHANGELOG.md](CHANGELOG.md) - Detailed change descriptions
- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues) - Report migration issues
- [Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions) - Ask questions

### Common Migration Issues

#### Issue: Plugin won't load after upgrade
**Solution**: Check plugin compatibility version and rebuild with new SDK.

#### Issue: Transport feature not found
**Solution**: Review feature flag changes in the release notes.

#### Issue: Trait method signatures changed
**Solution**: Consult the API documentation for the new signatures.

## Version Support Policy

| Version | Support Status | End of Support |
|---------|---------------|----------------|
| 0.1.x | Active | TBD |
| 0.2.x | Planned | TBD |

### LTS Versions

Long-term support versions will be announced after v1.0.0 release.

## Contributing to Migration Guides

If you encounter migration issues not covered here:

1. Open an issue describing the problem
2. Submit a PR with migration guide updates
3. Share your migration experience in discussions