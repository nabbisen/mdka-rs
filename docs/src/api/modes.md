# Conversion Modes

A **conversion mode** determines how mdka pre-processes the parsed DOM before
converting to Markdown. Choose the mode that matches the _origin_ and _purpose_
of your HTML.

## Overview

| Mode | Best for | Default? |
|---|---|---|
| `Balanced` | General use, blog posts, documentation pages | ✅ Yes |
| `Strict` | Debugging, comparing before/after, diff-friendly output | |
| `Minimal` | LLM pre-processing, text extraction, compression | |
| `Semantic` | SPA output, accessibility-aware pipelines, screen-reader content | |
| `Preserve` | Archiving, audit trails, round-trip fidelity | |

---

## Balanced (default)

**Goal:** Produce clean, readable Markdown without losing meaningful content.

- Removes decorative attributes: `class`, `style`, `data-*`
- Keeps semantic attributes: `href`, `src`, `alt`, `aria-*`, `lang`, `dir`
- Keeps `id` attributes (useful for anchor links)
- Does **not** remove navigation or structural elements

**Use when:** You want good-looking output without extra configuration.

```rust
let md = mdka::html_to_markdown(html); // Balanced is the default
```

---

## Strict

**Goal:** Preserve as much of the original HTML information as possible.
Output may be noisier, but nothing is silently dropped.

- Keeps `class`, `data-*`, `id`, `aria-*`, and most other attributes
- Does not unwrap wrapper elements
- Suitable for comparing two versions of a page, or for debugging
  unexpected output from other modes

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Strict);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Minimal

**Goal:** Extract the body text and essential structure; discard everything else.

- Removes all decorative attributes (`class`, `style`, `data-*`, `aria-*`)
- Optionally removes shell elements (`nav`, `header`, `footer`, `aside`)
  when [`drop_interactive_shell`](./options.md) is `true`
- Unwraps generic wrappers (`div`, `span`, `section`, `article`) that add no meaning
- Ideal for piping content into an LLM prompt or a search index

```rust
let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
opts.drop_interactive_shell = true;
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Semantic

**Goal:** Preserve document meaning and accessibility structure over visual appearance.

- Strongly retains `aria-*` attributes
- Retains `lang` and `dir`
- Retains heading hierarchy, list structure, link targets, and image alt text
- Removes purely visual attributes (`class`, `style`)
- Unwraps anonymous wrappers
- Good for SPA-rendered HTML where ARIA attributes carry structural meaning

```rust
let opts = ConversionOptions::for_mode(ConversionMode::Semantic);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Preserve

**Goal:** Maximum fidelity to the original HTML. Lose as little information as possible.

- Retains all attributes, including `class`, `data-*`, `aria-*`, `id`, and unknowns
- Retains HTML comments in the pre-processed output
- Does not unwrap any elements
- Intended for archiving or audit scenarios where the original structure matters

```rust
let opts = ConversionOptions::for_mode(ConversionMode::Preserve);
let md = mdka::html_to_markdown_with(html, &opts);
```

---

## Choosing a Mode

```
Is reproducibility the goal?          → Preserve
Are you feeding content to an LLM?    → Minimal  (+drop_interactive_shell)
Is the source a SPA or ARIA-heavy?    → Semantic
Debugging unexpected output?           → Strict
Everything else                        → Balanced  (default)
```
