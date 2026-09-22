# Conversion Modes

A **conversion mode** selects a preset of [`ConversionOptions`](./options.md)
fields. `mdka` reads these fields directly during its single-pass DOM
traversal — there is no separate pre-processing stage.

## Overview

| Mode | Default? |
|---|---|
| `Balanced` | ✅ Yes |
| `Strict` | |
| `Minimal` | |
| `Semantic` | |
| `Preserve` | |

## ⚠ Balanced, Strict, Semantic, and Preserve currently produce identical output

This is the single most important fact on this page.

`Balanced`, `Strict`, `Semantic`, and `Preserve` differ from each other
**only** in the defaults of six fields — `preserve_classes`,
`preserve_data_attrs`, `preserve_aria_attrs`, `preserve_unknown_attrs`,
`drop_presentation_attrs` (deprecated, no effect on output at all — see
[Field Reference](./options.md#field-reference)), and `unwrap_unknown_wrappers`
(not deprecated, but *currently* no effect either: unwrapping a `<div>`,
`<section>`, `<article>`, or `<main>` removes the tag while keeping the
paragraph break it stood for, so the tag's removal alone leaves no
Markdown-visible trace). The fields that do affect output —
`preserve_ids`, `drop_interactive_shell` — have the same value across all
four modes.

This is a statement about **today's behaviour, not a deprecation**. The
four modes remain distinct API, are not merged, and may diverge again if
attribute preservation, or a mode that preserves raw HTML wrappers, is ever
implemented as a real feature. Proven directly in
[`tests/characterisation_structural.rs`](https://github.com/nabbisen/mdka-rs/blob/main/tests/characterisation_structural.rs)
(`balanced_strict_semantic_preserve_are_identical_on_the_wrapper_fixture`,
`balanced_strict_semantic_preserve_are_identical_on_an_attribute_rich_element`),
which run all four through fixtures specifically chosen to discriminate a
difference if one existed, rather than inferring identity from fixtures
that happen not to distinguish them.

`Minimal` is the only mode genuinely distinct from the other four — it
drops shell elements (`drop_interactive_shell`) and does not emit `id`
anchors (`preserve_ids`).

---

## Balanced (default)

**What it does today:** keeps `id` attributes (emits anchors), keeps shell
elements (`nav`/`header`/`footer`/`aside`), does not unwrap wrapper
elements.

```rust,fragment
let md = mdka::html_to_markdown(html); // Balanced is the default
```

**Use when:** you want the default behaviour without extra configuration.

---

## Strict

**Currently identical to `Balanced`, `Semantic`, and `Preserve`** — see the
notice above. Distinct API, in case attribute preservation becomes a real
feature later.

```rust,fragment
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Strict);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Minimal

**What it does today:** drops shell elements (`nav`/`header`/`footer`/`aside`
and their children), does not emit `id` anchors. `unwrap_unknown_wrappers`
is also on here (as in `Semantic`), but — like there — it currently has
nothing left to change output-wise; see the notice above.

The most aggressive mode for extracting body content — useful for piping
into an LLM prompt or a search index, where surrounding navigation chrome
and wrapper markup are noise.

```rust,fragment
let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Semantic

**Currently identical to `Balanced`, `Strict`, and `Preserve`** — see the
notice above. `unwrap_unknown_wrappers` is on by default here (off in the
other three), but it currently has nothing left to change: unwrapping a
wrapper element keeps its paragraph break, so the tag's removal alone is
invisible in Markdown. Distinct API — a future mode that preserves raw HTML
wrappers would make this default matter again, and `preserve_ids` and
`drop_interactive_shell` (the fields that do affect output) are set the
same way here as in `Balanced`, `Strict`, and `Preserve`.

```rust,fragment
let opts = ConversionOptions::for_mode(ConversionMode::Semantic);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Preserve

**Currently identical to `Balanced`, `Strict`, and `Semantic`** — see the
notice above. Distinct API, in case attribute preservation becomes a real
feature later.

```rust,fragment
let opts = ConversionOptions::for_mode(ConversionMode::Preserve);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Choosing a Mode

```
Want the most aggressive extraction (LLM input, etc.)? → Minimal
Everything else                                        → Balanced (default)
```

`Strict`, `Semantic`, and `Preserve` are not listed above because they
currently behave identically to `Balanced` — pick `Balanced` unless you
specifically want one of their distinct API surfaces for forward
compatibility.
