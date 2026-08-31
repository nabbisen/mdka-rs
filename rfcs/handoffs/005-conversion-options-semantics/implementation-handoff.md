# Developer Handoff — RFC 005 · Slice A only (characterisation tests)

**Governing RFC.** [RFC 005](../../done/005-conversion-options-semantics.md) — Proposed
**Milestone.** M2 · Truth in the API surface
**Scope of this Handoff.** **Slice A only.** Slices B and C are blocked on a
project-owner decision and are not covered here.
**Prepared.** 2026-08-04

This Handoff directs execution of RFC 005 Slice A. It does not redefine the RFC.
If implementation uncovers a conflict, stop and raise it.

---

## 1. Purpose

Lock in what the conversion engine does **today**, exhaustively, before RFC 005
changes it.

You are not fixing anything. You are writing down the truth so the eventual
change is visible as a test diff rather than argued from intent.

## 2. Why this comes first

Six of the eight `ConversionOptions` fields have no effect on output. Which six,
and in exactly which cases, is currently known only from spot checks scattered
across review documents.

There is a sharper reason. RFC 004's harvested inventory was collected on the
premise that it specified intended behaviour. Re-examined since, the deleted
`preprocess()` returned a **filtered HTML string**, and its tests asserted
against that string — not against Markdown. They passed for years while the
feature did not exist, because they tested the wrong thing.

**That is the mistake this slice exists to make impossible.** Every assertion
you write must go through the real public API and assert on real Markdown
output.

## 3. Change scope

| Path | Change |
|---|---|
| `tests/` | New characterisation test file(s) |

Nothing else. No `src/`, no manifests, no docs, no workflows.

## 4. Non-change scope — do not touch

- **`src/` in its entirety.** Not one line. If a test reveals a bug, that is a
  finding to report, not a licence to fix.
- `docs/` — RFC 006 owns it.
- The eight `ConversionOptions` fields — RFC 005 Slice B owns them.
- The existing six integration test files. Add new files; do not restructure
  what is there.
- Japanese comments — RFC 007 and RFC 013.

## 5. Required implementation

A characterisation matrix asserting **current** behaviour.

### Dimensions

**Modes:** `Balanced`, `Strict`, `Minimal`, `Semantic`, `Preserve`.

**Option fields:** all eight, each toggled independently from its mode default.

**Element classes** — at minimum one representative of each:

| Class | Example |
|---|---|
| Block with a Markdown form | `<p>`, `<h2>` |
| Inline with a Markdown form | `<a>`, `<code>`, `<strong>` |
| Inline without one | `<span>` |
| Generic block container | `<div>` |
| Unknown tag | `<custom-tag>` |
| Void element | `<br>`, `<hr>`, `<img>` |
| Always-skipped | `<script>`, `<svg>`, `<head>` |
| Shell element | `<nav>`, `<footer>` |

**Attributes** — carry `id`, `class`, `data-*`, `aria-*`, `style`, and an
unknown attribute, so every `preserve_*` field has something to act on.

### The rule that matters

**Assert what the engine actually produces, not what you think it should.**

Many cases will be identical across all five modes. That identity *is* the
finding — record it as an assertion, not as a comment saying "modes don't
differ here."

Where you believe the chosen design will later change a case, mark it clearly
(a naming convention, or a comment naming RFC 005 Slice B). When Slice B lands,
those are the tests expected to change, and everything else changing is a
regression.

### Structure

Follow the project convention: integration tests under `tests/`, split by
logical boundary, and split a file that grows past roughly 300 effective lines.
`tests/common.rs` already provides a `conv` helper; extend that pattern rather
than duplicating it.

## 6. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The existing **74 tests must still pass unchanged**. Your additions raise the
total; the pre-existing 74 must not move. If any pre-existing test changes
behaviour, stop — you have touched something you should not have.

## 7. What to report — this is the deliverable

The tests are the artifact; the findings are the value.

1. **The count**: how many option/mode combinations produce identical output,
   and how many differ. A concrete number, not "most".
2. **Which fields demonstrably do nothing**, confirmed by test rather than by
   reading `src/`.
3. **Anything surprising.** Places where modes differ that nobody expected, or
   where they do not differ and the documentation implies they should. Expect to
   find some — the review passes so far have only spot-checked.
4. **Any suspected bug**, reported and not fixed.

## 8. Prohibited shortcuts

- Do not assert on anything but `html_to_markdown` / `html_to_markdown_with`
  output. No internal functions, no intermediate representations. This is the
  precise failure the deleted preprocessor's suite made.
- Do not fix anything in `src/`, however small or obvious.
- Do not skip a combination because you expect it to be identical. The identity
  is the data.
- Do not write a test asserting what the docs claim. Assert what the engine does.

## 9. Known risks

| Risk | If it happens |
|---|---|
| The matrix is large and repetitive | Expected. Use table-driven tests; keep each case's input and expected output legible. |
| A case's current output looks plainly wrong | Record it as-is and report it. Slice B decides; you document. |
| A pre-existing test breaks | Stop and report. Nothing here should affect them. |

## 10. Required evidence

1. `cargo test --workspace --locked` — full output, with the pre-existing 74
   still passing and the new total stated.
2. `cargo fmt --check` and clippy, both exit 0.
3. `git diff --stat` — `tests/` only.
4. The four findings from §7.

## 11. Acceptance checklist

- [ ] Characterisation tests cover five modes × eight fields × the element classes in §5
- [ ] Every assertion goes through the public API against Markdown output
- [ ] Cases expected to change under Slice B are marked as such
- [ ] Pre-existing 74 tests unchanged
- [ ] fmt and clippy clean
- [ ] `src/` untouched
- [ ] §7's four findings reported, with concrete counts
- [ ] No file outside `tests/` modified

## 12. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 005 acceptance criteria 1–2, and 5)
3. Changed files
4. **The four findings from §7, with counts**
5. **Any suspected bug found and deliberately not fixed**
6. Differences from RFC 005, if any
7. Executed verification and results
8. Evidence per §10
9. Unresolved issues
10. Known limitations
11. Requested review focus

Items 4 and 5 are the substance. The tests are mechanical; what they reveal is
not.

## 13. Evidence standard

Standing: if a count or total does not reconcile, say so explicitly, even when
it does not change the conclusion you were asked to reach.

Particularly relevant here — you will be reporting counts, and a matrix is easy
to miscount.

## 14. Escalate rather than decide

Stop and raise it if: a pre-existing test changes; you find behaviour that looks
like data loss or a panic rather than merely a no-op option; or the matrix as
specified turns out to be ambiguous for some combination.
