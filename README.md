# Chrysalis 🦋

Modern, high-performance build system for Flutter Web projects written in Rust.

> Transform your Flutter web code like a butterfly emerging from its chrysalis.

## Features

- 🚀 **Blazingly Fast**: Rewritten in Rust for maximum performance
- 📦 **Single Binary**: Zero dependencies, works on any CI/CD system
- 🔧 **Integrated**: Runs Flutter commands (`pub get`, `build web`) automatically
- 🎯 **Optimized**: Minification, hashing, and chunking built-in
- ⚙️ **Configurable**: TOML-based configuration with sensible defaults
- 🏗️ **Modular**: Plugin-based architecture with clean separation
- 🔒 **Type-Safe**: Leverages Rust's type system for correctness

## Quick Start

### Installation

#### One-Line Install (Recommended)

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/ZenAI-International-Corp/chrysalis/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/ZenAI-International-Corp/chrysalis/main/install.ps1 | iex
```

#### Download Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/ZenAI-International-Corp/chrysalis/releases) page:

- **Linux**: `chrysalis-linux-amd64.tar.gz` or `chrysalis-linux-arm64.tar.gz`
- **macOS**: `chrysalis-darwin-amd64.tar.gz` or `chrysalis-darwin-arm64.tar.gz`
- **Windows**: `chrysalis-windows-amd64.exe.zip`

Extract and add to your PATH.

#### From Source

```bash
git clone https://github.com/ZenAI-International-Corp/chrysalis.git
cd chrysalis
cargo build --release
cp target/release/chrysalis /usr/local/bin/
```

### Usage

```bash
# Initialize configuration
chrysalis init

# Build Flutter web project
chrysalis build

# Build with custom options
chrysalis build --skip-chunk --verbose

# Clean build artifacts
chrysalis clean
```

## Architecture

Chrysalis is organized as a Rust workspace with multiple crates:

```
chrysalis/
├── crates/
│   ├── chrysalis-cli/       # CLI entry point
│   ├── chrysalis-config/    # Configuration system
│   ├── chrysalis-core/      # Core build system
│   ├── chrysalis-flutter/   # Flutter integration
│   └── chrysalis-plugins/   # Build plugins
└── Cargo.toml               # Workspace configuration
```

### Build Pipeline

```
1. Flutter Build → Run `flutter pub get` and `flutter build web`
2. Scan         → Index all output files
3. Minify       → Compress JS/CSS/HTML/JSON
4. Hash         → Add content hashes to filenames
5. Chunk        → Split large files into chunks
6. Inject       → Inject chunk loader into HTML
```

## Configuration

Create `chrysalis.toml` in your project root:

```toml
[flutter]
run_pub_get = true
release = true
target_dir = "build/web"

[build]
chunk_size_kb = 400
hash_length = 8

[plugins.minify]
enabled = true
minify_js = true
minify_css = true

[plugins.chunk]
enabled = true
include = ["*.js"]
```

See [chrysalis.toml](./chrysalis.toml) for full configuration options.

## CLI Usage

```bash
# Build with all optimizations
chrysalis build

# Build without chunking
chrysalis build --skip-chunk

# Build with verbose output
chrysalis build --verbose

# Clean build artifacts
chrysalis clean

# Generate default config
chrysalis init

# Show version
chrysalis --version
```

## CI/CD Integration

### Automated Releases

Chrysalis uses GitHub Actions to automatically build and release binaries for all platforms. To create a new release:

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

This will automatically:
- Build binaries for Linux (amd64, arm64), macOS (amd64, arm64), and Windows (amd64)
- Create a GitHub release with all binaries
- Generate SHA256 checksums for verification

### GitHub Actions

**Using pre-built binary (Recommended):**

```yaml
- name: Install Chrysalis
  run: curl -fsSL https://raw.githubusercontent.com/ZenAI-International-Corp/chrysalis/main/install.sh | bash

- name: Build Flutter Web
  run: chrysalis build --verbose
```

**Building from source:**

```yaml
- name: Setup Rust
  uses: actions-rs/toolchain@v1
  with:
    toolchain: stable

- name: Build and Install Chrysalis
  run: |
    git clone https://github.com/ZenAI-International-Corp/chrysalis.git
    cd chrysalis
    cargo build --release
    cp target/release/chrysalis /usr/local/bin/

- name: Build Flutter Web
  run: chrysalis build --verbose
```

### GitLab CI

```yaml
build:
  before_script:
    - cargo build --release
    - cp target/release/chrysalis /usr/local/bin/
  script:
    - chrysalis build
```

### Docker

```dockerfile
FROM ghcr.io/cirruslabs/flutter:stable

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build Chrysalis from source
WORKDIR /tmp/chrysalis
COPY . .
RUN cargo build --release && \
    cp target/release/chrysalis /usr/local/bin/ && \
    rm -rf /tmp/chrysalis

WORKDIR /app
COPY . .

RUN chrysalis build
```

## Development

### Prerequisites

- Rust 1.70+
- Flutter SDK (for testing)

### Building

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Run specific crate
cargo run -p chrysalis-cli -- build
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Test specific crate
cargo test -p chrysalis-core
```

## Design Principles

1. **Performance First**: Written in Rust for speed and safety
2. **Zero Config**: Works out of the box with sensible defaults
3. **Single Binary**: No runtime dependencies or complex setup
4. **CI/CD Ready**: Designed for automated build systems
5. **Type Safe**: Leverages Rust's type system for correctness
6. **Modular**: Clean separation of concerns via crates
7. **Observable**: Clear progress reporting and error messages

## Roadmap

- [x] Core build system
- [x] Flutter integration
- [x] Plugin architecture
- [x] Minification (JS/CSS/HTML/JSON)
- [x] Content hashing
- [x] File chunking
- [x] Chunk loader injection
- [ ] Incremental builds
- [ ] Parallel processing
- [ ] Source maps
- [ ] Tree shaking
- [ ] Image optimization
- [ ] Bundle analysis
- [ ] Watch mode

## Release Process

See [RELEASE.md](./RELEASE.md) for instructions on creating a new release.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.
