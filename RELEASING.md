# 🚀 Release Guide

This document describes how to create and publish releases for NameForge across multiple platforms.

## 📋 Prerequisites

Before creating a release, ensure you have:

1. ✅ **Crates.io account** with API token
2. ✅ **GitHub repository** with proper permissions
3. ✅ **Clean working directory** (no uncommitted changes)
4. ✅ **All tests passing** (`cargo test`)
5. ✅ **Code formatted** (`cargo fmt`)
6. ✅ **No clippy warnings** (`cargo clippy`)

## 🎯 Release Process

### Option 1: Automated Release (Recommended)

Use the provided release script:

```bash
# Make the script executable (if not already)
chmod +x scripts/release.sh

# Create a release (example for version 0.2.0)
./scripts/release.sh 0.2.0
```

This script will:
- ✅ Update version in `Cargo.toml`
- ✅ Update version in Homebrew formula
- ✅ Run tests and checks
- ✅ Commit changes
- ✅ Create and push git tag
- ✅ Trigger GitHub Actions for release

### Option 2: Manual Release

If you prefer manual control:

1. **Update version in Cargo.toml**:
   ```toml
   [package]
   version = "0.2.0"  # Update this
   ```

2. **Update Homebrew formula** (`homebrew/nameforge.rb`):
   ```ruby
   url "https://github.com/frontmesh/nameforge/archive/v0.2.0.tar.gz"
   ```

3. **Run quality checks**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo build --release
   ```

4. **Commit and tag**:
   ```bash
   git add Cargo.toml homebrew/nameforge.rb
   git commit -m "chore: bump version to 0.2.0"
   git tag -a "v0.2.0" -m "Release v0.2.0"
   git push origin master
   git push origin "v0.2.0"
   ```

## 🔄 Automated Release Pipeline

When you push a tag starting with `v` (e.g., `v0.2.0`), GitHub Actions will automatically:

### 1. 📦 Create GitHub Release
- ✅ Generate release notes
- ✅ Mark as stable release
- ✅ Create download page

### 2. 🏗️ Build Cross-Platform Binaries
- ✅ **Linux x86_64** (GNU)
- ✅ **Linux x86_64** (musl - static)
- ✅ **macOS x86_64** (Intel)
- ✅ **macOS ARM64** (Apple Silicon)
- ✅ **Windows x86_64**

### 3. 🦀 Publish to Crates.io
- ✅ Automatic publication after successful builds
- ✅ Available via `cargo install nameforge`

## 📚 Platform-Specific Instructions

### 🦀 Crates.io

**Setup (One-time)**:
1. Create account at https://crates.io
2. Generate API token in account settings
3. Add token to GitHub secrets as `CARGO_REGISTRY_TOKEN`

**Manual Publishing**:
```bash
# Login (if not automated)
cargo login

# Publish
cargo publish
```

### 🍺 Homebrew

#### For Personal Tap
1. The formula in `homebrew/nameforge.rb` can be used in a personal tap
2. After release, update the SHA256:
   ```bash
   # Get SHA256 of release tarball
   curl -sL https://github.com/frontmesh/nameforge/archive/v0.2.0.tar.gz | sha256sum
   ```

#### For Homebrew Core (Official)
1. **Fork** https://github.com/Homebrew/homebrew-core
2. **Create** `Formula/nameforge.rb` with our formula
3. **Update** SHA256 with actual release hash
4. **Submit** PR to homebrew-core

**Testing Homebrew Formula**:
```bash
# Test locally
brew install --build-from-source homebrew/nameforge.rb
brew test nameforge
```

### 🐧 Other Package Managers

#### AUR (Arch Linux)
Create a PKGBUILD that builds from source:
```bash
# Example PKGBUILD structure
source=("nameforge-$pkgver.tar.gz::https://github.com/frontmesh/nameforge/archive/v$pkgver.tar.gz")
```

#### Scoop (Windows)
Create a manifest in a Scoop bucket:
```json
{
    "version": "0.2.0",
    "url": "https://github.com/frontmesh/nameforge/releases/download/v0.2.0/nameforge-x86_64-pc-windows-msvc.zip"
}
```

## 🔐 Required GitHub Secrets

Add these secrets to your GitHub repository settings:

| Secret Name | Purpose | How to Get |
|-------------|---------|------------|
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | Generate at https://crates.io/me |

**Note**: `GITHUB_TOKEN` is automatically provided by GitHub Actions.

## 🐛 Troubleshooting

### Common Issues

**Release workflow fails to publish to crates.io**:
- ✅ Check `CARGO_REGISTRY_TOKEN` is set correctly
- ✅ Ensure version hasn't been published before
- ✅ Verify all required fields in `Cargo.toml`

**Binary builds fail**:
- ✅ Check for cross-compilation issues
- ✅ Ensure all dependencies support target platforms
- ✅ Review GitHub Actions logs

**Homebrew formula issues**:
- ✅ Verify SHA256 matches release tarball
- ✅ Test formula with `brew install --build-from-source`
- ✅ Check Rust version compatibility

### Manual Recovery

If automated release fails, you can:

1. **Delete the tag** and recreate:
   ```bash
   git tag -d v0.2.0
   git push origin :refs/tags/v0.2.0
   ```

2. **Manual binary uploads** to GitHub release
3. **Manual crates.io publish**: `cargo publish`

## 📈 Post-Release Checklist

After a successful release:

- [ ] ✅ Verify GitHub release page looks correct
- [ ] ✅ Test `cargo install nameforge` works
- [ ] ✅ Download and test platform binaries
- [ ] ✅ Update documentation if needed
- [ ] ✅ Announce on social media/forums
- [ ] ✅ Consider Homebrew core PR if it's a major release

## 🎉 Success!

Your release should now be available on:
- 📦 **GitHub Releases**: Download binaries
- 🦀 **Crates.io**: `cargo install nameforge`
- 🍺 **Homebrew**: `brew install nameforge` (after core acceptance)

## 📝 Release Notes Template

Use this template for release notes:

```markdown
## 📸 NameForge v0.2.0

### 🎉 New Features
- Feature 1 description
- Feature 2 description

### 🐛 Bug Fixes
- Fix 1 description
- Fix 2 description

### 🔧 Improvements
- Improvement 1
- Improvement 2

### 📦 Installation

#### From Binary
Download for your platform from the release assets below.

#### From Cargo
```bash
cargo install nameforge
```

#### From Homebrew
```bash
brew install nameforge
```

### 🙏 Contributors
Thanks to all contributors who made this release possible!
```