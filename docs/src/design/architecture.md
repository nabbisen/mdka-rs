# Architecture

## Workspace Layout

```
mdka/
├── src/               mdka library crate (lib only)
│   ├── lib.rs             Public API surface
│   ├── options.rs         ConversionMode, ConversionOptions
│   ├── traversal.rs       Markdown conversion traversal
│   ├── renderer.rs        MarkdownRenderer state machine
│   └── utils.rs           Whitespace normalisation + escaping
├── tests/             integration test modules
├── cli/               mdka-cli binary crate
│   └── src/main.rs        Argument parsing + dispatch
├── node/              Node.js bindings (napi-rs v3)
├── python/            Python bindings (PyO3 v0)
├── benches/           criterion benchmarks
│   └── alloc_counter.rs   Custom allocator (dev-only; benches + examples, not shipped in the library)
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

`MarkdownRenderer` is a state machine that maintains:

- **`output`**: the accumulated Markdown string
- **`list_stack`**: tracks nested ordered/unordered lists
- **`blockquote_depth`**: counts blockquote nesting level
- **`in_pre`**: whether inside a `<pre>` block
- **`at_line_start`**: deferred prefix flag for blockquote `> ` emission
- **`newlines_emitted`**: prevents double-blank-line accumulation

The `at_line_start` flag is key: rather than emitting `> ` prefixes
immediately when entering a blockquote, the renderer defers them until
actual content is written. This ensures nested blockquotes emit the
correct number of `>` characters regardless of how many block elements
intervene.

## Language Bindings

Both the Node.js and Python bindings are thin wrappers:

- **Node.js** (napi-rs): exposes sync and async (`tokio::spawn_blocking`)
  variants. The async variants release the Node.js event loop during conversion.
- **Python** (PyO3): exposes `py.detach()` on the batch function
  `html_to_markdown_many`, releasing the GIL for rayon parallel conversion.

The binding crates (`mdka-node`, `mdka-python`) have no conversion logic
of their own — they call the same Rust functions as the library and CLI.
