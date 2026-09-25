# Conversion Modes

A **conversion mode** selects a preset of [`ConversionOptions`](./options.md)
fields. `mdka` reads these fields directly during its single-pass DOM
traversal — there is no separate pre-processing stage.

## Overview

| Mode | Default? | Converts |
|---|---|---|
| `Balanced` | ✅ Yes | General use |
| `Minimal` | | Differently: drops shell elements, emits no `id` anchors |
| `Strict` | | As `Balanced` — an alias. **Deprecated since 2.8.0, removed in 3.0** |
| `Semantic` | | As `Balanced` — an alias. **Deprecated since 2.8.0, removed in 3.0** |
| `Preserve` | | As `Balanced` — an alias. **Deprecated since 2.8.0, removed in 3.0** |

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

Because they cannot differ, `Strict`, `Semantic` and `Preserve` are
**deprecated since 2.8.0 and will be removed in 3.0**. Until then the enum
variants, the CLI `--mode` values and the string names in the Node.js and Python
bindings are all still accepted and keep their output; naming one emits a
warning (see [Deprecation](#deprecation) below). `Balanced` and `Minimal` are
not affected.

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

**Aliases of `Balanced`, deprecated since 2.8.0 and removed in 3.0.** Each
produces exactly `Balanced`'s output, for the reasons in the notice above; they
are kept until then so that code and command lines that name them keep working.
Their names suggest more than they do:

- `Strict` does not retain any more of the input than `Balanced`.
- `Semantic` does not treat ARIA attributes or document structure any
  differently. (`unwrap_unknown_wrappers` is on by default here, and in
  `Minimal`, but it has no effect.)
- `Preserve` does not keep anything `Balanced` drops — HTML comments, for
  example, are removed in every mode.

### Deprecation

Each surface warns in its own way, and only when the caller **names** one of the
three; a call that never names a mode, or names `Balanced` or `Minimal`, is
silent.

| Surface | What you see |
|---|---|
| Rust | A `#[deprecated(since = "2.8.0")]` warning at the use of `ConversionMode::Strict`, `Semantic` or `Preserve` |
| Node.js | A `DeprecationWarning` from `htmlToMarkdownWith` and `htmlToMarkdownMany`. The `Async` and file functions cannot emit it (they have no access to the runtime's warning channel) but convert identically |
| Python | A `DeprecationWarning` when `ConversionMode.Strict`, `Semantic` or `Preserve` is passed |
| CLI | One line on **stderr** — `mdka: warning: --mode strict is an alias of balanced and produces identical output; it is removed in 3.0` — with stdout untouched |

The fix is the same everywhere: use `Balanced` (`"balanced"`, `--mode balanced`),
or simply name no mode. Output does not change.

The string forms — `"strict"`, `"semantic"`, `"preserve"`, as accepted by
`--mode`, by the bindings and by `ConversionMode::from_str` — are deprecated in
the same way, but **nothing can warn you about them at compile time**: a mode
read from a config file or a variable is a string the compiler never sees, and a
library does not print. Change them to `"balanced"` now; what they do at `3.0`
is decided with the removal.

```rust,fragment
use mdka::options::{ConversionMode, ConversionOptions};

// Before: ConversionOptions::for_mode(ConversionMode::Strict) -- deprecated.
let opts = ConversionOptions::for_mode(ConversionMode::Balanced);
let md = mdka::html_to_markdown_with(html, &opts); // identical output
```

---

## Choosing a Mode

```
Want the most aggressive extraction (LLM input, etc.)? → Minimal
Everything else                                        → Balanced (default)
```

`Strict`, `Semantic` and `Preserve` are not listed above because they behave
identically to `Balanced` and are deprecated; choosing one of them changes
nothing except that it warns.
