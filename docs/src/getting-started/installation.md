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

**Download a prebuilt binary** — no Rust toolchain needed — from the
[latest release](https://github.com/nabbisen/mdka-rs/releases/latest). Which
platforms have one, and the exact archive name for each, is listed in the
[README's Quick Start](https://github.com/nabbisen/mdka-rs#try-it-from-the-command-line);
that table is the single source, so it is linked here rather than repeated.

Or install directly with cargo, which builds for whatever platform you are on:

```bash
cargo install mdka-cli
```

Or build from source using the `mdka-cli` crate in the workspace:

```bash
git clone https://github.com/nabbisen/mdka-rs
cd mdka-rs
cargo build --release -p mdka-cli
# Binary: ./target/release/mdka
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

Requires CPython 3.10 or later.

Pre-built wheels are published for CPython on these platforms. Each wheel uses
Python's stable ABI, so one wheel serves every CPython from 3.10 upward,
including versions released after it:

| Platform | Wheel tag |
|---|---|
| Linux x86_64, glibc 2.17 or later | `manylinux_2_17_x86_64` |
| Linux aarch64, glibc 2.17 or later | `manylinux_2_17_aarch64` |
| Linux x86_64, musl 1.2 or later (e.g. Alpine) | `musllinux_1_2_x86_64` |
| Linux aarch64, musl 1.2 or later | `musllinux_1_2_aarch64` |
| Windows x64 | `win_amd64` |
| macOS on Apple silicon, 11.0 or later | `macosx_11_0_arm64` |

Other interpreters — PyPy, free-threaded CPython — and other platforms have no
wheel. `pip install mdka` there falls back to the source distribution, which
needs a Rust toolchain to build. On free-threaded CPython that build works, but
mdka requires the GIL: Python re-enables it when mdka is imported, with a
`RuntimeWarning` that the GIL "has been enabled to load module
'mdka.mdka_python'". To build from source on purpose:
`pip install mdka --no-binary mdka` with Rust installed.

These apply from mdka 2.3.0. On CPython 3.8 or 3.9, pip does not offer 2.3.0
or later and installs the latest 2.2.x release instead.
