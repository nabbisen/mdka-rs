# RFC 005 — `ConversionOptions` semantics

**Status.** Proposed
**Tracks.** M2 · Truth in the API surface. The milestone's anchor RFC.
**Touches.** `src/options.rs`, `src/traversal.rs`, `src/renderer.rs`, new tests.
Binding and documentation realignment is RFC 006's.
**Depends on.** RFC 004 (the harvested inventory).

## Summary

Six of the eight `ConversionOptions` fields have no effect on output. This RFC
establishes what they *should* mean in a Markdown-producing engine — a question
the original design never answered — and implements that.

It opens with a characterisation test suite that locks in today's behaviour.
That slice is unblocked and independent of the design decision.

## Motivation

### The measured facts

Verified repeatedly across this milestone's reviews:

- `preserve_ids`, `preserve_classes`, `preserve_data_attrs`,
  `preserve_aria_attrs`, `preserve_unknown_attrs`, and
  `drop_presentation_attrs` are read **nowhere** in `src/`.
- All five modes produce byte-identical output for attribute-bearing HTML.
  `<p id="x" class="c" data-k="v" aria-label="l" style="color:red" foo="bar">Hi</p>`
  → `"Hi\n"` in every mode, and flipping every flag changes nothing.
- Only `drop_interactive_shell` and `unwrap_unknown_wrappers` are live.

Yet all eight are public API, exposed through the CLI, the Node object, and the
Python signatures, and documented field-by-field.

### The finding that changes the shape of this RFC

RFC 004 harvested ten tests from the never-compiled preprocessor, on the premise
that they were the surviving specification of intended behaviour. Re-examined at
RFC 005 drafting, the recovered source shows:

```rust
pub fn preprocess(document: &Html, opts: &ConversionOptions) -> String
```

It returned a **filtered HTML string**, and its tests asserted against that
string — `assert!(!out.contains("class="))`, `assert!(out.contains("class=\"foo\""))`.

**Those are assertions about an intermediate HTML representation, not about
Markdown output.** The current architecture has no such intermediate; it parses
once and traverses once (RFC 003).

Two consequences:

1. **The inventory is evidence of intent, not a portable test suite.** Porting
   its assertions literally — expecting `class="foo"` in *Markdown* output —
   would require emitting raw HTML. That is a design decision, not a
   translation.
2. **These options could never have worked as documented.** Even with
   `preprocess()` wired in, `<p class="foo">Hi</p>` would have become the
   Markdown paragraph `Hi`; the renderer has no mechanism to carry attributes
   into Markdown. The options were not broken by the 2.0.0 rewrite. They were
   never expressible.

That is the real defect: **the option surface promises something Markdown cannot
represent, and no implementation choice was ever made about what to do instead.**

## Goals

- Every public option field demonstrably changes output, or is explicitly
  documented as unable to and deprecated.
- The choice is a recorded decision, not an accident of what was easy.
- Existing behaviour is characterised by tests *before* it changes, so the delta
  is visible rather than inferred.

## Non-goals

- Documentation and binding realignment — RFC 006.
- Table support — RFC 008.
- Comment handling. `Preserve` mode's documented intent to keep HTML comments is
  equally unimplemented, but it is a distinct question and belongs with RFC 006's
  documentation pass or its own RFC.
- Changing `drop_interactive_shell` or `unwrap_unknown_wrappers`, which work.

## The design question

Markdown has no attribute syntax. So "preserve `class`" must mean one of:

| # | Interpretation | Consequence |
|---|---|---|
| **1** | **Raw-HTML passthrough.** An element carrying preserved attributes is emitted as literal HTML instead of converted. | Honours the options fully. But CommonMark suppresses Markdown parsing inside HTML *blocks*, so preserving one attribute on a `<div>` can cascade its whole subtree to HTML. `Strict`/`Preserve` output stops being recognisably Markdown. |
| **2** | **Passthrough only where Markdown has no representation** — `span`, non-unwrapped `div`, unknown tags. Elements with a Markdown form (`p`, `h1`, `a`, `img`, `code`) drop attributes, documented as a limitation. | Modes differ visibly and usefully (`<span class="hl">x</span>` survives under `Strict`, vanishes under `Balanced`) without output ceasing to be Markdown. Does not satisfy a literal reading of today's docs. |
| **3** | **Implement only what Markdown can express** — `preserve_ids` as emitted anchors — and deprecate the other five as documented no-ops. | Smallest, most honest. Reduces the advertised surface; deprecation is a compatibility change. |

**Recommendation: 2, plus `preserve_ids` as anchors from 3.**

Rationale: 1 makes `Strict` and `Preserve` emit HTML documents wearing a `.md`
extension, which serves no stated use case — "debugging" and "archiving" are
better served by keeping the original HTML. 3 discards a feature the project has
advertised for its whole 2.x life. 2 gives every field a real, demonstrable
effect while keeping output Markdown, and `preserve_ids` via anchors is the one
attribute with genuine value in a Markdown world (anchor links).

**This choice is a product decision about what mdka is for, and it changes
`Strict`/`Preserve` output for existing users.** It is therefore raised as a
decision request to the project owner rather than settled here — see
`.git-exclude/review-request/005-conversion-semantics/`.

## Slices

### Slice A — Characterisation tests (unblocked, do first)

Lock in today's behaviour before changing it. No design decision required.

A test matrix over: five modes × the eight option fields × representative
element classes (block with Markdown form, inline with Markdown form, `span`,
`div`, unknown tag, void element, attribute-only differences).

Every case asserts the **current** output, whatever it is — including the many
cases that are currently identical across modes. Where a test documents
behaviour that the chosen design will later change, mark it so the diff is
legible when it changes.

This is genuinely useful independent of the outcome:

- It is the evidence base RFC 006 needs to document what modes actually do.
- It makes the eventual behaviour change reviewable as a test diff rather than
  an assertion about intent.
- It will likely surface further inert or surprising behaviour.

### Slice B — Implement the agreed semantics

Blocked on the decision. Scope depends on which interpretation is chosen.

### Slice C — Per-field, per-surface proof

One test per option field demonstrating it changes output, per M2's exit
criterion. Blocked on Slice B.

## Compatibility

Slice A: none — tests only.

Slice B under interpretation 2: `Strict` and `Preserve` output changes for
input carrying attributes on elements without a Markdown representation. Under
the project's release policy this is a minor-version change (`2.2.0`), additive
in the sense that no *documented* behaviour is removed — but real output does
change, and that is the owner's call to accept.

Under interpretation 3: deprecating five public fields is a larger compatibility
event and would need its own migration note.

## Testing and verification

Standing rule, learned from `verify-ci`: **verification must run in an
environment resembling the target.** For this RFC that means asserting on real
`html_to_markdown_with` output, never on an intermediate representation — the
precise mistake the deleted preprocessor's own test suite made, which is why
those tests passed while the feature did not exist.

## Acceptance criteria

1. Slice A: a characterisation suite covering five modes × eight fields ×
   the representative element classes, asserting current behaviour.
2. Slice A: any further inert or surprising behaviour found is reported.
3. Slice B: every option field demonstrably changes output, or is deprecated
   with a recorded reason.
4. Slice C: one proof test per field.
5. No assertion anywhere depends on an intermediate representation.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Characterisation tests entrench behaviour nobody wants | Slice B fights its own tests | Slice A marks cases expected to change; the diff is the point, not the baseline |
| Interpretation 2's boundary ("has a Markdown form") is fuzzy at the edges | Inconsistent behaviour | Slice B must enumerate the classification explicitly in `src/utils.rs`, as `is_skip_tag` and friends already do |
| Output change surprises downstream users | Complaints on a patch-shaped release | Minor version, CHANGELOG entry, and RFC 006 documents the new semantics |

## Alternatives considered

| Option | Assessment |
|---|---|
| **Port the harvested assertions directly** | Rejected — they assert on a representation that no longer exists. Doing so would silently choose interpretation 1 without deciding it. |
| **Remove the six fields outright** | A breaking change requiring a major version, which the roadmap explicitly does not plan. Interpretation 3's deprecation is the softer form of this. |
| **Leave them inert and document them as such** | Rejected. The whole milestone exists because documented behaviour that does not exist is the project's most persistent defect class. |

---

## Decision recorded — Option 3, 2026-08-08

**Project owner decision: Option 3.** Implement `preserve_ids` as emitted
anchors; deprecate the other five as documented no-ops.

Slice A's evidence removed the main objection. Balanced, Strict and Preserve are
already byte-identical across the matrix, so Option 3 does not collapse three
modes into one — they are already collapsed. It documents reality rather than
changing behaviour.

Attribute preservation remains a legitimate feature. Some Markdown flavours
(Pandoc, kramdown) do have attribute syntax. If it is ever wanted it should be
**its own feature RFC**, designed deliberately, not folded into repairing a
defect.

## Two consequences the RFC did not anticipate

Found while preparing the Slices B/C handoff.

### 1. Deprecation breaks the workspace build — scope amendment

`#[deprecated]` on the five fields fires wherever they are *set*, and they are
set in four crates:

| Crate | Sites |
|---|---|
| `src/options.rs` | the five `for_mode` presets |
| `cli/src/main.rs` | flag handling |
| `node/src/lib.rs` | `to_rust_opts` |
| `python/src/lib.rs` | keyword-argument plumbing |

Under `clippy -D warnings`, that fails CI across the workspace.

**Scope amendment:** Slice B must add `#[allow(deprecated)]` at those call sites,
which means touching `cli/`, `node/` and `python/` — files this RFC originally
assigned wholly to RFC 006.

The touch is deliberately minimal: attributes only, no signature or behaviour
change. RFC 006 still owns binding **parity** and documentation. Without this,
Slice B cannot land green.

### 2. Anchor emission is a new HTML injection surface — security

`preserve_ids` introduces the engine's first emission of an **attribute value
into raw HTML**. Today the engine emits input-derived values only into Markdown
link and image syntax, never into HTML attribute context.

An `id` such as `x" onload="alert(1)` would, unescaped, produce:

```html
<a id="x" onload="alert(1)"></a>
```

— an injected attribute in output that downstream renderers may render. mdka
documents that it does not sanitise HTML (SEC-005), but that non-goal covers
*passing through* what was already there; it does not license **constructing**
new HTML from untrusted values.

**Requirement:** the emitted `id` value must be escaped for attribute context —
at minimum `&` → `&amp;` and `"` → `&quot;`. Required test cases in the Slice B/C
handoff.

This is a genuine new attack surface and the reason anchor emission needs more
care than its one-line appearance suggests.

### 3. Anchor placement — correction after Slice B1 landed

Recorded 2026-08-12, after reviewing the Slices B/C submission.

Slice B1 emitted the anchor **before** the element, as its own block. That
satisfies a heading and a paragraph in isolation. It fails in two common
structures:

- **Lists.** An anchor line between two `<li>`s is a CommonMark *lazy
  continuation* of the preceding item's paragraph, so the anchor renders inside
  the **wrong item**.
- **Blockquotes.** An anchor emitted for a `<p>` inside a `<blockquote>` lands
  outside the quote, unprefixed, because `push_raw` bypasses
  `emit_pending_prefix()`.

**Corrected rule: the anchor is leading *content* of the element, emitted after
any prefix or marker** — `## <a id="x"></a>Install`, `- <a id="b"></a>two`,
`> <a id="p"></a>Q`. This supersedes the placement in the Slices B/C handoff §5,
including its heading and paragraph expectations.

The cause of the miss is worth recording, because it is not the implementer's:
the handoff's required-cases table listed a heading, a paragraph, an inline span
and two negative cases, and **no structural context at all** — no list, no
blockquote, no nesting. The implementation met the contract exactly as written.

**A required-cases table is a specification, and an incomplete one is a defect in
the specification.** For any change to the renderer, the table must cover block
elements, list items, blockquote descendants and nesting, or it is not a contract
— it is a sample.

Corrected in
[`slice-b1-placement-correction-handoff.md`](../handoffs/005-conversion-options-semantics/slice-b1-placement-correction-handoff.md).
