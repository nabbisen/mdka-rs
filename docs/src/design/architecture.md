# Architecture

## Workspace Layout

```
mdka/
├── src/               mdka library crate (lib only)
│   ├── lib.rs             Public API surface
│   ├── options.rs         ConversionMode, ConversionOptions
│   ├── traversal.rs       Markdown conversion traversal
│   ├── renderer.rs        MarkdownRenderer state machine
│   │   ├── sink.rs            The output sink: the only writer of Markdown
│   │   └── escape.rs          Escaping by context (RFC 010)
│   ├── utils.rs           Tag classification helpers
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
    │               · unwraps generic wrappers when opted in (tag removed,
    │                 separation kept — no output effect today)
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
- **The container prefix.** Blockquotes and list items form a stack in the
  sink. Every line written inside them -- content, blank separator lines and
  code block lines -- starts with the whole stack's prefix, outermost first:
  `> ` for a quote (`>` alone on a blank line), and for a list item as many
  spaces as its content column (`- ` → 2, `1. ` → 3). No element handler
  writes a prefix; the sink writes it before the first byte of a line. Line
  breaks are recorded and written when the next content arrives, so a blank
  line carries the prefix of exactly the containers still open across it.
  Whether a list is tight or loose is decided before rendering, by the same
  one pass over the document that finds inline elements around blocks.
- **Escaping by context** (`renderer/escape.rs`). Text is escaped as the sink
  writes it, from the line it is on -- a list item's or quote's content starts
  a block after its prefix -- and the character before it. An escape that
  depends on the character *after* it (`1986.` is a list marker only before a
  space; `!` opens an image only before `[`) waits, and is settled when the
  destination's next byte is written, by whatever writes it: text, markup or a
  line break. A backslash is then inserted in front of the waiting character.
  Code-span captures are not escaped, and a fenced block's opening fence is
  lengthened when it closes if its content holds a longer backtick run.

Inside code -- an inline `<code>`, or a `<pre>` with or without `<code>` --
child elements contribute text only: Markdown has no emphasis, links or images
inside code, so an image there contributes nothing. A `<pre>` owns its fence
and produces exactly one code block, holding the text of everything inside it
in order; a `<pre>` without a `<code>` child still produces a balanced block,
and the language comes from a `<code>` that opens the block.

**Inline elements around block content.** Before rendering, the traversal makes
one bottom-up pass over the document and marks each `<strong>`/`<b>`,
`<em>`/`<i>`, `<code>` and `<a>` that has a rendered block among its
descendants. What counts as a block comes from the same classification the
renderer's block arms use; an element the mode skips is left out, as the
traversal itself would leave it. An unwrapped wrapper (`<div>`, `<section>`,
`<article>`, `<main>`) still counts as the paragraph-like block it stands
in for — unwrapping removes the tag, not the separation (RFC 036 §5.2) —
matching what the traversal itself now does. A marked emphasis or code
element writes no delimiters. A marked link is written once per run of
inline content between
block boundaries: `## [Title](/x)`. Separately, a `<b>`/`<strong>` or
`<i>`/`<em>` whose own `style` negates its emphasis (`font-weight` `normal` or
≤ 500, `font-style: normal`) writes no delimiters.

## Language Bindings

Both the Node.js and Python bindings are thin wrappers:

- **Node.js** (napi-rs): exposes sync and async (`tokio::spawn_blocking`)
  variants. The async variants release the Node.js event loop during conversion.
- **Python** (PyO3): exposes `py.detach()` on the batch function
  `html_to_markdown_many`, releasing the GIL for rayon parallel conversion.

The binding crates (`mdka-node`, `mdka-python`) have no conversion logic
of their own — they call the same Rust functions as the library and CLI.
