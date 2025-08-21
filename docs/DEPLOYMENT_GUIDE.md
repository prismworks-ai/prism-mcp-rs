# Deployment Guide

## Overview

This guide covers how to package, distribute, and deploy Rust MCP servers built with prism-mcp-rs. After building your server, you'll need to make it accessible to users across different platforms and distribution channels.

## Table of Contents

- [Binary Packaging](#binary-packaging)
- [Cross-Platform Builds](#cross-platform-builds)
- [Distribution Methods](#distribution-methods)
- [Installation Scripts](#installation-scripts)
- [Desktop Extensions (.dxt)](#desktop-extensions-dxt)
- [Package Managers](#package-managers)
- [Container Deployment](#container-deployment)
- [System Integration](#system-integration)

## Binary Packaging

### Release Builds

Always use optimized release builds for distribution:

```bash
# Standard release build
cargo build --release

# Optimized for size
cargo build --release --profile release-lto
```

Add to your `Cargo.toml`:

```toml
[profile.release-lto]
inherits = "release"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Platform-Specific Considerations

#### Linux
```bash
# Static linking for better portability
cargo build --release --target x86_64-unknown-linux-musl

# Check dependencies
ldd target/release/your-server
```

#### macOS
```bash
# Universal binary for Intel + Apple Silicon
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine into universal binary
lipo -create \
  target/x86_64-apple-darwin/release/your-server \
  target/aarch64-apple-darwin/release/your-server \
  -output target/release/your-server-universal
```

#### Windows
```bash
# Standard Windows build
cargo build --release --target x86_64-pc-windows-msvc

# For older Windows compatibility
cargo build --release --target i686-pc-windows-msvc
```

## Cross-Platform Builds

### Using GitHub Actions

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            asset_name: linux-amd64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            asset_name: linux-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            asset_name: windows-amd64.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            asset_name: darwin-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            asset_name: darwin-arm64

    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install musl tools
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: sudo apt-get install -y musl-tools
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package
        run: |
          mkdir -p dist
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            cp target/${{ matrix.target }}/release/your-server.exe dist/your-server-${{ matrix.asset_name }}
          else
            cp target/${{ matrix.target }}/release/your-server dist/your-server-${{ matrix.asset_name }}
          fi
      
      - name: Upload Release Asset
        uses: actions/upload-artifact@v3
        with:
          name: your-server-${{ matrix.asset_name }}
          path: dist/
```

### Using Cross

Install and use [cross](https://github.com/cross-rs/cross):

```bash
cargo install cross

# Build for multiple targets
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
cross build --release --target x86_64-pc-windows-gnu
```

## Distribution Methods

### 1. GitHub Releases

**Automated release script** (`scripts/release.sh`):

```bash
#!/bin/bash
set -e

VERSION=${1:-$(git describe --tags --always)}
BINARY_NAME="your-server"

echo "Building release $VERSION..."

# Build for all targets
targets=(
  "x86_64-unknown-linux-musl"
  "aarch64-unknown-linux-musl"
  "x86_64-pc-windows-msvc"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
)

mkdir -p releases

for target in "${targets[@]}"; do
  echo "Building for $target..."
  cargo build --release --target $target
  
  # Package binary
  if [[ $target == *"windows"* ]]; then
    binary="target/$target/release/${BINARY_NAME}.exe"
    archive="releases/${BINARY_NAME}-${VERSION}-${target}.zip"
    zip -j $archive $binary
  else
    binary="target/$target/release/${BINARY_NAME}"
    archive="releases/${BINARY_NAME}-${VERSION}-${target}.tar.gz"
    tar -czf $archive -C "target/$target/release" $BINARY_NAME
  fi
done

echo "Release artifacts created in releases/"
```

### 2. Package Repositories

#### Homebrew (macOS)

Create a Homebrew formula:

```ruby
# Formula/your-server.rb
class YourServer < Formula
  desc "MCP server built with prism-mcp-rs"
  homepage "https://github.com/your-org/your-server"
  url "https://github.com/your-org/your-server/releases/download/v1.0.0/your-server-v1.0.0-darwin-universal.tar.gz"
  sha256 "sha256_hash_here"
  license "MIT"

  def install
    bin.install "your-server"
  end

  test do
    assert_match "your-server", shell_output("#{bin}/your-server --version")
  end
end
```

#### APT Repository (Debian/Ubuntu)

Create `.deb` packages:

```bash
# Install packaging tools
sudo apt-get install dpkg-dev devscripts

# Create package structure
mkdir -p your-server-1.0.0/DEBIAN
mkdir -p your-server-1.0.0/usr/local/bin

# Copy binary
cp target/release/your-server your-server-1.0.0/usr/local/bin/

# Create control file
cat > your-server-1.0.0/DEBIAN/control << EOF
Package: your-server
Version: 1.0.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Your Name <your.email@example.com>
Description: MCP server built with prism-mcp-rs
 A high-performance MCP server implementation.
EOF

# Build package
dpkg-deb --build your-server-1.0.0
```

#### Cargo (Rust Package Manager)

If distributing as a Cargo package:

```toml
# Cargo.toml
[package]
name = "your-server"
version = "1.0.0"

[[bin]]
name = "your-server"
path = "src/main.rs"
```

Users can install with:
```bash
cargo install your-server
```

### 3. Container Images

#### Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-alpine as builder

WORKDIR /app
COPY . .

RUN apk add --no-cache musl-dev
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.18

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/your-server /usr/local/bin/

EXPOSE 8080
CMD ["your-server"]
```

#### Multi-arch builds

```bash
# Build and push multi-arch image
docker buildx build --platform linux/amd64,linux/arm64 \
  -t your-org/your-server:latest \
  --push .
```

## Installation Scripts

### Universal installer script

Create `install.sh`:

```bash
#!/bin/bash
set -e

BINARY_NAME="your-server"
GITHUB_REPO="your-org/your-server"
INSTALL_DIR="/usr/local/bin"

# Detect platform
detect_platform() {
  local os=$(uname -s | tr '[:upper:]' '[:lower:]')
  local arch=$(uname -m)
  
  case $os in
    linux)
      case $arch in
        x86_64) echo "x86_64-unknown-linux-musl" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-musl" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    darwin)
      case $arch in
        x86_64|arm64) echo "universal-apple-darwin" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    mingw*|msys*|cygwin*)
      echo "x86_64-pc-windows-msvc"
      ;;
    *)
      echo "Unsupported OS: $os" >&2
      exit 1
      ;;
  esac
}

# Get latest release
get_latest_release() {
  curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
    grep '"tag_name":' | \
    sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
main() {
  local platform=$(detect_platform)
  local version=${1:-$(get_latest_release)}
  
  echo "Installing $BINARY_NAME $version for $platform..."
  
  local url="https://github.com/$GITHUB_REPO/releases/download/$version/${BINARY_NAME}-${version}-${platform}"
  if [[ $platform == *"windows"* ]]; then
    url="${url}.zip"
  else
    url="${url}.tar.gz"
  fi
  
  # Create temp directory
  local temp_dir=$(mktemp -d)
  cd "$temp_dir"
  
  # Download
  echo "Downloading from $url..."
  curl -L "$url" -o archive
  
  # Extract
  if [[ $platform == *"windows"* ]]; then
    unzip archive
  else
    tar -xzf archive
  fi
  
  # Install
  sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
  sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
  
  # Cleanup
  cd /
  rm -rf "$temp_dir"
  
  echo "$BINARY_NAME installed successfully to $INSTALL_DIR"
  echo "Run '$BINARY_NAME --version' to verify installation"
}

main "$@"
```

### Platform-specific installers

#### PowerShell installer for Windows

Create `install.ps1`:

```powershell
param(
    [string]$Version = "latest",
    [string]$InstallPath = "$env:LOCALAPPDATA\Programs\YourServer"
)

$ErrorActionPreference = "Stop"

$repoOwner = "your-org"
$repoName = "your-server"
$binaryName = "your-server.exe"

function Get-LatestVersion {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$repoOwner/$repoName/releases/latest"
    return $response.tag_name
}

function Install-YourServer {
    if ($Version -eq "latest") {
        $Version = Get-LatestVersion
    }
    
    Write-Host "Installing $binaryName $Version..."
    
    $downloadUrl = "https://github.com/$repoOwner/$repoName/releases/download/$Version/$binaryName-$Version-x86_64-pc-windows-msvc.zip"
    $tempPath = Join-Path $env:TEMP "your-server-install.zip"
    
    # Download
    Write-Host "Downloading from $downloadUrl..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath
    
    # Create install directory
    if (!(Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    }
    
    # Extract
    Expand-Archive -Path $tempPath -DestinationPath $InstallPath -Force
    
    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$InstallPath*") {
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$InstallPath", "User")
        Write-Host "Added $InstallPath to PATH"
    }
    
    # Cleanup
    Remove-Item $tempPath
    
    Write-Host "$binaryName installed successfully to $InstallPath"
    Write-Host "Restart your terminal and run '$binaryName --version' to verify installation"
}

Install-YourServer
```

## Desktop Extensions (.dxt)

Desktop Extensions provide one-click installation for Claude Desktop and other compatible AI tools.

### Creating a Desktop Extension

1. **Install the DXT CLI:**
   ```bash
   npm install -g @anthropic-ai/dxt
   ```

2. **Create manifest.json:**
   ```json
   {
     "name": "your-mcp-server",
     "version": "1.0.0",
     "description": "Your MCP server description",
     "author": "Your Name",
     "license": "MIT",
     "homepage": "https://github.com/your-org/your-server",
     "type": "binary",
     "main": {
       "linux": "bin/your-server-linux",
       "darwin": "bin/your-server-darwin",
       "win32": "bin/your-server.exe"
     },
     "mcpSettings": {
       "serverName": "your-server",
       "description": "A powerful MCP server",
       "args": [],
       "env": {}
     },
     "capabilities": [
       "tools",
       "resources"
     ],
     "schema": {
       "type": "object",
       "properties": {
         "apiKey": {
           "type": "string",
           "title": "API Key",
           "description": "Your API key",
           "sensitive": true
         },
         "endpoint": {
           "type": "string",
           "title": "API Endpoint",
           "default": "https://api.example.com"
         }
       }
     }
   }
   ```

3. **Directory structure:**
   ```
   your-server-dxt/
   ├── manifest.json
   ├── bin/
   │   ├── your-server-linux
   │   ├── your-server-darwin
   │   └── your-server.exe
   └── README.md
   ```

4. **Build the extension:**
   ```bash
   dxt pack
   ```

### Distribution

Upload the `.dxt` file to:
- GitHub releases
- Your website
- Extension directories

Users install with:
- Download `.dxt` file
- Open Claude Desktop
- Go to Extensions menu
- Install extension

## System Integration

### Service Files

#### systemd (Linux)

Create `/etc/systemd/system/your-server.service`:

```ini
[Unit]
Description=Your MCP Server
After=network.target

[Service]
Type=simple
User=mcp-user
Group=mcp-user
ExecStart=/usr/local/bin/your-server --config /etc/your-server/config.json
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable your-server
sudo systemctl start your-server
```

#### launchd (macOS)

Create `~/Library/LaunchAgents/com.yourorg.your-server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.yourorg.your-server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/your-server</string>
        <string>--config</string>
        <string>/usr/local/etc/your-server/config.json</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load service:
```bash
launchctl load ~/Library/LaunchAgents/com.yourorg.your-server.plist
```

#### Windows Service

Use a service wrapper like [winsw](https://github.com/winsw/winsw):

Create `your-server-service.xml`:

```xml
<service>
  <id>YourMCPServer</id>
  <n>Your MCP Server</n>
  <description>MCP server built with prism-mcp-rs</description>
  <executable>C:\Program Files\YourServer\your-server.exe</executable>
  <arguments>--config "C:\Program Files\YourServer\config.json"</arguments>
  <log mode="roll"/>
</service>
```

Install service:
```cmd
winsw install your-server-service.xml
winsw start YourMCPServer
```

## Configuration Management

### Environment-based Configuration

Support multiple deployment environments:

```rust
// In your server code
#[derive(Debug, Deserialize)]
pub struct Config {
    pub environment: Environment,
    pub database_url: String,
    pub api_endpoint: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!(
                "config/{}",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            )).required(false))
            .add_source(config::Environment::with_prefix("MCP"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

### Configuration Files

Provide environment-specific configs:

```yaml
# config/production.yaml
database_url: "${DATABASE_URL}"
api_endpoint: "https://api.production.com"
log_level: "warn"
max_connections: 100

# config/development.yaml  
database_url: "sqlite:dev.db"
api_endpoint: "http://localhost:3000"
log_level: "debug"
max_connections: 10
```

## Monitoring and Observability

### Health Checks

Implement health check endpoints:

```rust
use warp::Filter;

pub fn health_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "version": env!("CARGO_PKG_VERSION")
            }))
        })
}
```

### Metrics

Add metrics collection:

```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref REQUEST_COUNTER: Counter = register_counter!(
        "mcp_requests_total", 
        "Total number of MCP requests"
    ).unwrap();
    
    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "mcp_request_duration_seconds",
        "Duration of MCP requests"
    ).unwrap();
}
```

### Logging

Structure logs for production:

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

## Security Hardening

### Binary Hardening

1. **Strip debug symbols:**
   ```bash
   strip target/release/your-server
   ```

2. **Enable security features:**
   ```toml
   [profile.release]
   strip = true
   lto = true
   codegen-units = 1
   ```

3. **Static analysis:**
   ```bash
   cargo audit
   cargo clippy -- -D warnings
   ```

### Runtime Security

1. **Run as non-root user**
2. **Use minimal file permissions**
3. **Implement input validation**
4. **Use secure transport (TLS)**
5. **Regular security updates**

## Deployment Checklist

Before deploying to production:

- [ ] **Security audit completed**
- [ ] **All tests passing**
- [ ] **Performance benchmarks meet requirements**
- [ ] **Logging configured appropriately**
- [ ] **Health checks implemented**
- [ ] **Monitoring setup**
- [ ] **Backup procedures in place**
- [ ] **Rollback plan prepared**
- [ ] **Documentation updated**
- [ ] **User notification sent**

## Next Steps

After deployment:

1. **Monitor server performance and logs**
2. **Collect user feedback**
3. **Plan incremental updates**
4. **Maintain security patches**
5. **Scale based on usage patterns**

For troubleshooting deployment issues, see the [Troubleshooting Guide](./TROUBLESHOOTING.md).