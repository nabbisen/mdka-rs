# RFC 004 — Orphaned preprocessor disposition

**Status.** Implemented (2.1.7)
**Tracks.** M1 · Trustworthy baseline. Resolves inherited technical debt before
RFC 005 builds on the same ground.
**Touches.** `tests/utils/preprocessor.rs`, `tests/utils/preprocessor/tests.rs`,
the commented-out block at `src/utils.rs:78-99`.

## Summary

A standalone, mode-aware DOM preprocessor exists in the repository but is never
compiled, never executed, and not reachable from any target. This RFC deletes
it, after first harvesting its test cases into the RFC 005 test plan.

## Motivation

The handoff bundle recorded this as gap G-01 and risk RISK-002: `src/preprocessor.rs`
implemented attribute filtering, shell dropping, and wrapper unwrapping, while
the engine performed its own preprocessing inline in `traversal.rs`. Two
implementations of one concept, with drift risk, and five dead-code warnings.

Git history corrects that account, and the corrected version is worse.
`src/preprocessor.rs` never existed on `main`. It lived only on the pre-squash
`2.0.0 dev` branch. When that branch landed as squash commit `e7b2dbd`
(122 files, ~47k insertions), the code arrived already at
`tests/utils/preprocessor.rs`. The blob hash is identical (`c6444907`) at tag
`2.0.0` and at tag `2.1.6`.

So this code has been dead since **2.0.0 shipped on 2026-04-15** — across all
eleven releases of the 2.x line, roughly four months — and it was never once
compiled in a published version. The handoff bundle's description of a
`src/preprocessor.rs` emitting five dead-code warnings describes the branch
state before the squash, not anything that ever shipped.

The debt is therefore not a recent regression to be tidied. It is a structural
defect that no gate has ever been able to see:

- No test target declares `mod utils`. Cargo compiles `tests/*.rs` as targets;
  `tests/utils/` is a subdirectory and is only built if some target declares it.
  Only `tests/common.rs` is declared, by `block_elements`, `compat`,
  `inline_elements`, and `robustness`.
- Confirmed by binary enumeration, not only by source inspection:
  `cargo test --workspace --no-run` produces exactly ten executables — six
  `tests/*.rs` targets plus four unit-test binaries — and none for
  `tests/utils/`. There is no `[[test]]` entry in any manifest wiring it in.
- `tests/utils/preprocessor/tests.rs` uses `use crate::options::…` and
  `utils::preprocessor::preprocess(…)` — paths that only resolve inside the
  library crate. The file could not compile where it now sits even if declared.
- Net effect: **227 lines of implementation and 115 lines of tests are dead**,
  invisible to the compiler, and absent from the 74-test baseline.

No compiler has ever warned about this code in any shipped version, because no
compiler has ever seen it. That is why it survived eleven releases: there was
nothing to notice. RFC 001's gate would not have caught it either — a gate only
checks what compiles.

`src/utils.rs:78-99` additionally carries `is_void_element` commented out with
a `// moved to /tests/utils/preprocessor.rs` note, leaving a dangling reference.

## Goals

- One preprocessing implementation in the project, not two.
- No unreachable code masquerading as tests.
- The behavioural intent encoded in those 115 lines of tests is preserved,
  in the place where it will actually run.

## Non-goals

- Implementing attribute filtering in the engine. That is RFC 005; this RFC
  only ensures RFC 005 starts from one implementation and a captured test
  intent, not from two competing code paths.
- Changing any public API. Nothing being removed is reachable from any public
  surface.

## Proposed design

### Decision: delete, after harvesting

Remove `tests/utils/preprocessor.rs`, `tests/utils/preprocessor/tests.rs`, and
the commented-out `is_void_element` block in `src/utils.rs`.

Before deletion, transcribe every distinct behavioural assertion in
`tests/utils/preprocessor/tests.rs` into a test-case inventory recorded in this
RFC's implementation notes, to be consumed by RFC 005. Cases observed include
script dropping regardless of mode, and `class`/`style` removal under Balanced.
The inventory must be complete, not a sample.

### Rationale

RFC 005 will implement attribute handling **inside the traversal**, because that
is where the engine's single-pass architecture (DEC-002, T-01) puts it.
Retaining a standalone preprocessor would recreate exactly the two-implementation
drift the handoff warned about, and the retained copy has already proven it can
rot silently for a full release cycle.

The design record is not lost by deleting the code: this RFC, the handoff
bundle, and git history all retain it. RFC 000's "never delete an RFC" principle
governs design documents, not superseded implementation code.

### Alternatives considered

| Option | Assessment |
|---|---|
| **Restore to `src/` with `#[allow(dead_code)]`** | Rejected. Keeps two implementations, and an explicit `allow` makes the drift permanent and sanctioned rather than temporary. |
| **Restore and expose `preprocess()` as public API** | Rejected. Widens the public surface with a second, differently-behaving conversion path, during the same milestone sequence in which we are trying to make the option surface truthful. |
| **Restore and wire into the engine** | Rejected. Reintroduces the parse → filter → serialise → re-parse pipeline that the current architecture deliberately removed, at direct cost to the performance positioning. |
| **Leave as-is** | Rejected. Dead, uncompiled code that the compiler cannot warn about is worse than dead code it can. |

## Compatibility

None. No public API, no persistent format, no behaviour change. The removed code
is not reachable from `mdka`, `mdka-cli`, `mdka-node`, or `mdka-python`.

## Security

Neutral, marginally positive: removes an unmaintained DOM-manipulation code path
that would otherwise be a candidate for future reuse without review.

## Testing and verification

- `cargo test --workspace` must still report **74 passed, 0 failed**. The count
  must not change, since the deleted tests were never running. A changed count
  means something else moved and requires explanation.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- `grep -rn "preprocessor\|is_void_element" src/ tests/ cli/ node/ python/`
  returns no live reference. Documentation references are handled by RFC 003.
- `cargo test --workspace --no-run` still lists ten executables. Use binary
  enumeration rather than source inspection as the check — it is what
  established the problem, and it is the only method that distinguishes
  "declared but empty" from "not compiled at all".

## Acceptance criteria

1. `tests/utils/` is removed in full.
2. The commented-out `is_void_element` block is removed from `src/utils.rs`.
3. A complete test-case inventory from the deleted test file is recorded for RFC 005 consumption.
4. `cargo test --workspace` reports 74 passed, 0 failed.
5. No source file references the removed module.
6. Documentation still referencing it is left alone — RFC 003 owns that edit.

## Prohibited shortcuts

- Do not delete the test file without first transcribing its assertions. Those
  cases are the only surviving specification of intended attribute behaviour,
  and RFC 005 depends on them.
- Do not "fix" the module by wiring it into a test target. That would add 115
  tests asserting behaviour the engine does not implement, and they would fail.
- Do not touch `docs/` under this RFC.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Test intent lost on deletion | RFC 005 reimplements attribute rules from scratch and diverges from original intent | Harvest inventory is a blocking acceptance criterion, verified at review |
| Deleted code turns out to be wanted later | Rework | Recoverable from git history; this RFC records why it went |

## Harvested test inventory

Transcribed 2026-08-02, before deletion, from `tests/utils/preprocessor/tests.rs`
(115 lines, 10 `#[test]` functions — all ten are recorded below; this is not a
sample). "Currently implemented?" is checked directly against `src/traversal.rs`
and `src/renderer.rs` as they stand today, not against the deleted preprocessor.

| # | Test name | Input HTML | Mode / options exercised | Asserted behaviour | Currently implemented? |
|---|---|---|---|---|---|
| 1 | `test_script_always_dropped` | `<p>Text</p><script>alert(1)</script>` | Strict | `<script>` content never appears in output, regardless of mode; surrounding text survives | **Yes.** `utils::is_skip_tag` (shared with the live `traversal.rs`) already includes `"script"`; the live engine drops script content unconditionally today. |
| 2 | `test_balanced_drops_class_style` | `<p class="foo" style="color:red">Hi</p>` | Balanced | Neither `class=` nor `style=` appears in output; `"Hi"` survives | **No, only vacuously.** The live renderer has no code path that emits any raw HTML attribute into Markdown output, in any mode. The assertion happens to hold today, but not because `drop_presentation_attrs` / `preserve_classes` gate anything — there is nothing to gate. |
| 3 | `test_strict_keeps_class_data` | `<p class="foo" data-x="1">Hi</p>` | Strict | `class="foo"` and `data-x="1"` literally appear in output | **No.** Same reason as #2, in the other direction: the live renderer never emits `class=` or `data-*=` regardless of mode, so this assertion would fail against the live engine. `preserve_classes` and `preserve_data_attrs` are read nowhere in `src/`. |
| 4 | `test_minimal_drops_shell` | `<nav><a href='/'>Home</a></nav><main><p>Content</p></main>` | Minimal | `<nav>` and its content are dropped; `"Content"` survives | **Yes.** `opts.drop_interactive_shell` (set by the `Minimal` preset) combined with `utils::is_shell_tag` is live in `traversal.rs`, functionally identical to the deleted preprocessor's check. |
| 5 | `test_semantic_keeps_aria` | `<button aria-label="close" class="btn">X</button>` | Semantic | `aria-label=` present; `class=` absent | **Split.** The "`class=` absent" half holds vacuously (see #2). The "`aria-label=` present" half does **not** hold — the live renderer has no path that emits `aria-*` attributes; `preserve_aria_attrs` is read nowhere in `src/`. |
| 6 | `test_preserve_keeps_comments` | `<!-- note --><p>Text</p>` | Preserve | The literal string `<!-- note -->` appears in output | **No.** `traversal.rs` has no `Node::Comment` arm; comments fall into the catch-all branch (which only walks children) and are silently, unconditionally dropped in every mode today. This directly contradicts the assertion for `Preserve` mode. |
| 7 | `test_balanced_drops_comment` | `<!-- note --><p>Text</p>` | Balanced | Comment absent from output | **Yes, only vacuously.** Comments are unconditionally dropped today (see #6), in every mode — not specifically because `Balanced` chose to drop them. |
| 8 | `test_href_always_preserved` | `<a href="https://example.com">Link</a>`, checked across all five modes | Balanced, Minimal, Strict, Semantic, Preserve | `href=` (i.e. the Markdown link target) present regardless of mode | **Yes.** `renderer.rs`'s `a`-tag handling reads `href` unconditionally, with no option check, for every mode — matches the live engine exactly. |
| 9 | `test_code_lang_class_always_kept` | `<pre><code class="language-rust">fn main(){}</code></pre>` | Balanced | `language-rust` present in the output (as a fenced code-block language tag) | **Yes.** `renderer.rs`'s `code`-in-`pre` handling calls `utils::extract_code_lang` on the `class` attribute and emits the language in the fence header — live today, unconditional on mode. |
| 10 | `test_deep_nest_no_stack_overflow` | 10,000 nested `<div>` wrapping `<p>deep</p>` | default (Balanced) | No stack overflow; `"deep"` content survives | **Yes, equivalently.** The live `traversal.rs` is likewise a non-recursive `Vec`-stack DFS (NFR-02). `tests/robustness.rs::deep_nest_no_stack_overflow` already exercises the identical property against the live `html_to_markdown` engine, not against the deleted module. |

### Design elements captured beyond the ten tests

The deleted file's implementation (`preprocess()` / `emit_attrs()`) encodes design
intent that no `#[test]` directly exercises, but which is real signal for RFC 005:

- **A "semantic attributes, always preserved regardless of mode" allowlist:**
  `href`, `src`, `alt`, `title`, `lang`, `dir`, `type`, `start`, `colspan`,
  `rowspan`. In the dead code, these bypassed every option flag. In the live
  engine, `href`/`src`/`alt`/`title`/`start` are each already hard-coded into
  the specific tag handling that needs them (`a`, `img`, `ol`); `lang`, `dir`,
  `colspan`, `rowspan` are not read anywhere live today.
- **A void-element list** (`area base br col embed hr img input link meta
  param source track wbr`), used by the dead preprocessor to decide whether to
  self-close a tag. The live engine has no equivalent concept — `is_void_element`
  exists only as the commented-out block this RFC also removes (`src/utils.rs`
  lines 78–99). No live code currently classifies void elements at all.
- **Attribute-branch logic for the two fields no test exercises directly:**
  `preserve_ids` (`emit_attrs` had a dedicated `id` branch, but zero `#[test]`
  ever set an `id` attribute in its input HTML) and `preserve_unknown_attrs`
  (the dead code's fallback branch for attributes matching none of the named
  categories, likewise never exercised by name in any test).

### Coverage gaps in the harvested file itself

Two of the eight `ConversionOptions` fields, and one of the two currently-live
behavioral flags, have **no surviving test at all** — not merely "currently
failing," but never asserted in the first place, even by the dead code's own
suite:

- `preserve_ids` — implementation existed in `emit_attrs`; no test ever set an
  `id` attribute.
- `preserve_unknown_attrs` — implementation existed; no test ever used an
  attribute name that would hit that branch.
- `unwrap_unknown_wrappers` — this flag **is** live in `traversal.rs` today
  (wrapper-tag unwrapping works), but none of the ten harvested tests exercises
  it; there is no surviving specification of intended wrapper-unwrap behavior
  beyond the implementation itself.

RFC 005 starts these three from zero test coverage, not from an adapted case.

### Summary for RFC 005

Of the eight `ConversionOptions` fields: **`drop_interactive_shell` and
`unwrap_unknown_wrappers` are live and tested** (the latter only by the live
suite, not by the harvested file — see above). The other six
(`preserve_ids`, `preserve_classes`, `preserve_data_attrs`,
`preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs`) are
inert: not because any mode-specific logic suppresses them, but because the
live renderer has no mechanism at all for emitting arbitrary HTML attributes
into Markdown output. Comment handling (`Preserve` mode's intent to keep HTML
comments) is likewise entirely unimplemented, in every mode, today.
