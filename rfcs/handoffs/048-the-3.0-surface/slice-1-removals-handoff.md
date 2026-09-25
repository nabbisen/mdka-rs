# Developer Handoff — RFC 048 slice 1 of 3 · the removals

**RFC.** `rfcs/accepted/048-the-3.0-surface.md` §5 and §6 — accepted by the owner, 2026-09-25
**Milestone.** `3.0` (breaking).
**Priority.** P1 — first of three; the other two build on the surface this leaves.
**Prepared.** 2026-09-26
**Baseline.** `37db2a3`, `2.9.0` published — 636 Rust (`cargo test --workspace`), 52 Node, 25 loader, 101 Python; eight workflows green
**Scope.** Remove three modes and six options; make `FromStr` say *removed*, not *unknown*. **No conversion output changes — §2 is the hard constraint.**

---

## 0. Where this sits

`3.0` is three slices. This is the first; the other two are written after it lands, from what it actually
leaves behind.

| | |
|---|---|
| **1. The removals — this handoff** | §5, §6: three modes, six options, the `FromStr` message |
| 2. The result model and Node's conventions | §3, §4: `ConvertResult` out, `FileOutcome` in, Node's `…With` folded away |
| 3. Documentation and the migration guide | §8.7, §8.10 |

**`2.9.0` is what makes this legitimate.** Every name removed here warned in `2.2.0`, `2.8.0` or `2.9.0`.
**Remove nothing that did not.**

## 1. What goes

**Modes.** `ConversionMode::Strict`, `Semantic`, `Preserve`. `Balanced` and `Minimal` stay, unrenamed.

**Options — six of the eight boolean fields:**

```
preserve_classes   preserve_data_attrs   preserve_aria_attrs
preserve_unknown_attrs   drop_presentation_attrs   unwrap_unknown_wrappers
```

**`preserve_ids` and `drop_interactive_shell` stay.** They are the two that affect output.

**Per surface, and they are not symmetric** — checked at `2.9.0`, do not assume:

| | attribute options | wrapper option |
|---|---|---|
| Rust | all **5** | `unwrap_unknown_wrappers` |
| Node | **3** (`preserveClasses`, `preserveDataAttrs`, `preserveAriaAttrs`) | `unwrapUnknownWrappers` |
| Python | **3** (same three) | `unwrap_unknown_wrappers` |
| CLI | **3** (`--preserve-classes`, `--preserve-data`, `--preserve-aria`) | `--unwrap-wrappers` |

`preserve_unknown_attrs` and `drop_presentation_attrs` are **Rust-only**; there is nothing to remove for them
elsewhere. `python/mdka/mdka_python.pyi:76` carries a comment saying passing `preserve_unknown_attrs` raises
`TypeError` — that comment goes too.

## 2. The trap: removing an inert flag still chooses a behaviour 🛑

**This is the one thing in this slice that can silently change output, and the assertion that would have
caught it disappears in the same commit.**

`src/traversal.rs:65–76` is the only place the wrapper flag is read:

```rust
} else if opts.unwrap_unknown_wrappers
    && utils::is_wrapper_tag(tag)
    && !utils::is_structural_tag(tag)
{
    Disposition::Unwrap
} else {
    Disposition::Render
}
```

`Minimal` sets it **`true`**; `Balanced` sets it **`false`**. Delete the field and you must pick one branch
for everybody. `mode_identity.rs` P1 currently proves both branches produce identical bytes — **and P1 is one
of the things this slice deletes**, because it flips fields that will no longer exist.

So:

1. **Decide the branch deliberately and say which in the report.** Either is defensible; `Render`
   (i.e. behave as `unwrap_unknown_wrappers: false`) is the simpler code and the `Balanced` default.
2. **Prove the choice changed nothing, before you delete P1.** Run the `mode_identity` corpus through the
   `2.9.0` binary with the flag forced *both* ways and confirm both match your `3.0` output. Paste it.
3. **Replace P1 with a property that survives**, over the same corpus: `Balanced` and `Minimal` output is
   byte-identical to what published `2.9.0` produced for those two modes. That is the promise `3.0` makes,
   and after this slice nothing else asserts it.

**`tests/output_validity/wrappers.rs` is the file that cares.** Its cells assert an unwrapped wrapper keeps
its block separation "in all five modes". Two modes now — and the cells must still pass, because the
documented behaviour of `<div>`/`<section>`/`<article>`/`<main>` does not change.

## 3. `FromStr` must say *removed*, not *unknown* — RFC 048 §6 🛑

Today an unknown mode is a hard error everywhere:

```
CLI   mdka --mode bogus   ->  exit 1, "error: unknown conversion mode: bogus. Valid: balanced|…"
Node  {mode:'bogus'}      ->  throws  "unknown conversion mode: bogus"
```

Without this, `--mode strict` falls into that message after removal — telling a user their config is
**wrong** rather than **obsolete**. RFC 041 §10 called that the one unacceptable outcome.

**Add a removed-names table to `FromStr::from_str`** so the three names get their own error:

```
conversion mode 'strict' was removed in 3.0; it was an alias of 'balanced'. Use 'balanced'.
```

**`FromStr` is the single place all three string surfaces route through** — the CLI, Node and `parse_mode`.
One table serves them all, **including the Rust caller who parses a mode from a config file and whom nothing
could warn** (RFC 048 §2.2). This is the only notice that population ever gets; the message is the feature.

**Assert it on all three surfaces**, and assert that a genuinely unknown name still gets the *unknown*
message. Two different errors, both tested.

## 4. What else must move

- **`for_mode`** collapses to two arms, each with `mode`, `preserve_ids`, `drop_interactive_shell`.
- **`--help`**, the mode list and the deprecated-flag lines. The CLI must **still fail cleanly** on
  `--preserve-classes` and `--unwrap-wrappers` — they are gone, so an unknown-flag error is correct, but
  **check the message names what to do**; a bare "unknown option" for a flag we told people about three
  releases running is poor. Say in the report what it prints.
- **The deprecation machinery for the removed items**: `warn_deprecated_field` / `warn_deprecated_flag` call
  sites for anything removed. Keep the helpers if anything still uses them; delete what is dead. **Do not
  leave a warning for a field that no longer exists.**
- **`tests/deprecation_runway.rs`** asserts the three modes carry `#[deprecated]`. Its job is done the moment
  they are gone — **delete it, and say so**, rather than weakening it into something that passes.
- Every `#[allow(deprecated)]` added for these items in `2.8.0` and `2.9.0` is now dead. **Remove them all**
  and list them in the report; a leftover `allow` silences the next real deprecation.
- The eight files listed by
  `grep -rln 'Strict\|Semantic\|Preserve\|preserve_classes\|…' tests/ cli/tests/ node/test.js python/test_mdka.py`
  — re-run that grep yourself; it found 14 files at baseline.

## 5. Not in this slice

- **`ConvertResult`, `FileOutcome`, Node's `…With` names.** Slice 2.
- The migration guide and the documentation rewrite. Slice 3 — though **`docs/` must not be left describing
  removed names**, so make the minimum edits that keep the docs true and say which you made.
- Renaming `Balanced` or `Minimal`, or any function.
- Any new option or mode.

## 6. Criteria

1. The three modes and six options are gone from every surface they existed on (§1's table), and
   `preserve_ids` / `drop_interactive_shell` remain.
2. **`Balanced` and `Minimal` output is byte-identical to published `2.9.0`** over the `mode_identity`
   corpus, with a positive control showing the comparison can see a difference. **Build `2.9.0` from the tag
   in a clean worktree** — do not use a stash.
3. §2's three steps: the branch chosen and named; both `2.9.0` branches shown equal to `3.0`'s output before
   P1 is deleted; a replacement property asserting the `2.9.0` equality, since nothing else will.
4. `tests/output_validity/wrappers.rs` passes, its cells unchanged in meaning.
5. `FromStr` gives the *removed* message for the three names and the *unknown* message for anything else,
   **asserted on the CLI, in Node, and on `parse_mode` directly**.
6. The CLI's response to a removed flag is stated in the report, with its exact output.
7. `deprecation_runway.rs` deleted, not weakened. Every now-dead `#[allow(deprecated)]` removed and listed.
8. `docs/` contains no reference to a removed mode or option as though it still exists; the minimum edits are
   listed.
9. Counts reported with their commands; eight workflows green.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours. **Name who approved anything you push.**
`git commit -F <file> -- <explicit paths>`.

**This lands on `main` as a breaking change with no release behind it yet.** That is expected: `3.0` is
three slices and ships once. Between slices `main` is a `3.0` in progress, so **do not treat a red
downstream gate as necessarily yours** — `npm install gate` and `pypi published gate` report on published
`2.9.0` and will stay green, but `docs example gate` compiles examples from `docs/`, and an example naming a
removed mode will fail. That is criterion 8 doing its job.

Report to `.git-exclude/review-request/048-slice-1-removals/README.md`, leading with §6.2 and §6.3 — the
byte-identity evidence and the branch decision. Those are the deliverable; the deletions are the easy half.
