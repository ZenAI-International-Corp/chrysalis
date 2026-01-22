# Release Guide

This guide explains how to create a new release of Chrysalis.

## Automated Release Process

Chrysalis uses GitHub Actions to automatically build and publish releases for all supported platforms.

### Prerequisites

- Write access to the repository
- Git installed locally
- All changes committed and pushed to `main` branch

### Creating a Release

1. **Ensure all changes are committed and tests pass:**

```bash
cargo test
cargo build --release
```

2. **Create and push a version tag:**

```bash
# Create a new version tag (follow semantic versioning)
git tag v1.0.0

# Push the tag to GitHub
git push origin v1.0.0
```

3. **Wait for GitHub Actions to complete:**

The workflow will automatically:
- Build binaries for all platforms (Linux x64/ARM64, macOS x64/ARM64, Windows x64)
- Create a GitHub Release with the tag version
- Upload all binaries and checksums to the release
- Generate release notes with installation instructions

4. **Verify the release:**

Visit `https://github.com/ZenAI-International-Corp/chrysalis/releases` to verify:
- All platform binaries are uploaded
- SHA256 checksums are present
- Installation instructions are included

### Supported Platforms

The automated release process builds for:

- **Linux**:
  - `chrysalis-linux-amd64.tar.gz` (x86_64)
  - `chrysalis-linux-arm64.tar.gz` (ARM64)

- **macOS**:
  - `chrysalis-darwin-amd64.tar.gz` (Intel)
  - `chrysalis-darwin-arm64.tar.gz` (Apple Silicon)

- **Windows**:
  - `chrysalis-windows-amd64.exe.zip` (x86_64)

### Manual Release (if needed)

If you need to manually build and upload binaries:

```bash
# Build for current platform
cargo build --release

# The binary will be in target/release/chrysalis (or chrysalis.exe on Windows)

# Create archive
tar czf chrysalis-{os}-{arch}.tar.gz -C target/release chrysalis
# or on Windows:
# 7z a chrysalis-windows-amd64.exe.zip target/release/chrysalis.exe

# Generate checksum
shasum -a 256 chrysalis-{os}-{arch}.tar.gz > chrysalis-{os}-{arch}.sha256

# Upload to GitHub Releases manually
```

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **Major** (v2.0.0): Breaking changes
- **Minor** (v1.1.0): New features, backwards compatible
- **Patch** (v1.0.1): Bug fixes, backwards compatible

## Testing Installation Scripts

Before tagging a release, test the installation scripts:

### macOS/Linux:

```bash
# Test locally
bash install.sh

# Test from GitHub (after pushing)
curl -fsSL https://raw.githubusercontent.com/ZenAI-International-Corp/chrysalis/main/install.sh | bash
```

### Windows:

```powershell
# Test locally
.\install.ps1

# Test from GitHub (after pushing)
iwr -useb https://raw.githubusercontent.com/ZenAI-International-Corp/chrysalis/main/install.ps1 | iex
```

## Rollback

If a release has issues:

1. Delete the problematic release and tag from GitHub
2. Delete the local tag: `git tag -d v1.0.0`
3. Delete the remote tag: `git push origin :refs/tags/v1.0.0`
4. Fix the issues and create a new release

## Troubleshooting

### Build Fails

- Check GitHub Actions logs
- Ensure all tests pass locally
- Verify Cargo.toml versions are consistent

### Release Not Created

- Ensure tag starts with 'v' (e.g., v1.0.0)
- Check that GITHUB_TOKEN has proper permissions
- Verify the workflow file is correct

### Binary Not Working

- Test locally before releasing
- Check that all dependencies are statically linked
- Verify binary is compiled in release mode

## Post-Release

After a successful release:

1. Announce on relevant channels
2. Update documentation if needed
3. Monitor for issues
4. Prepare changelog for next release
