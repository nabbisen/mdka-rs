# mdka

**A HTML to Markdown convert written in Rust.**

[![crates.io](https://img.shields.io/crates/v/mdka?label=rust)](https://crates.io/crates/mdka)
[![npm](https://img.shields.io/npm/v/mdka)](https://www.npmjs.com/package/mdka)
[![pypi](https://img.shields.io/pypi/v/mdka)](https://www.pypi.org/project/mdka)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)

[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka)
[![Executable](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml)
[![npm](https://github.com/nabbisen/mdka-rs/actions/workflows/release-npm.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-npm.yaml)
[![PyPi](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml)

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

mdka balances conversion quality with runtime efficiency —
readable output from real-world HTML, without sacrificing speed or memory.

Documentation: https://nabbisen.github.io/mdka-rs/

---

## Why mdka?

There are several good HTML-to-Markdown converters in the Rust ecosystem.
mdka's specific focus is:

**Reliable output from diverse HTML sources.**
It is built on [scraper](https://crates.io/crates/scraper), which uses
[html5ever](https://github.com/servo/html5ever) — the HTML5 parser from
the Servo browser engine. html5ever applies the same parsing algorithm that
web browsers use, so it handles malformed tags, deeply nested structures,
CMS output, and SPA-rendered DOM without special-casing.

**Crash resistance.**
Conversion uses non-recursive DFS throughout. There is no stack overflow,
no matter the nesting depth.

**Configurable pre-processing.**
Five [conversion modes](#conversion-modes) let you tune what gets kept or
stripped — from noise-free LLM input to lossless archiving.

**Multi-language.**
The same Rust implementation is accessible from Node.js (napi-rs v3) and
Python (PyO3 v0).

---

## Quick Start

### Try it from the command line

```bash
cargo install mdka-cli

echo '<h1>Hello</h1><p><strong>world</strong></p>' | mdka
# # Hello
#
# **world**
```

```bash
mdka page.html                          # → page.md  (same directory)
mdka --mode minimal --drop-shell *.html # strip nav/header/footer
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
use mdka::{html_to_markdown_with};
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Minimal)
    .drop_interactive_shell(true);
let md = html_to_markdown_with(html, &opts);
```

### Add to a Node.js project

```bash
npm install mdka
```

```js
const { htmlToMarkdown, htmlToMarkdownWith } = require('mdka')

const md = htmlToMarkdown('<h1>Hello</h1>')

const md = await htmlToMarkdownWithAsync(html, {
  mode: 'minimal',
  dropInteractiveShell: true,
})
```

### Add to a Python project

```bash
pip install mdka
```

```python
import mdka

md = mdka.html_to_markdown('<h1>Hello</h1>')

md = mdka.html_to_markdown_with(
    html,
    mode=mdka.ConversionMode.MINIMAL,
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

---

## Learn More

Full documentation lives in the [`docs/`](./docs/) folder, published as GitHub Pages: https://nabbisen.github.io/mdka-rs/ . 

| Topic | Link |
|---|---|
| Installation | [/getting-started/installation](https://nabbisen.github.io/mdka-rs/getting-started/installation) |
| Rust usage & examples | [/getting-started/usage-rust](https://nabbisen.github.io/mdka-rs/getting-started/usage-rust) |
| Node.js usage | [/getting-started/usage-nodejs](https://nabbisen.github.io/mdka-rs/getting-started/usage-nodejs) |
| Python usage | [/getting-started/usage-python](https://nabbisen.github.io/mdka-rs/getting-started/usage-python) |
| CLI reference | [/getting-started/usage-cli](https://nabbisen.github.io/mdka-rs/getting-started/usage-cli) |
| API reference | [/api/index](https://nabbisen.github.io/mdka-rs/api/index) |
| Conversion modes | [/api/modes](https://nabbisen.github.io/mdka-rs/api/modes) |
| ConversionOptions | [/api/options](https://nabbisen.github.io/mdka-rs/api/options) |
| Supported elements | [/api/elements](https://nabbisen.github.io/mdka-rs/api/elements) |
| Design philosophy | [/design/philosophy](https://nabbisen.github.io/mdka-rs/design/philosophy) |
| Performance concern | [/design/performance](https://nabbisen.github.io/mdka-rs/design/performance) |
| Architecture | [/design/architecture](https://nabbisen.github.io/mdka-rs/design/architecture) |

 Note: docs are built as an [mdBook](https://rust-lang.github.io/mdBook/) project. To build them locally (requires mdBook):

```bash
cd docs
mdbook build   # output → docs/book/
mdbook serve   # live-reload preview at http://localhost:3000
```

---

## Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your work.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [scraper](https://crates.io/crates/scraper), Servo's [html5ever](https://github.com/servo/html5ever) / markup5ever.

Also, [napi-rs](https://github.com/napi-rs/napi-rs) on binding for Node.js and PyO3's [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python.
