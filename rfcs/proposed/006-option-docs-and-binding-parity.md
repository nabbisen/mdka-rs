# RFC 006 — Option documentation and binding parity

**Status.** Proposed
**Tracks.** M2 · Truth in the API surface. Closes the milestone.
**Depends on.** [RFC 005](./005-conversion-options-semantics.md) — complete
**Touches.** `docs/src/api/options.md`, `modes.md`, `elements.md`; `cli/src/main.rs`, `node/src/lib.rs`, `python/src/lib.rs`; `src/renderer.rs` comments and one test.
**Prepared.** 2026-08-12

## Summary

RFC 005 made the option **code** honest. The documented and exposed surface is
still not. This RFC closes that gap.

## The measured facts

All verified against source on 2026-08-12, not read from documentation.

### 1. The docs describe machinery that does not exist

`docs/src/api/options.md` describes options acting on a **pre-processed DOM**:

> `preserve_ids` — "Whether to keep `id="…"` attributes in the pre-processed DOM."
> `drop_presentation_attrs` — "Whether to remove `style` … during pre-processing."

The preprocessor was never wired in and was deleted under RFC 004. The
architecture parses once and traverses once. There is no pre-processed DOM to
keep anything in.

One entry is affirmatively false rather than merely stale:

> `preserve_aria_attrs` — "The attributes themselves do not appear in Markdown
> output, but they are used by the `Semantic` mode's conversion logic."

`preserve_aria_attrs` is read nowhere in `src/`. Semantic mode's logic does not
consult it. RFC 005 deprecated it as a no-op.

### 2. The mode docs promise differentiation that measurement disproved

`modes.md` gives `Strict` the goal *"Preserve as much of the original HTML
information as possible"* and offers `Preserve` for *"round-trip fidelity."*

RFC 005 Slice A measured all five modes across the option matrix: **Balanced,
Strict and Preserve are byte-identical.** They differ only in the defaults of
five fields that do nothing.

A user choosing `Strict` for "diff-friendly output" gets exactly what `Balanced`
gives them.

### 3. The bindings expose exactly the wrong subset

| Field | Works? | CLI | Node | Python |
|---|---|---|---|---|
| `preserve_ids` | ✅ anchors, since 2.2.0 | ✅ | ✅ | ✅ |
| `drop_interactive_shell` | ✅ | ✅ | ✅ | ✅ |
| **`unwrap_unknown_wrappers`** | ✅ | ❌ | ❌ | ❌ |
| `preserve_classes` | ❌ no-op | ✅ | ✅ | ✅ |
| `preserve_data_attrs` | ❌ no-op | ✅ | ✅ | ✅ |
| `preserve_aria_attrs` | ❌ no-op | ✅ | ✅ | ✅ |

`unwrap_unknown_wrappers` — one of only two fields that ever worked — appears
**zero times** across `cli/src`, `node/src` and `python/src`. Meanwhile all three
bindings expose three fields that do nothing.

Every binding user has been offered the inert options and denied the working one.

### 4. Carried finding — `figure` / `figcaption`

`docs/src/api/elements.md`'s Block Elements table groups `<div>`, `<article>`,
`<section>`, `<main>`, `<figure>`, `<figcaption>` into one row claiming all six
are unwrapped in Minimal/Semantic. **False for `<figure>` and `<figcaption>`** —
`is_wrapper_tag` excludes them and `is_structural_tag` explicitly includes them,
which blocks unwrapping regardless. Recorded at RFC 003 review.

## Goals

- Every documented option describes what the code does.
- Mode documentation reflects measured output, including where modes coincide.
- Every working field is reachable from every binding.
- No documented behaviour survives that no test demonstrates.

## Non-goals

- **Removing the deprecated fields from the bindings.** That is breaking for
  npm and PyPI consumers and no major version is planned. They stay, documented
  as no-ops.
- Reviving attribute preservation. RFC 005 settled that; if wanted it is its own
  feature RFC.
- Japanese comments in `cli/src/main.rs` — RFC 007 and RFC 013.
- Table support — RFC 008.

## Design decisions

### The mode table stays, with an effect column

`options.md`'s "Field Defaults by Mode" table is accurate about defaults and
misleading about consequences. Keep it — the defaults are real — and mark which
rows have no effect, so the table stops implying five axes of behaviour that do
not exist.

### Modes that coincide say so

`modes.md` must state plainly that Balanced, Strict and Preserve currently
produce identical output, and why. This is a documentation change only; it does
not merge the modes. They remain distinct API surface and may diverge later.

Not doing this leaves users choosing between three names for one behaviour.

### `unwrap_unknown_wrappers` is added to all three bindings

Additive, no compatibility cost. CLI gets a flag, Node an object key, Python a
keyword argument, each following the naming already used in that binding.

### Deprecated fields keep working and start saying so

`#[deprecated]` does not cross the FFI boundary — a Node or Python caller sees
nothing. Node and Python should emit a **runtime deprecation warning** when one
of the three no-op fields is explicitly passed, using each ecosystem's normal
mechanism. Passing nothing must stay silent.

The CLI should mark the three flags deprecated in `--help`.

### Renderer drift guard — carried from RFC 005

`enter_element` decides anchor placement with
`anchor_before = matches!(tag, "a" | "pre")`, duplicating knowledge that lives at
`renderer.rs:267` (`in_pre = true`) and `:315` (`capture_depth += 1`). A third
mutation site added later would silently drop anchors — the bug class that bit
twice during RFC 005.

Wanted: a comment at each mutation site, and a test asserting every specially
handled tag emits an anchor for a non-empty `id`. Small, and it closes a defect
class rather than an instance.

## Slices

| Slice | Content | Blocked on |
|---|---|---|
| **A** | `options.md` — rewrite the field reference and annotate the defaults table | — |
| **B** | `modes.md` — describe measured behaviour, including the three-mode identity; `elements.md` — fix the `figure`/`figcaption` row | — |
| **C** | Binding parity — add `unwrap_unknown_wrappers` everywhere; deprecation warnings | — |
| **D** | Renderer drift guard | — |

All four are independent. A and B are documentation only.

## Compatibility

Slices A, B, D: none.

Slice C is additive — one new option per binding, plus warnings on explicitly
passed deprecated fields. No signature breaks. The warnings are new output on
stderr for callers who pass those fields, which is the intent.

## Testing and verification

**Standing rule: assert on real output.** Every documentation claim this RFC
introduces must correspond to something a test demonstrates or that was run and
recorded in the review request. The failure this milestone exists to repair is
documentation asserted from intent rather than measurement.

Specifically: the claim that Balanced, Strict and Preserve are identical must be
backed by the Slice A characterisation suite, not restated from this RFC.

## Acceptance criteria

1. No option description references a pre-processed DOM or any deleted machinery.
2. Every no-op field is marked as such in the field reference **and** the defaults table.
3. `modes.md` states which modes currently coincide, backed by tests.
4. The `figure`/`figcaption` row is correct.
5. `unwrap_unknown_wrappers` is reachable from CLI, Node and Python, with a test per binding.
6. Explicitly passing a deprecated field warns in Node and Python; passing nothing is silent.
7. A comment at both guard-mutation sites, and a test that fails if a new one appears.
8. No field removed from any surface.

## Risks

| Risk | Mitigation |
|---|---|
| Documenting "three modes are identical" reads as an invitation to remove two | State explicitly that they remain distinct API and may diverge; this is a description of today, not a deprecation. |
| Deprecation warnings become noise in normal use | Warn only when the field is *explicitly passed*, never on defaults. |
| Adding a CLI flag conflicts with RFC 007/013's comment work | Different lines; sequence C after those if they are in flight. |
| The drift test entrenches the current tag list rather than detecting change | It must assert the observable (an anchor is emitted), not the list itself. |

## After this lands

M2's exit criterion — every public option demonstrably does what it says, on
every surface — is met, and M2 closes.
