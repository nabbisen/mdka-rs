# Installation

## As a Rust Library

Add mdka to your `Cargo.toml`:

```toml
[dependencies]
mdka = "2"
```

That is the only step. mdka has no system dependencies.

**Minimum Supported Rust Version:** 1.85 (2024 Edition)

## As a CLI Binary

Build from source using the `mdka-cli` crate in the workspace:

```bash
git clone https://github.com/example/mdka
cd mdka
cargo build --release -p mdka-cli
# Binary: ./target/release/mdka
```

Or install directly with cargo:

```bash
cargo install mdka-cli
```

## As a Node.js Package

```bash
npm install mdka
# or
yarn add mdka
```

Requires Node.js 16 or later.    
Pre-built binaries are bundled for major platforms such as Linux, macOS and Windows of specific architecture.    
On other platforms, run `npm run build` with Rust installed.

## As a Python Package

```bash
pip install mdka
```

Requires Python 3.8 or later.
Pre-built wheels are provided for CPython on major platforms.
To build from source: `pip install mdka --no-binary mdka` with Rust installed.
