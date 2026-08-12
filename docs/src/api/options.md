# ConversionOptions

```rust
pub struct ConversionOptions {
    pub mode: ConversionMode,

    // Attribute retention
    pub preserve_ids:             bool,
    pub preserve_classes:         bool,        // deprecated, no effect
    pub preserve_data_attrs:      bool,        // deprecated, no effect
    pub preserve_aria_attrs:      bool,        // deprecated, no effect
    pub preserve_unknown_attrs:   bool,        // deprecated, no effect

    // Structural behaviour
    pub drop_presentation_attrs:  bool,        // deprecated, no effect
    pub drop_interactive_shell:   bool,
    pub unwrap_unknown_wrappers:  bool,
}
```

`ConversionOptions` controls the details of how mdka's single-pass DOM
traversal renders Markdown. There is no separate pre-processing stage —
the traversal in `src/traversal.rs` reads these fields directly as it
walks the parsed document once. You rarely need to set individual fields —
start with a mode and override only what differs from the default for
that mode.

**Five of the eight fields below have no effect on output and are
deprecated as of `2.2.0`.** Markdown has no attribute syntax, so
"preserve this attribute" was never expressible in the output format —
see [RFC 005](https://github.com/nabbisen/mdka-rs/blob/main/rfcs/done/005-conversion-options-semantics.md)
for the full history. They are marked below; nothing is removed, and no
output changes if you are currently setting them.

## Creating Options

### From a mode (recommended)

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
```

`for_mode` returns sensible defaults for the chosen mode. See the table below.

### Modify fields after creation

```rust
let mut opts = ConversionOptions::for_mode(ConversionMode::Balanced);
opts.drop_interactive_shell = true; // also strip nav/header/footer/aside
opts.preserve_ids           = false; // don't emit <a id="…"> anchors
```

### Default

```rust
let opts = ConversionOptions::default(); // equivalent to for_mode(Balanced)
```

## Field Defaults by Mode

| Field | Balanced | Strict | Minimal | Semantic | Preserve | Effect |
|---|---|---|---|---|---|---|
| `preserve_ids` | ✅ | ✅ | ❌ | ✅ | ✅ | Emits anchors |
| `preserve_classes` | ❌ | ✅ | ❌ | ❌ | ✅ | **None — deprecated** |
| `preserve_data_attrs` | ❌ | ✅ | ❌ | ❌ | ✅ | **None — deprecated** |
| `preserve_aria_attrs` | ✅ | ✅ | ❌ | ✅ | ✅ | **None — deprecated** |
| `preserve_unknown_attrs` | ❌ | ✅ | ❌ | ❌ | ✅ | **None — deprecated** |
| `drop_presentation_attrs` | ✅ | ❌ | ✅ | ✅ | ❌ | **None — deprecated** |
| `drop_interactive_shell` | ❌ | ❌ | ✅ | ❌ | ❌ | Drops shell elements |
| `unwrap_unknown_wrappers` | ❌ | ❌ | ✅ | ✅ | ❌ | Unwraps wrapper elements |

Because the five deprecated fields have no effect, **`Balanced`, `Strict`,
and `Preserve` currently produce byte-identical output** — they differ
from each other only in these fields' defaults. See
[Conversion Modes](./modes.md) for what this means when choosing a mode.

## Field Reference

### `mode`
The [ConversionMode](./modes.md) this options object was built from.
Changing `mode` after construction does not re-apply mode defaults
to the other fields — use `for_mode()` again instead.

### `preserve_ids`
Whether to emit an anchor for elements carrying a non-empty `id`
attribute. When enabled, `<h2 id="install">Install</h2>` produces:

```markdown
## <a id="install"></a>Install
```

The anchor is the element's **leading content**, placed after any
heading marker, list marker, or blockquote prefix:

| Input | Output |
|---|---|
| `<h2 id="x">Text</h2>` | `## <a id="x"></a>Text` |
| `<li id="x">Text</li>` | `- <a id="x"></a>Text` |
| `<p id="x">Text</p>` inside a `<blockquote>` | `> <a id="x"></a>Text` |

**Exception: `<a>` and `<pre>`.** These two elements open their own
inline-link capture or code-fence region as part of entering them, so
their anchor is emitted *before* the element instead, to avoid disturbing
the link text or code content:

| Input | Output |
|---|---|
| `<a id="x" href="/">text</a>` | `<a id="x"></a>[text](/)` |
| `<pre id="x"><code>y</code></pre>` | `<a id="x"></a>` on its own line, then the fenced block |

An `id` on a **descendant** of a link or a code block is deliberately
**not** emitted — an anchor injected into captured link text or into
literal code content would corrupt it. `<a href="/"><span id="s">Home</span></a>`
produces `[Home](/)` with no anchor for `s`.

The `id` value is escaped for HTML attribute context (`&` → `&amp;`,
`"` → `&quot;`) before being written — this is the one place mdka
constructs new HTML from an input-derived value, rather than passing
existing markup through.

An empty `id=""` emits nothing. `preserve_ids = false` emits nothing
regardless of `id`.

### `preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs`
**No effect on output. Deprecated since `2.2.0`.** Markdown has no syntax
for HTML attributes, so "preserve" or "drop" an attribute was never
expressible in the output — these fields described behaviour the format
could not represent, and never changed a single byte of Markdown in any
released version. See
[RFC 005](https://github.com/nabbisen/mdka-rs/blob/main/rfcs/done/005-conversion-options-semantics.md)
for the analysis. If you are currently setting any of these, nothing
changes: they remain present on the struct and accept any value, they
simply do nothing.

Attribute preservation is a legitimate feature some Markdown flavours
(Pandoc, kramdown) support. If mdka adds it, it will be a new,
deliberately designed feature — not a repair of these fields.

### `drop_interactive_shell`
Whether to remove `<nav>`, `<header>`, `<footer>`, and `<aside>` elements
**and all their children**.
Useful for content extraction from full web pages.
Enabled by default in `Minimal`; disabled by default in every other mode.

### `unwrap_unknown_wrappers`
Whether to replace `<div>`, `<span>`, `<section>`, `<article>`, and
`<main>` with their children, discarding the wrapper tag itself, when
`unwrap_unknown_wrappers` is enabled. Enabled in `Minimal` and `Semantic`.

**`<figure>` and `<figcaption>` are never unwrapped**, in any mode — see
the [Block Elements table](./elements.md) for why they're excluded even
though they visually resemble the other wrapper elements.
