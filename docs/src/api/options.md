# ConversionOptions

```rust,fragment
pub struct ConversionOptions {
    pub mode: ConversionMode,
    pub preserve_ids:           bool,
    pub drop_interactive_shell: bool,
    pub emphasis_from_style:    bool,
}
```

`ConversionOptions` controls the details of how mdka's single-pass DOM
traversal renders Markdown. There is no separate pre-processing stage —
the traversal in `src/traversal.rs` reads these fields directly as it
walks the parsed document once. You rarely need to set individual fields —
start with a mode and override only what differs from the default for
that mode.

**There are four fields, and three of them act on the output:** `preserve_ids`,
`drop_interactive_shell` and `emphasis_from_style`. Six more existed until 3.0 and
are described under [Removed in 3.0](#removed-in-30) below — none of them ever
changed a byte of output.

## Creating Options

### From a mode (recommended)

```rust
use mdka::options::{ConversionMode, ConversionOptions};

let opts = ConversionOptions::for_mode(ConversionMode::Minimal);
```

`for_mode` returns sensible defaults for the chosen mode. See the table below.

### Modify fields after creation

```rust,fragment
let mut opts = ConversionOptions::for_mode(ConversionMode::Balanced);
opts.drop_interactive_shell = true; // also strip nav/header/footer/aside
opts.preserve_ids           = false; // don't emit <a id="…"> anchors
```

### Default

```rust,fragment
let opts = ConversionOptions::default(); // equivalent to for_mode(Balanced)
```

## Field Defaults by Mode

| Field | Balanced | Minimal | Effect |
|---|---|---|---|
| `preserve_ids` | ✅ | ❌ | Emits anchors |
| `drop_interactive_shell` | ❌ | ✅ | Drops shell elements |
| `emphasis_from_style` | ❌ | ❌ | Lets inline `style` add bold/italic |

See [Conversion Modes](./modes.md) for what this means when choosing a mode.

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

### `drop_interactive_shell`
Whether to remove `<nav>`, `<header>`, `<footer>`, and `<aside>` elements
**and all their children**.
Useful for content extraction from full web pages.
Enabled by default in `Minimal`; disabled by default in every other mode.

### `emphasis_from_style`

**mdka reads the element, not the environment.** An inline `style` attribute
is part of the element it appears on — it arrived in the document mdka was
handed, and reading it is reading the input. A `class` is a reference to a
stylesheet mdka was never given; resolving it would mean inventing what
mdka cannot see.

- `<span style="font-weight:700">` **works** — the document itself says bold.
- `<span class="c7">` **never will**, whatever the stylesheet says — mdka does
  not read stylesheets, and this is permanent, not merely unimplemented.
- `<style>` blocks and linked stylesheets are out of scope for the same reason.

With this option **on** (default **off**), any element's inline `style` can
add bold and/or italic it would not otherwise carry, read independently:

| Property | Adds | Value grammar |
|---|---|---|
| `font-weight` | bold | `bold`, or a number ≥ 600 |
| `font-style` | italic | `italic` or `oblique` |

A value the grammar does not recognise (`bolder`/`lighter` — relative to a
parent's computed weight mdka does not track — a number strictly between
500 and 600, or anything unparseable) is read as "this element says
nothing", the same as no `style` at all: never an error, and never guessed
at.

**Turning this on can change output for any document that has such a
`style`.** That is why it is opt-in.

Independently of the option, and unaffected by it, a `font-weight` of
`normal` or ≤ 500 (or a `font-style` of `normal`) still **removes** the
emphasis a `<b>`/`<strong>`/`<i>`/`<em>` tag would otherwise carry on its
own — this shipped in `3.0.0`, before this option existed, most often seen
on a Google Docs paste's own un-bolding wrapper, and stays on with the
option off:

```html
<b style="font-weight:400">not actually bold</b>
```
→ `not actually bold` (no `**`), with `emphasis_from_style` either way.

**A style that only restates a tag's own default adds nothing, either.**
`b`/`strong`/`i`/`em` are not the only elements whose own rendering is
already bold or italic by the browser's default stylesheet:

- **Bold by default:** `b`, `strong`, `h1`–`h6`, `th`
- **Italic by default:** `i`, `em`, `cite`, `address`, `var`, `dfn`

For the wider set, the rule is the same one the negation case above already
relies on: a `style` saying only what the tag already means is not new
information, so it must not add a span the tag's own rendering never had:

```html
<h1 style="font-weight:700">Title</h1>
```
→ `# Title` (no `**`), with `emphasis_from_style` either way. A heading's
`#` already carries the whole meaning; `# **Title**` would say "a heading
containing bold text," which the source did not mean — there is no
markdown for "heading, and also bold," nor any need.

`cite`, `address`, `var` and `dfn` convert as plain text (see
[Block Elements](./elements.md)) independently of this option — restating
their default italic rendering via `style` does not change that:

```html
<cite style="font-style:italic">A Work</cite>
```
→ `A Work`, with `emphasis_from_style` either way. The same input converts
identically whether or not the source happened to inline the tag's own
default style — unlike the option's general case, output here does not
depend on which browser produced the clipboard HTML.

`th` is in the bold-default set for the same reason, even though table
cells never receive emphasis from this option regardless (see the table
boundary below): the set describes what the tag's own rendering already
means, not only the paths this option happens to reach today.

**An authored style nested inside one of these tags still works** — only
the outer tag's own restated default is ignored, not a genuinely distinct
declaration on a descendant:

```html
<h1 style="font-weight:700"><span style="font-weight:700">Title</span></h1>
```
→ `# **Title**` — the `<span>` is not a heading, and its own `style` still
adds emphasis.

A `style` that *removes* one of these tags' own default (`font-weight:400`
on a heading, say) has no visible effect, unlike the `<b>` case just above:
a heading's `#` never consults the emphasis-span state at all, so there is
no markdown for "un-bolding" a heading — nothing to remove, where `<b>` had
an actual span.

With the option on, a declaration on a container reaches **paragraphs and
headings among its descendants, including one nested inside a quote or a
list item** — not every descendant a browser would render bold. A `style`
on a `<div>` applies to the `<p>`s inside it, each opening and closing its
own bold:

```html
<div style="font-weight:700"><p>a</p><p>b</p></div>
```
→
```markdown
**a**

**b**
```

**Two boundaries worth knowing, both invisible in rendered HTML:**

- **Whether a list item or a quote is emphasised depends on whether its text
  is wrapped in `<p>`.** `<blockquote style="font-weight:700">a</blockquote>`
  stays plain (`> a`); `<blockquote style="font-weight:700"><p>a</p></blockquote>`
  is emphasised (`> **a**`). A browser renders both the same; mdka's rule is
  about the `Block` kind the traversal already builds, not about how the
  bare text would look on screen.
- **Table cells never receive it, even wrapped in `<p>`.**
  `<div style="font-weight:700"><table><tr><td><p>a</p></td></tr></table></div>`
  converts with `a` plain in its cell, same as without the `<p>`.

Neither is planned to widen: the option's decided cases are met, it is off
by default, and output with it off is byte-identical to `3.0.0`.

**In `Minimal` mode, the option does nothing for its own motivating case.**
`Minimal` unwraps `<span>`/`<div>`/`<section>`/`<article>`/`<main>` before
this option — or anything else — can read their `style`; a discarded tag
carries no declaration to read. The Google Docs paste shape this option was
built for:

```html
<b style="font-weight:normal" id="…"><span style="font-weight:700">bold</span> plain</b>
```

→ `**bold** plain` in `Balanced`, but plain `bold plain` in `Minimal` — the
`<span>` that carried the bold declaration is gone by the time anything
downstream could read it. **This follows from `Minimal`'s own wrapper
unwrapping, not from a limit of this option**: see
[Conversion Modes](./modes.md#minimal) for why. A reader converting Google
Docs pastes in `Minimal` should not expect this option to help.

## Removed in 3.0

Six fields were removed, along with the builder method `preserve_aria_attrs`. **None
of them ever changed the output**, in any released version, so removing them
changes nothing about how a document converts — only code that names them has to
change. Delete the assignment, the keyword argument or the flag.

| Removed | Where it existed | Why it could not act |
|---|---|---|
| `preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs` | Rust (all five). Node.js and Python had the first three; the CLI had `--preserve-classes`, `--preserve-data` and `--preserve-aria` | Markdown has no syntax for HTML attributes, so "preserve" or "drop" an attribute was never expressible in the output. Deprecated since `2.2.0`; see [RFC 005](https://github.com/nabbisen/mdka-rs/blob/main/rfcs/done/005-conversion-options-semantics.md) for the analysis |
| `unwrap_unknown_wrappers` | Rust; `unwrapUnknownWrappers` in Node.js; `unwrap_unknown_wrappers` in Python; `--unwrap-wrappers` on the CLI | Unwrapping a wrapper element (`<div>`, `<section>`, `<article>`, `<main>`) removes the tag but keeps the paragraph break it stood for, and Markdown has no wrapper element to show the difference. Deprecated since `2.9.0` |

**The two reasons are not the same.** The attribute options can never come back:
Markdown has no attribute syntax. `unwrap_unknown_wrappers` was inert because of
what the renderer leaves behind, so **if wrapper handling ever becomes
expressible it returns as a new option**, designed for what it can then do — not
as a repair of the old field. (`Minimal` still unwraps wrappers, internally; that
is a property of the mode, not something you can set.)

What you see if you still use one:

| Surface | What happens |
|---|---|
| Rust | Compile error: the field (and `ConversionOptions::preserve_aria_attrs`) no longer exists |
| Node.js | TypeScript: a compile error. **Plain JavaScript: nothing — the option is ignored, with no error and no warning** |
| Python | `TypeError: … got an unexpected keyword argument` |
| CLI | Exit status 1 and one line on stderr saying the flag was removed and what to do |

**`<figure>` and `<figcaption>` are never unwrapped**, in any mode — see
the [Block Elements table](./elements.md) for why they're excluded even
though they visually resemble the other wrapper elements.
