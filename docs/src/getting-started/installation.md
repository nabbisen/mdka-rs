# Installation

## As a Rust Library

Add mdka to your `Cargo.toml`:

```toml
[dependencies]
mdka = "2"
```

That is the only step. mdka has no system dependencies.

**Minimum Supported Rust Version:** 1.88 (2024 Edition)

## As a CLI Binary

Build from source using the `mdka-cli` crate in the workspace:

```bash
git clone https://github.com/nabbisen/mdka-rs
cd mdka-rs
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

Prebuilt native bindings are published for **three** platforms, resolved
automatically through `optionalDependencies`:

| Platform | Package |
|---|---|
| Linux x64 (glibc) | `@mdka/lib-linux-x64-gnu` |
| macOS Apple Silicon | `@mdka/lib-darwin-arm64` |
| Windows x64 (MSVC) | `@mdka/lib-win32-x64-msvc` |

**On any other platform — musl, Linux arm64, macOS Intel, Windows ARM — there
is no fallback inside the package.** The published tarball contains four files
(`index.js`, `index.d.ts`, `package.json`, `README.md`) and no Rust source, so
`npm run build` cannot work from an installed copy: there is nothing to build,
and the napi toolchain is a development dependency that is not installed for
consumers.

What does work on those platforms:

- Build the binding from the repository — clone
  [nabbisen/mdka-rs](https://github.com/nabbisen/mdka-rs), then `cd node && npm
  install && npm run build`, which needs a Rust toolchain. This produces a local
  binding; it does not make `npm install mdka` work elsewhere.
- Use the CLI instead: `cargo install mdka-cli`, which builds from source for
  whatever platform you are on.
- Use the Rust crate directly, if the surrounding project allows it.

## As a Python Package

```bash
pip install mdka
```

Requires Python 3.8 or later.
Pre-built wheels are provided for CPython on major platforms.
To build from source: `pip install mdka --no-binary mdka` with Rust installed.
