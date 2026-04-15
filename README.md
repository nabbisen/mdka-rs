# mdka

**A Rust library for converting HTML to Markdown.**

mdka balances conversion quality with runtime efficiency —
readable output from real-world HTML, without sacrificing speed or memory.

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
Python (PyO3 0.28).

> **On speed and memory:** streaming rewriters which, for example, uses
> `lol_html` internally are often faster on simple inputs
> because they skip the full DOM build. If raw throughput on clean,
> well-formed HTML is the only requirement, they are worth evaluating.
> mdka is the better fit when stability, mode control, and crash safety matter.
> See the [full benchmark notes](./docs/src/benchmarks/results.md).

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

Full documentation lives in the [`docs/`](./docs/) folder,
built as an [mdBook](https://rust-lang.github.io/mdBook/) project.

| Topic | Link |
|---|---|
| Installation | [docs/src/getting-started/installation.md](./docs/src/getting-started/installation.md) |
| Rust usage & examples | [docs/src/getting-started/usage-rust.md](./docs/src/getting-started/usage-rust.md) |
| Node.js usage | [docs/src/getting-started/usage-nodejs.md](./docs/src/getting-started/usage-nodejs.md) |
| Python usage | [docs/src/getting-started/usage-python.md](./docs/src/getting-started/usage-python.md) |
| CLI reference | [docs/src/getting-started/usage-cli.md](./docs/src/getting-started/usage-cli.md) |
| API reference | [docs/src/api/index.md](./docs/src/api/index.md) |
| Conversion modes | [docs/src/api/modes.md](./docs/src/api/modes.md) |
| ConversionOptions | [docs/src/api/options.md](./docs/src/api/options.md) |
| Supported elements | [docs/src/api/elements.md](./docs/src/api/elements.md) |
| Design philosophy | [docs/src/design/philosophy.md](./docs/src/design/philosophy.md) |
| Performance philosophy | [docs/src/design/performance.md](./docs/src/design/performance.md) |
| Architecture | [docs/src/design/architecture.md](./docs/src/design/architecture.md) |
| Benchmarks | [docs/src/design/performance.md#benchmark-results.md](./docs/src/design/performance.md#benchmark-results.md) |

To build the docs locally (requires mdBook):

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
