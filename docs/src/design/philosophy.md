# Design Philosophy

## The Goal: Balance, not Dominance

There are excellent HTML-to-Markdown libraries in the Rust ecosystem —
some prioritise raw speed, others maximise conversion fidelity.
mdka is not trying to beat them on every axis.

Its aim is a **practical balance**:

> Produce stable, readable Markdown from real-world HTML,
> with an easy API, without surprising the caller at runtime.

Speed and memory efficiency matter, and mdka is designed with both in mind.
But they are means to an end, not the end itself.

## Real-World HTML is Messy

Web content rarely arrives as clean, well-formed documents. In practice you
encounter:

- HTML that a CMS generated and no human ever wrote
- SPA-rendered DOM fragments extracted from DevTools
- Scraped pages with ad slots, cookie banners, and navigation wrapped around the content
- Documents with 5,000 levels of nested `<div>` elements
- Missing closing tags, duplicate attributes, and unknown elements

mdka uses [scraper](https://crates.io/crates/scraper), which is built on
[html5ever](https://github.com/servo/html5ever) — the same parser used by
the Servo browser engine. It applies the HTML5 parsing algorithm, meaning:
unknown elements are handled gracefully, missing tags are inferred, and the
result is always a well-formed DOM tree, regardless of the input quality.

## No Stack Overflows

A common failure mode in tree-processing code is stack overflow on deeply
nested input. mdka uses an explicit `Vec`-based stack (non-recursive DFS)
for every tree traversal — both in the pre-processing pipeline and in the
Markdown conversion step. This means it handles any nesting depth that
fits in heap memory.

## Configurable Pre-Processing

HTML from different sources needs different treatment. A page scraped from a
news site has navigation, advertising, and footer content that a content
extraction pipeline wants to remove. A document being archived for audit
purposes should retain as much as possible.

The five [conversion modes](../api/modes.md) encode these intent differences
as named, opinionated presets. They are applied in a pre-processing pass
that filters the DOM before Markdown conversion runs — keeping the
conversion logic itself simple and mode-agnostic.

## One Allocator, Minimal Copies

The conversion pipeline is designed to minimise heap allocations:

- Whitespace normalisation is done in a single pass, writing directly into
  the output `String`.
- No regular expressions are used at runtime (avoiding compiled regex objects).
- The output `String` is pre-allocated with a capacity estimate.
- The `#[global_allocator]` counter in the CLI and benchmarks measures this
  directly.
