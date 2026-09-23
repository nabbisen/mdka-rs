# mdka

**A HTML to Markdown converter written in Rust.**

[![crates.io](https://img.shields.io/crates/v/mdka?label=rust)](https://crates.io/crates/mdka)
[![npm](https://img.shields.io/npm/v/mdka)](https://www.npmjs.com/package/mdka)
[![pypi](https://img.shields.io/pypi/v/mdka)](https://www.pypi.org/project/mdka)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)    
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka)
[![CI](https://github.com/nabbisen/mdka-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/ci.yaml)
[![Executable](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml)
[![npm](https://github.com/nabbisen/mdka-rs/actions/workflows/release-npm.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-npm.yaml)
[![PyPi](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml)

![logo](https://raw.githubusercontent.com/nabbisen/mdka-rs/main/docs/src/assets/logo.png)

mdka balances conversion quality with runtime efficiency —
correct, readable Markdown from real-world HTML, at competitive speed and
near-flat memory.    
"ka" means "化 (か)" pointing to conversion.

---

## Why mdka?

There are several good HTML-to-Markdown converters in the Rust ecosystem.
mdka's specific focus is:

- **Reliable output from diverse HTML sources.**
    It is built on [scraper](https://crates.io/crates/scraper), which uses
[html5ever](https://github.com/servo/html5ever) — the HTML5 parser from
the Servo browser engine. html5ever applies the same parsing algorithm that
web browsers use, so it handles malformed tags, deeply nested structures,
CMS output, and SPA-rendered DOM without special-casing.
- **Crash resistance.**
    Conversion uses non-recursive DFS throughout. There is no stack overflow,
no matter the nesting depth. That is a claim about crashing, not speed —
deep nesting still costs real time, quadratically; see
[Scaling: Depth and Width](https://nabbisen.github.io/mdka-rs/design/performance-characteristics.html#scaling-depth-and-width).
- **Configurable pre-processing.**
    Five [conversion modes](https://nabbisen.github.io/mdka-rs/api/modes.html) let you tune what gets kept or
stripped, from noise-free LLM input to maximum retention. Four of the five
currently produce identical output — see [Conversion Modes](https://nabbisen.github.io/mdka-rs/api/modes.html).
- **Multi-language.**
    The same Rust implementation is accessible from Node.js (napi-rs) and
Python (PyO3).

---

## Quick Start

### Try it from the command line

**Download a prebuilt binary** (no Rust toolchain needed) from the
[latest release](https://github.com/nabbisen/mdka-rs/releases/latest):

| Platform | Asset |
|---|---|
| Linux x64 (glibc) | `mdka@Linux-x64-gnu-<version>.tar.gz` |
| Linux x64 (musl) | `mdka@Linux-x64-musl-<version>.tar.gz` |
| Linux aarch64 (musl) | `mdka@Linux-aarch64-musl-<version>.tar.gz` |
| macOS Apple Silicon | `mdka@macOS-aarch64-<version>.zip` |
| Windows x64 | `mdka@Windows-x64-<version>.zip` |

Other platforms (macOS Intel, Windows ARM, Linux aarch64 glibc) aren't built
as binaries — use `cargo install mdka-cli` below instead.

Extract the archive; it contains one folder holding the `mdka` binary:

```bash
cd mdka@Linux-x64-gnu-<version>  # the folder the archive extracted to

echo '<h1>Hello</h1><p><strong>world</strong></p>' | ./mdka
# # Hello
#
# **world**
```

**Or install via cargo** — requires `cargo` (Rust language) installed:

```bash
cargo install mdka-cli

echo '<h1>Hello</h1><p><strong>world</strong></p>' | mdka
# # Hello
#
# **world**
```

```bash
mdka page.html                          # → page.md  (same directory)
mdka --mode minimal --drop-shell -o out/ *.html  # strip nav/header/footer
mdka --help                             # full option list
```

### Add to a Rust project

```toml
# Cargo.toml
[dependencies]
mdka = "2"
```

```rust
use mdka::html_to_markdown;

let md = html_to_markdown("<h1>Hello</h1><p><em>world</em></p>");
// "# Hello\n\n*world*\n"
```

With options:

```rust
use mdka::html_to_markdown_with;
use mdka::options::{ConversionMode, ConversionOptions};

let html = "<nav>menu</nav><h1>Hello</h1>";
let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
opts.drop_interactive_shell = true;
let md = html_to_markdown_with(html, &opts);
// "# Hello\n"
```

### Add to a Node.js project

```bash
npm install mdka
```

```js
const { htmlToMarkdown, htmlToMarkdownWithAsync } = require('mdka')

const md = htmlToMarkdown('<h1>Hello</h1>')
// "# Hello\n"

async function main() {
  const html = '<nav>menu</nav><h1>Hello</h1>'
  const minimal = await htmlToMarkdownWithAsync(html, {
    mode: 'minimal',
    dropInteractiveShell: true,
  })
  console.log(minimal)
}
main()
```

### Add to a Python project

```bash
pip install mdka
```

```python
import mdka

md = mdka.html_to_markdown('<h1>Hello</h1>')
# "# Hello\n"

html = '<nav>menu</nav><h1>Hello</h1>'
minimal = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.Minimal,
    drop_interactive_shell=True,
)
```

---

## Conversion Modes

| Mode | Use when |
|---|---|
| `Balanced` | General use — default |
| `Strict` | Debugging, diff comparison |
| `Minimal` | LLM input, text extraction |
| `Semantic` | SPA content, ARIA-aware pipelines |
| `Preserve` | Archiving, audit trails |

**`Balanced`, `Strict`, `Semantic` and `Preserve` currently produce identical
output.** They differ only in the defaults of fields that have no effect
today, so choosing between them changes nothing today. They remain distinct
API and may diverge again — see
[Conversion Modes](https://nabbisen.github.io/mdka-rs/api/modes), which explains
why in full.

**Tables are not yet converted.** `<table>` cell text is emitted without
structure or separators, so a table becomes a run of joined text. See
[Supported Elements](https://nabbisen.github.io/mdka-rs/api/elements) for the
full list of what is and is not supported.

---

## Learn More

Full documentation is published as GitHub Pages, and its source lives in
[`docs/`](https://github.com/nabbisen/mdka-rs/tree/main/docs).

https://nabbisen.github.io/mdka-rs/

| Topic | Link |
|---|---|
| Installation | [/getting-started/installation](https://nabbisen.github.io/mdka-rs/getting-started/installation) |
| Rust Usage & Examples | [/getting-started/usage-rust](https://nabbisen.github.io/mdka-rs/getting-started/usage-rust) |
| Node.js Usage | [/getting-started/usage-nodejs](https://nabbisen.github.io/mdka-rs/getting-started/usage-nodejs) |
| Python Usage | [/getting-started/usage-python](https://nabbisen.github.io/mdka-rs/getting-started/usage-python) |
| CLI Reference | [/getting-started/usage-cli](https://nabbisen.github.io/mdka-rs/getting-started/usage-cli) |
| API Reference | [/api/index](https://nabbisen.github.io/mdka-rs/api/index) |
| Conversion Modes | [/api/modes](https://nabbisen.github.io/mdka-rs/api/modes) |
| ConversionOptions | [/api/options](https://nabbisen.github.io/mdka-rs/api/options) |
| Supported Elements | [/api/elements](https://nabbisen.github.io/mdka-rs/api/elements) |
| Design Philosophy | [/design/philosophy](https://nabbisen.github.io/mdka-rs/design/philosophy) |
| Performance Characteristics | [/design/performance-characteristics](https://nabbisen.github.io/mdka-rs/design/performance-characteristics) |
| Architecture | [/design/architecture](https://nabbisen.github.io/mdka-rs/design/architecture) |
| Features | [/design/features](https://nabbisen.github.io/mdka-rs/design/features) |
| Changelog | [CHANGELOG.md](https://github.com/nabbisen/mdka-rs/blob/main/CHANGELOG.md) |
| Roadmap | [ROADMAP.md](https://github.com/nabbisen/mdka-rs/blob/main/ROADMAP.md) |

---

## Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your work.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [scraper](https://crates.io/crates/scraper) (+ [html5ever](https://github.com/servo/html5ever)), [ego-tree](https://crates.io/crates/ego-tree), [rayon](https://crates.io/crates/rayon), [thiserror](https://crates.io/crates/thiserror).

Also, [napi-rs](https://github.com/napi-rs/napi-rs) on binding for Node.js and PyO3's [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python.
