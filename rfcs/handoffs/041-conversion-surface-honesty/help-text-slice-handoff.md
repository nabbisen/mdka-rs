# Developer Handoff — RFC 041 §6.2 follow-up · the two places generated from code

**RFC.** `rfcs/accepted/041-conversion-surface-honesty.md` §9
**Follows.** the wording slice — `.git-exclude/reviewed/041-wording-slice/README.md` §3
**Priority.** P2. Small, and it closes the last false statements about the modes
**Prepared.** 2026-09-24
**Baseline.** the wording slice's commit; take this after it is pushed and green
**Scope.** Two strings and some doc comments, plus one sentence. **No behaviour, no option, no mode, no signature.**

---

## 1. Why this exists

The wording slice corrected every prose statement that the alias modes convert differently. **It could not
touch the two texts that are generated from code**, because that handoff put code out of bounds — correctly,
to keep a documentation change reviewable as a documentation change.

The result is that the last inaccurate descriptions of the modes are now the ones a user is most likely to
meet: `mdka --help`, and the `ConversionMode` rustdoc that docs.rs publishes.

## 2. `mdka --help` (`cli/src/main.rs`, ≈ lines 59–62)

```
strict    Removes as few attributes as possible; for debugging and comparison
semantic  Favours semantic attributes and document structure; for SPAs and accessibility
preserve  Retains as much of the original as possible; for archiving and auditing
```

None of the three is true: they produce identical output to `balanced` and cannot differ from it.

Make the `Modes:` block say what `docs/src/api/modes.md` now says, in one screen of help text. `balanced` and
`minimal` keep real descriptions; the other three should be shown as aliases rather than given purposes they
do not have. The exact phrasing is yours — **match the docs page's substance, do not invent a third
wording**, and keep the block scannable.

**Check the width.** Help output is read in a terminal; if the aliases read better as one line than three,
use one line.

## 3. The `ConversionMode` rustdoc (`src/options.rs`, ≈ lines 8–22)

The type-level table still says *"Strict — Debugging and comparison; maximum retention"* and
*"Preserve — Archiving; retains as much as possible"*. **This is what docs.rs shows**, so for a Rust consumer
it is the primary reference.

Correct the table and the per-variant doc comments beneath it (`/// Accuracy first. …`, and the equivalents
on `Semantic` and `Preserve`). Same rule: match the docs page, do not invent new wording.

**`Minimal`'s and `Balanced`'s descriptions are accurate — leave them.**

## 4. One sentence in `docs/src/api/options.md` (≈ line 177)

> `unwrap_unknown_wrappers` is unimplementable for nothing — **a future mode that preserves raw HTML wrappers
> would make it observable again at once.**

That is the same shape as the *"may diverge again"* the last slice removed: a hypothetical nothing supports,
and RFC 041's accepted direction is to **collapse** the mode set, not add to it.

**Keep the surrounding point** — that this is *not* a deprecation, carries no `#[deprecated]`, and triggers
no warning. Replace only the speculation, with what is actually true: the option is inert because Markdown
has no wrapper element to show the difference, so nothing about today's output can distinguish it.

## 5. Not in this slice

- Any behaviour, option default, mode, or public signature.
- The structural collapse — `3.0`, with RFC 039 Half B.
- `<sup>` notation and `py.typed` stubs: both decided, both getting their own RFCs.

## 6. Criteria

1. `mdka --help` contains no claim that `strict`, `semantic` or `preserve` converts differently from
   `balanced`. **Paste the new `Modes:` block in the report.**
2. The `ConversionMode` rustdoc and its per-variant comments likewise. `cargo doc` builds.
3. `docs/src/api/options.md` no longer speculates about a future mode; the not-a-deprecation point survives.
4. **`--help` and `docs/src/api/modes.md` now say the same thing** — `usage-cli.md` tells the reader `--help`
   wins when they disagree, so they must not disagree.
5. No behaviour change: **583 Rust / 42 Node / 25 loader / 90 Python**, seven workflows green. The CLI flag
   table in `usage-cli.md` still mirrors `--help` — check it, since you are editing the help text.
6. No file outside `cli/src/main.rs`, `src/options.rs` and `docs/src/api/options.md`.

## 7. Report back

`.git-exclude/review-request/041-help-text-slice/README.md`, with the new `Modes:` block and the rustdoc
table pasted in full. They are short, and they are the deliverable.

## 8. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.
