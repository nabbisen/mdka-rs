# Architecture

## Workspace Layout

```
mdka/
├── src/               mdka library crate (lib only)
│   ├── lib.rs             Public API surface
│   ├── options.rs         ConversionMode, ConversionOptions
│   ├── preprocessor.rs    DOM pre-processing pipeline
│   ├── traversal.rs       Markdown conversion traversal
│   ├── renderer.rs        MarkdownRenderer state machine
│   ├── utils.rs           Whitespace normalisation + escaping
│   └── alloc_counter.rs   Custom allocator (for benchmarks)
├── cli/               mdka-cli binary crate
│   └── src/main.rs        Argument parsing + dispatch
├── node/              Node.js bindings (napi-rs v3)
├── python/            Python bindings (PyO3 0.28)
├── benches/           criterion benchmarks
└── examples/          Allocation measurement tool
```

## Conversion Pipeline

Each call to `html_to_markdown_with` follows these steps:

```
HTML string
    │
    ▼
[1] Parse          scraper::Html::parse_document()
    │               → html5ever DOM tree (tolerant HTML5 parsing)
    ▼
[2] Pre-process    preprocessor::preprocess(&doc, opts)
    │               → filtered HTML string
    │               Non-recursive DFS over ego-tree nodes
    │               Drops: script, style, iframe, …
    │               Filters attributes per ConversionOptions
    │               Removes shell elements (if opted in)
    │               Unwraps anonymous wrappers (if opted in)
    ▼
[3] Re-parse       scraper::Html::parse_document(&cleaned)
    │               → clean DOM for conversion
    ▼
[4] Convert        traversal::traverse(&doc)
    │               → Markdown string
    │               Non-recursive DFS with Enter/Leave events
    │               Drives MarkdownRenderer via event callbacks
    ▼
[5] Finalise       renderer.finish()
                    → trim leading/trailing whitespace
                    → ensure single trailing newline
```

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
