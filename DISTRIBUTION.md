# ASS LSP Distribution Guide

This guide covers the various methods for distributing the ASS LSP server to users and integrating it with different editors and systems.

## Table of Contents

1. [Distribution Methods](#distribution-methods)
2. [Publishing to Crates.io](#publishing-to-cratesio)
3. [GitHub Releases](#github-releases)
4. [Package Managers](#package-managers)
5. [Editor Extensions](#editor-extensions)
6. [Manual Installation](#manual-installation)
7. [Distribution Checklist](#distribution-checklist)
8. [Versioning Strategy](#versioning-strategy)
9. [Platform Support](#platform-support)
10. [Security Considerations](#security-considerations)

## Distribution Methods

### 1. Rust Crates.io (Recommended)

The primary distribution method for Rust projects. Users can install with:
```bash
cargo install ass-lsp
```

**Advantages:**
- Automatic dependency resolution
- Easy updates with `cargo install --force ass-lsp`
- Source code verification
- Cross-platform support

**Requirements:**
- Rust toolchain on user's system
- Compilation time during installation

### 2. Pre-compiled Binaries

Distribute pre-compiled binaries for major platforms via GitHub Releases.

**Supported Platforms:**
- `x86_64-unknown-linux-gnu` (Linux 64-bit)
- `x86_64-unknown-linux-musl` (Linux 64-bit, static linking)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows 64-bit)

**Advantages:**
- No compilation required
- Faster installation
- Works without Rust toolchain

### 3. Package Managers

Distribution through system package managers:

#### Homebrew (macOS/Linux)
```bash
brew install ass-lsp
```

#### Chocolatey (Windows)
```bash
choco install ass-lsp
```

#### Scoop (Windows)
```bash
scoop install ass-lsp
```

#### AUR (Arch Linux)
```bash
yay -S ass-lsp
```

### 4. Editor Extensions

Integrated extensions for popular editors that bundle or automatically install the LSP server.

## Publishing to Crates.io

### Prerequisites

1. Create a crates.io account at https://crates.io/
2. Generate an API token: https://crates.io/me
3. Login locally: `cargo login <token>`

### Pre-publication Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Ensure `README.md` is up to date
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `cargo clippy` to check for lints
- [ ] Run `cargo fmt` to ensure consistent formatting
- [ ] Update documentation with `cargo doc`

### Publishing Steps

1. **Dry Run:**
   ```bash
   cargo publish --dry-run
   ```

2. **Publish:**
   ```bash
   cargo publish
   ```

3. **Verify:**
   - Check the package page on crates.io
   - Test installation: `cargo install ass-lsp`

### Post-publication

1. Create a git tag: `git tag v0.2.0`
2. Push the tag: `git push origin v0.2.0`
3. Create a GitHub release
4. Update documentation and examples

## GitHub Releases

### Automated Releases with CI/CD

The included `.github/workflows/release.yml` automates:
- Building for multiple platforms
- Running tests and lints
- Creating release artifacts
- Publishing to crates.io

### Release Process

1. **Prepare Release:**
   ```bash
   # Update version
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   
   # Update changelog
   echo "## [0.2.0] - $(date +%Y-%m-%d)" >> CHANGELOG.md
   
   # Commit changes
   git add .
   git commit -m "Release v0.2.0"
   git push
   ```

2. **Create Release:**
   ```bash
   # Create and push tag
   git tag v0.2.0
   git push origin v0.2.0
   
   # Or use GitHub CLI
   gh release create v0.2.0 --generate-notes
   ```

3. **Monitor CI/CD:**
   - Check GitHub Actions for build status
   - Verify artifacts are uploaded
   - Confirm crates.io publication

## Package Managers

### Homebrew Formula

Create a formula for Homebrew:

```ruby
# Formula/ass-lsp.rb
class AssLsp < Formula
  desc "Language Server Protocol implementation for ASS/SSA subtitle format"
  homepage "https://github.com/wiedymi/ass-lsp"
  url "https://github.com/wiedymi/ass-lsp/archive/v0.2.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/ass-lsp", "--version"
  end
end
```

### Chocolatey Package

Create a `nuspec` file for Chocolatey:

```xml
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>ass-lsp</id>
    <version>0.2.0</version>
    <title>ASS LSP Server</title>
    <authors>wiedymi</authors>
    <projectUrl>https://github.com/wiedymi/ass-lsp</projectUrl>
    <licenseUrl>https://github.com/wiedymi/ass-lsp/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>Language Server Protocol implementation for Advanced SubStation Alpha (ASS/SSA) subtitle format</description>
    <tags>lsp language-server ass ssa subtitles</tags>
  </metadata>
  <files>
    <file src="ass-lsp.exe" target="tools" />
  </files>
</package>
```

### AUR Package

Create a `PKGBUILD` for Arch Linux:

```bash
# Maintainer: wiedymi <email@example.com>
pkgname=ass-lsp
pkgver=0.2.0
pkgrel=1
pkgdesc="Language Server Protocol implementation for ASS/SSA subtitle format"
arch=('x86_64')
url="https://github.com/wiedymi/ass-lsp"
license=('MIT')
depends=()
makedepends=('rust')
source=("$pkgname-$pkgver.tar.gz::https://github.com/wiedymi/ass-lsp/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/ass-lsp" "$pkgdir/usr/bin/ass-lsp"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

## Editor Extensions

### VS Code Extension

1. **Create Extension Structure:**
   ```
   ass-lsp-vscode/
   ├── package.json
   ├── src/
   │   └── extension.ts
   ├── client/
   │   └── src/
   │       └── extension.ts
   └── server/
       └── ass-lsp (binary)
   ```

2. **Package Configuration:**
   ```json
   {
     "name": "ass-lsp",
     "displayName": "ASS LSP",
     "description": "Language support for ASS/SSA subtitle format",
     "version": "0.2.0",
     "engines": {
       "vscode": "^1.60.0"
     },
     "categories": ["Programming Languages"],
     "main": "./out/extension.js",
     "activationEvents": [
       "onLanguage:ass"
     ],
     "contributes": {
       "languages": [{
         "id": "ass",
         "aliases": ["ASS", "SSA"],
         "extensions": [".ass", ".ssa"]
       }]
     }
   }
   ```

3. **Publishing:**
   ```bash
   npm install -g vsce
   vsce package
   vsce publish
   ```

### Neovim Plugin

Create a plugin for Neovim package managers:

```lua
-- lua/ass-lsp/init.lua
local M = {}

function M.setup(opts)
  opts = opts or {}
  
  -- Auto-install LSP server if not found
  if opts.auto_install and not vim.fn.executable('ass-lsp') then
    M.install_server()
  end
  
  -- Configure LSP
  require('lspconfig').ass_lsp.setup(opts)
end

function M.install_server()
  vim.fn.jobstart({'cargo', 'install', 'ass-lsp'}, {
    on_exit = function(_, code)
      if code == 0 then
        vim.notify('ASS LSP server installed successfully')
      else
        vim.notify('Failed to install ASS LSP server', vim.log.levels.ERROR)
      end
    end
  })
end

return M
```

## Manual Installation

### Installation Script

The included `install.sh` script provides:
- Automatic dependency checking
- Multiple installation methods
- PATH configuration
- Error handling and recovery

### Usage:
```bash
# Install from crates.io
curl -sSL https://raw.githubusercontent.com/wiedymi/ass-lsp/main/install.sh | bash -s -- -m crates

# Install from source
git clone https://github.com/wiedymi/ass-lsp
cd ass-lsp
./install.sh -m source
```

### Docker Distribution

Create a Docker image for containerized usage:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ass-lsp /usr/local/bin/ass-lsp
ENTRYPOINT ["ass-lsp"]
```

## Distribution Checklist

### Pre-Release

- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated
- [ ] Documentation updated
- [ ] Tests passing
- [ ] Clippy lints resolved
- [ ] Code formatted
- [ ] Security audit passed (`cargo audit`)

### Release

- [ ] Git tag created
- [ ] GitHub release created
- [ ] Crates.io publication successful
- [ ] CI/CD pipeline completed
- [ ] Binaries available for download
- [ ] Documentation deployed

### Post-Release

- [ ] Package manager PRs submitted
- [ ] Editor extension updates
- [ ] Community announcements
- [ ] Usage examples updated
- [ ] Issue templates updated

## Versioning Strategy

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Examples:
- `0.1.0` → `0.1.1`: Bug fixes
- `0.1.0` → `0.2.0`: New features
- `0.1.0` → `1.0.0`: Breaking changes

## Platform Support

### Tier 1 (Fully Supported)
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `x86_64-pc-windows-msvc`

### Tier 2 (Best Effort)
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-gnu`

### Tier 3 (Community Supported)
- `i686-pc-windows-msvc`
- `i686-unknown-linux-gnu`
- Other platforms on request

## Security Considerations

### Supply Chain Security

1. **Dependency Auditing:**
   ```bash
   cargo audit
   ```

2. **Reproducible Builds:**
   - Use locked dependencies
   - Pin CI/CD tool versions
   - Verify build reproducibility

3. **Signing:**
   - Sign releases with GPG
   - Use GitHub's attestation features
   - Verify package integrity

### Distribution Security

1. **HTTPS Only:**
   - Use HTTPS for all download links
   - Verify SSL certificates

2. **Checksums:**
   - Provide SHA256 checksums
   - Include checksums in releases

3. **Verification:**
   - Provide signature verification instructions
   - Document verification process

## Troubleshooting Distribution Issues

### Common Problems

1. **Crates.io Upload Failures:**
   - Check package size limits
   - Verify all required metadata
   - Ensure unique version number

2. **CI/CD Build Failures:**
   - Check cross-compilation setup
   - Verify target platform availability
   - Monitor GitHub Actions logs

3. **Binary Distribution Issues:**
   - Test on target platforms
   - Verify dynamic library dependencies
   - Check architecture compatibility

### Support Channels

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Community support
- Documentation: Comprehensive guides
- Examples: Working configurations

## Maintenance

### Regular Tasks

- [ ] Update dependencies monthly
- [ ] Security audit quarterly
- [ ] Documentation review quarterly
- [ ] Platform support review annually

### Monitoring

- Download statistics
- Issue reports
- Community feedback
- Performance metrics

This distribution guide ensures comprehensive coverage of all distribution methods while maintaining security and reliability standards.