# Conversion Modes

A **conversion mode** selects a preset of [`ConversionOptions`](./options.md)
fields. `mdka` reads these fields directly during its single-pass DOM
traversal — there is no separate pre-processing stage.

## Overview

| Mode | Default? | Converts |
|---|---|---|
| `Balanced` | ✅ Yes | General use |
| `Minimal` | | Differently: drops shell elements, emits no `id` anchors |
| `Strict` | | As `Balanced` — an alias |
| `Semantic` | | As `Balanced` — an alias |
| `Preserve` | | As `Balanced` — an alias |

## ⚠ There are five modes but two behaviours

This is the single most important fact on this page.

**`Balanced`, `Strict`, `Semantic`, and `Preserve` produce identical output, and
cannot differ.** Only `Balanced` and `Minimal` convert differently from each
other. Choosing between the first four changes nothing, so pick `Balanced`.

Why they cannot differ: the four presets differ from each other **only** in the
defaults of six fields — `preserve_classes`, `preserve_data_attrs`,
`preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs` and
`unwrap_unknown_wrappers` — and none of the six can change the output.

- The first five are about HTML **attributes**, and Markdown has no syntax for
  attributes, so there is nothing in the output that could carry them. They are
  deprecated and have no effect at all — see the
  [Field Reference](./options.md#field-reference).
- `unwrap_unknown_wrappers` is not deprecated, but it has no effect either:
  unwrapping a `<div>`, `<section>`, `<article>` or `<main>` removes the tag
  while keeping the paragraph break it stood for, and Markdown has no wrapper
  element to show the difference. The tag's removal leaves no trace.
- The fields that **do** affect output — `preserve_ids` and
  `drop_interactive_shell` — have the same value in all four modes.

So there is no mechanism by which the four could diverge: the only things that
distinguish them are settings that cannot change a byte of Markdown. The mode
names describe purposes ("for debugging", "for archiving", "for accessibility")
that the conversion cannot act on, because none of them can be expressed in the
output format.

This is a statement about how the modes behave, and it says nothing about their
future: the enum variants, the CLI `--mode` values and the string names in the
Node.js and Python bindings are all still accepted, and nothing here is
deprecated.

It is asserted, not just observed. `tests/output_validity/mode_identity.rs`
runs a corpus of inputs — including wrappers in table cells, list items, quotes
and `<pre>` — through all five modes and fails if `Balanced`, `Strict`,
`Semantic` and `Preserve` differ by a byte
([P2](https://github.com/nabbisen/mdka-rs/blob/main/tests/output_validity/mode_identity.rs)),
and if flipping any one of the six fields above changes the output in any mode
([P1](https://github.com/nabbisen/mdka-rs/blob/main/tests/output_validity/mode_identity.rs)).
Older fixtures in
[`tests/characterisation_structural.rs`](https://github.com/nabbisen/mdka-rs/blob/main/tests/characterisation_structural.rs)
check the same thing on inputs chosen to discriminate a difference if one
existed.

`Minimal` is the only mode that converts differently — it drops shell elements
(`drop_interactive_shell`) and does not emit `id` anchors (`preserve_ids`).

---

## Balanced (default)

**What it does:** keeps `id` attributes (emits anchors), keeps shell
elements (`nav`/`header`/`footer`/`aside`), does not unwrap wrapper
elements.

```rust,fragment
let md = mdka::html_to_markdown(html); // Balanced is the default
```

**Use when:** you want the default behaviour without extra configuration.

---

## Minimal

**What it does:** drops shell elements (`nav`/`header`/`footer`/`aside`
and their children), does not emit `id` anchors. `unwrap_unknown_wrappers`
is also on here, but — as in every mode — it changes nothing in the output;
see the notice above.

The most aggressive mode for extracting body content — useful for piping
into an LLM prompt or a search index, where surrounding navigation chrome
is noise.

```rust,fragment
let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Strict, Semantic and Preserve

**Aliases of `Balanced`.** Each produces exactly `Balanced`'s output, for the
reasons in the notice above; they are kept so that code and command lines that
name them keep working. Their names suggest more than they do:

- `Strict` does not retain any more of the input than `Balanced`.
- `Semantic` does not treat ARIA attributes or document structure any
  differently. (`unwrap_unknown_wrappers` is on by default here, and in
  `Minimal`, but it has no effect.)
- `Preserve` does not keep anything `Balanced` drops — HTML comments, for
  example, are removed in every mode.

```rust,fragment
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Strict);
let md = mdka::html_to_markdown_with(html, &opts); // same as Balanced
```

---

## Choosing a Mode

```
Want the most aggressive extraction (LLM input, etc.)? → Minimal
Everything else                                        → Balanced (default)
```

`Strict`, `Semantic` and `Preserve` are not listed above because they behave
identically to `Balanced`; choosing one of them changes nothing.
