# Conversion Modes

A **conversion mode** selects a preset of [`ConversionOptions`](./options.md)
fields. `mdka` reads these fields directly during its single-pass DOM
traversal — there is no separate pre-processing stage.

## Overview

| Mode | Default? | Converts |
|---|---|---|
| `Balanced` | ✅ Yes | General use |
| `Minimal` | | Differently: drops shell elements, emits no `id` anchors |

There are two modes, and they make two behaviours.

## Removed in 3.0: `Strict`, `Semantic` and `Preserve`

Until 3.0 there were five modes. `Strict`, `Semantic` and `Preserve` were **aliases
of `Balanced`**: they produced byte-for-byte the same output and could not
differ, so they were deprecated in 2.8.0 and removed in 3.0.

Why they could not differ: the presets differed from each other **only** in the
defaults of six fields that could not change the output — five about HTML
**attributes**, for which Markdown has no syntax, and one about wrapper
elements, whose unwrapping leaves no trace in Markdown. Their names described
purposes ("for debugging", "for archiving", "for accessibility") that the
conversion could not act on. Nothing that used one converts differently now; the
name only has to change.

**What to do:** use `Balanced`, or name no mode at all. Use `Minimal` if you
wanted something genuinely different.

**What you see if you still name one**, on each surface — all of them fail, and say
*removed* rather than *unknown*, because a removed name is an obsolete
configuration, not a wrong one:

| Surface | What happens |
|---|---|
| Rust, `"strict".parse::<ConversionMode>()` | `Err("conversion mode 'strict' was removed in 3.0; it was an alias of 'balanced'. Use 'balanced'.")`. The variants `ConversionMode::Strict`, `Semantic` and `Preserve` no longer exist, so naming one is a compile error. `ConversionMode::parse_mode` returns `None` and no message, as it does for any name it does not accept — which is why it is **deprecated since 3.0.0** in favour of `str::parse`, which reports why a name was rejected |
| Node.js | `htmlToMarkdown` and `htmlToMarkdownMany` **throw**, and `htmlToMarkdownAsync` and the file functions **reject**, with the same message |
| Python | `ConversionMode.Strict`, `Semantic` and `Preserve` do not exist: `AttributeError`. A mode is never a string in Python, so there is no string form |
| CLI | `mdka --mode strict` exits with status 1 and prints `error: conversion mode 'strict' was removed in 3.0; it was an alias of 'balanced'. Use 'balanced'.` on stderr |

A name that never existed (`--mode bogus`) still gets the *unknown* message, with
the list of valid modes.

It is asserted, not just observed. `tests/output_validity/mode_identity.rs` runs a
corpus of inputs — including wrappers in table cells, list items, quotes and
`<pre>` — through `Balanced` and `Minimal` and fails if either differs by a byte
from what the published `2.9.0` produced
([P3](https://github.com/nabbisen/mdka-rs/blob/main/tests/output_validity/mode_identity.rs)).

`Minimal` is the only mode that converts differently from `Balanced` — it drops
shell elements (`drop_interactive_shell`) and does not emit `id` anchors
(`preserve_ids`).

---

## Balanced (default)

**What it does:** keeps `id` attributes (emits anchors) and keeps shell
elements (`nav`/`header`/`footer`/`aside`). A wrapper element (`<div>`,
`<section>`, `<article>`, `<main>`) is rendered as the paragraph break it stands
for.

```rust,fragment
let md = mdka::html_to_markdown(html); // Balanced is the default
```

**Use when:** you want the default behaviour without extra configuration.

---

## Minimal

**What it does:** drops shell elements (`nav`/`header`/`footer`/`aside`
and their children), does not emit `id` anchors, and unwraps wrapper elements —
which changes nothing in the output, since unwrapping keeps the paragraph break
the wrapper stood for.

The most aggressive mode for extracting body content — useful for piping
into an LLM prompt or a search index, where surrounding navigation chrome
is noise.

```rust,fragment
let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Choosing a Mode

```
Want the most aggressive extraction (LLM input, etc.)? → Minimal
Everything else                                        → Balanced (default)
```
