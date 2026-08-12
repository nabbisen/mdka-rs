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

## ⚠ Balanced, Strict, and Preserve currently produce identical output

This is the single most important fact on this page.

`Balanced`, `Strict`, and `Preserve` differ from each other **only** in the
defaults of five fields — `preserve_classes`, `preserve_data_attrs`,
`preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs` —
and those five fields have no effect on output (see
[Field Reference](./options.md#field-reference)). The fields that *do*
affect output — `preserve_ids`, `drop_interactive_shell`,
`unwrap_unknown_wrappers` — have the same value across all three modes.

This is a statement about **today's behaviour, not a deprecation**. The
three modes remain distinct API, are not merged, and may diverge again if
attribute preservation is ever implemented as a real feature. Proven
directly in
[`tests/characterisation_structural.rs`](https://github.com/nabbisen/mdka-rs/blob/main/tests/characterisation_structural.rs)
(`balanced_strict_preserve_are_identical_on_the_wrapper_fixture`,
`balanced_strict_preserve_are_identical_on_an_attribute_rich_element`),
which run all three through fixtures specifically chosen to discriminate a
difference if one existed, rather than inferring identity from fixtures
that happen not to distinguish them.

`Minimal` and `Semantic` are genuinely distinct from the other three and
from each other — `Minimal` additionally drops shell elements
(`drop_interactive_shell`), and `Semantic` additionally unwraps generic
wrappers (`unwrap_unknown_wrappers`) without dropping shell elements.

---

## Balanced (default)

**What it does today:** keeps `id` attributes (emits anchors), keeps shell
elements (`nav`/`header`/`footer`/`aside`), does not unwrap wrapper
elements.

```rust
let md = mdka::html_to_markdown(html); // Balanced is the default
```

**Use when:** you want the default behaviour without extra configuration.

---

## Strict

**Currently identical to `Balanced` and `Preserve`** — see the notice
above. Distinct API, in case attribute preservation becomes a real feature
later.

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Strict);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Minimal

**What it does today:** drops shell elements (`nav`/`header`/`footer`/`aside`
and their children), unwraps generic wrapper elements
(`div`/`span`/`section`/`article`/`main`), does not emit `id` anchors.

The most aggressive mode for extracting body content — useful for piping
into an LLM prompt or a search index, where surrounding navigation chrome
and wrapper markup are noise.

```rust
let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Semantic

**What it does today:** keeps shell elements, unwraps generic wrapper
elements, emits `id` anchors. The one mode that unwraps wrappers *without*
dropping shell elements — useful when you want compact structure but still
need navigation landmarks preserved.

```rust
let opts = ConversionOptions::for_mode(ConversionMode::Semantic);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Preserve

**Currently identical to `Balanced` and `Strict`** — see the notice above.
Distinct API, in case attribute preservation becomes a real feature later.

```rust
let opts = ConversionOptions::for_mode(ConversionMode::Preserve);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Choosing a Mode

```
Want wrappers unwrapped, but keep nav/header/footer?  → Semantic
Want the most aggressive extraction (LLM input, etc.)? → Minimal
Everything else                                        → Balanced (default)
```

`Strict` and `Preserve` are not listed above because they currently behave
identically to `Balanced` — pick `Balanced` unless you specifically want
the distinct API surface for forward compatibility.
