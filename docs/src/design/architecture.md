# Architecture

## Workspace Layout

```
mdka/
├── src/               mdka library crate (lib only)
│   ├── lib.rs             Public API surface
│   ├── options.rs         ConversionMode, ConversionOptions
│   ├── traversal.rs       Markdown conversion traversal
│   ├── renderer.rs        MarkdownRenderer state machine
│   │   └── sink.rs            The output sink: the only writer of Markdown
│   ├── utils.rs           Whitespace normalisation + escaping
│   └── alloc_counter.rs   Custom allocator for benchmarks (deprecated since 2.2.2, removed in 2.4.0)
├── tests/             integration test modules
├── cli/               mdka-cli binary crate
│   └── src/main.rs        Argument parsing + dispatch
├── node/              Node.js bindings (napi-rs v3)
├── python/            Python bindings (PyO3 v0)
├── benches/           criterion benchmarks
└── examples/          Allocation measurement tool
```

## Conversion Pipeline

Each call to `html_to_markdown_with` follows these steps:

```
HTML string
    │
    ▼
[1] Parse        scraper::Html::parse_document()
    │             → html5ever DOM (tolerant HTML5 parsing)
    ▼
[2] Traverse     traversal::traverse(&doc, opts)
    │             → non-recursive DFS over ego-tree, Enter/Leave events
    │             Preprocessing is applied inline during this traversal:
    │               · drops script/style/head/svg/… unconditionally
    │               · drops shell elements when opted in
    │               · unwraps generic wrappers when opted in
    │             Drives MarkdownRenderer
    ▼
[3] Finalise     renderer.finish()
                  → trim trailing whitespace, single trailing newline
```

There is no intermediate HTML serialisation and no second parse. An earlier
version of the engine preprocessed HTML into a filtered HTML string and
re-parsed it before conversion; that round trip was removed, and this page now
describes the single-parse, single-traversal pipeline that actually runs.

## MarkdownRenderer

`MarkdownRenderer` is a state machine that tracks element context:

- **`list_stack`**: nested ordered/unordered lists
- **`in_pre`** and the pending code fence of the current `<pre>`
- which open `<a>` and `<code>` elements opened a capture

It never writes Markdown itself. Every byte goes through the **output sink**
(`renderer/sink.rs`), whose fields are private to its module, so an element
handler cannot write around it:

- **Destinations.** While a link's text or an inline code span is being
  collected, content goes into that construct's buffer; otherwise into the
  document. When the construct closes, the sink writes it into the destination
  it was opened in -- or nothing, for a link with no text and no image or a
  code span with no text.
- **Bookkeeping per destination**: `newlines_emitted` (prevents double blank
  lines), `at_line_start`, and the pending space between words.
- **The blockquote prefix.** Rather than emitting `> ` on entering a
  blockquote, the sink writes it before the first content byte at a line
  start -- text, emphasis delimiters, images, links, list and heading markers
  alike. Nested blockquotes get the correct number of `>` however many block
  elements intervene.

Inside an inline `<code>`, and a `<pre>` without a `<code>` child, child
elements contribute text only: Markdown has no emphasis, links or images inside
code. A `<pre>` opens its own fence, so a `<pre>` without a `<code>` child
still produces a balanced code block. Inside `<pre><code>`, output is kept
exactly as in 2.2.3.

## Language Bindings

Both the Node.js and Python bindings are thin wrappers:

- **Node.js** (napi-rs): exposes sync and async (`tokio::spawn_blocking`)
  variants. The async variants release the Node.js event loop during conversion.
- **Python** (PyO3): exposes `py.detach()` on the batch function
  `html_to_markdown_many`, releasing the GIL for rayon parallel conversion.

The binding crates (`mdka-node`, `mdka-python`) have no conversion logic
of their own — they call the same Rust functions as the library and CLI.
