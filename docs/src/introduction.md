# mdka

**mdka** is a Rust library for converting HTML to Markdown.

It aims to strike a practical balance between **conversion quality**
and **runtime efficiency** — readable output from real-world HTML,
without sacrificing speed or memory.

## At a Glance

| What you give it | What you get back |
|---|---|
| Any HTML string — a full page, a snippet, CMS output, SPA-rendered DOM | Clean, readable Markdown |
| A list of HTML files | Parallel Markdown output via rayon |
| A conversion mode (`minimal`, `semantic`, …) | Pre-processed output tuned for your use case |

## Key Properties

- **Parser foundation**: [scraper](https://crates.io/crates/scraper), which is built on
  [html5ever](https://github.com/servo/html5ever) — the same battle-tested parser
  used by the Servo browser engine. It handles malformed, deeply-nested, and
  real-world HTML gracefully.
- **Crash-resistant**: a non-recursive DFS traversal means even 10,000 levels of
  nesting will not overflow the stack.
- **Configurable**: five [conversion modes](./api/modes.md) let you tune the
  pre-processing pipeline — from noise-free LLM input to lossless archiving.
- **Multi-language**: available as a Rust library, a Node.js package (napi-rs v3),
  and a Python package (PyO3 v0).

## When to Choose mdka

mdka is a good fit if you need:

- **Stable, predictable output** from diverse HTML sources (CMS, SPA, scraped pages)
- **Mode-based pre-processing** to strip navigation, preserve ARIA, or retain attributes
- **Memory efficiency** at scale (bulk file conversion, streaming pipelines)
- **Multi-language access** from a single underlying Rust implementation

If raw speed on simple, well-formed HTML is the only concern, a streaming rewriter will be faster.

## Quick Navigation

- New to mdka? Start with [Installation](./getting-started/installation.md).
- Ready to integrate? Jump to [Usage & Examples](./getting-started/usage.md).
- Evaluating? Read [Design Philosophy](./design/philosophy.md) and
  [Performance Philosophy](./design/performance.md).
