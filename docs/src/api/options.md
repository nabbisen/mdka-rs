# ConversionOptions

```rust
pub struct ConversionOptions {
    pub mode: ConversionMode,

    // Attribute retention
    pub preserve_ids:             bool,
    pub preserve_classes:         bool,
    pub preserve_data_attrs:      bool,
    pub preserve_aria_attrs:      bool,
    pub preserve_unknown_attrs:   bool,

    // Pre-processing behaviour
    pub drop_presentation_attrs:  bool,
    pub drop_interactive_shell:   bool,
    pub unwrap_unknown_wrappers:  bool,
}
```

`ConversionOptions` controls every detail of the pre-processing pipeline.
You rarely need to set individual fields — start with a mode and override
only what differs from the default for that mode.

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
opts.preserve_ids           = false; // don't keep id= attributes
opts.preserve_aria_attrs    = true;  // (already true in Balanced, shown for clarity)
```

### Default

```rust
let opts = ConversionOptions::default(); // equivalent to for_mode(Balanced)
```

## Field Defaults by Mode

| Field | Balanced | Strict | Minimal | Semantic | Preserve |
|---|---|---|---|---|---|
| `preserve_ids` | ✅ | ✅ | ❌ | ✅ | ✅ |
| `preserve_classes` | ❌ | ✅ | ❌ | ❌ | ✅ |
| `preserve_data_attrs` | ❌ | ✅ | ❌ | ❌ | ✅ |
| `preserve_aria_attrs` | ✅ | ✅ | ❌ | ✅ | ✅ |
| `preserve_unknown_attrs` | ❌ | ✅ | ❌ | ❌ | ✅ |
| `drop_presentation_attrs` | ✅ | ❌ | ✅ | ✅ | ❌ |
| `drop_interactive_shell` | ❌ | ❌ | ✅ | ❌ | ❌ |
| `unwrap_unknown_wrappers` | ❌ | ❌ | ✅ | ✅ | ❌ |

## Field Reference

### `mode`
The [ConversionMode](./modes.md) this options object was built from.
Changing `mode` after construction does not re-apply mode defaults
to the other fields — use `for_mode()` again instead.

### `preserve_ids`
Whether to keep `id="…"` attributes in the pre-processed DOM.
Useful when the output is rendered in a context that relies on
anchor links (`#section-name`).

### `preserve_classes`
Whether to keep `class="…"` attributes.
Rarely useful in Markdown output, but can help when feeding the
Markdown back into an HTML renderer that applies CSS.

### `preserve_data_attrs`
Whether to keep `data-*` custom attributes.
Mostly relevant for `Strict` and `Preserve` modes.

### `preserve_aria_attrs`
Whether to keep `aria-*` accessibility attributes.
Enabled by default in `Balanced`, `Strict`, `Semantic`, and `Preserve`.
The attributes themselves do not appear in Markdown output, but they
are used by the `Semantic` mode's conversion logic.

### `preserve_unknown_attrs`
Whether to keep attributes not otherwise handled (everything except
`href`, `src`, `alt`, `title`, `aria-*`, `data-*`, `id`, `class`, `style`).

### `drop_presentation_attrs`
Whether to remove `style` and other purely visual attributes during pre-processing.
Enabled in `Balanced`, `Minimal`, and `Semantic`.

### `drop_interactive_shell`
Whether to remove `<nav>`, `<header>`, `<footer>`, and `<aside>` elements
**and all their children**.
Useful for content extraction from full web pages.
Disabled by default in all modes; opt in explicitly.

### `unwrap_unknown_wrappers`
Whether to replace generic container elements (`<div>`, `<span>`,
`<section>`, `<article>`, `<main>`) with their children when they
carry no structural meaning. Enabled in `Minimal` and `Semantic`.
