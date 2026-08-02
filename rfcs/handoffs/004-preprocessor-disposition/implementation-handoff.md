# Developer Handoff — RFC 004 · Orphaned preprocessor disposition

**Governing RFC.** [RFC 004](../../done/004-preprocessor-disposition.md) — Implemented (2.1.7)
**Milestone.** M1 · Trustworthy baseline → `2.1.7`
**Position in M1.** Third, after RFC 001 and RFC 002. **RFC 003 is blocked on this landing.**
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 004. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

---

## 1. Purpose

Delete a mode-aware DOM preprocessor that has never been compiled in any
published release — but only after transcribing its test assertions, which are
the sole surviving specification of intended attribute behaviour and are needed
by RFC 005.

## 2. Background

`tests/utils/preprocessor.rs` (227 lines) and
`tests/utils/preprocessor/tests.rs` (115 lines) are not built by any target.

Cargo compiles `tests/*.rs` as integration-test targets. `tests/utils/` is a
subdirectory, built only if some target declares `mod utils`. Nothing does —
only `tests/common.rs` is declared, by `block_elements`, `compat`,
`inline_elements`, and `robustness`. Confirmed by binary enumeration:
`cargo test --workspace --no-run` produces exactly ten executables, none for
`tests/utils/`.

The code could not compile there even if declared: `tests/utils/preprocessor/tests.rs`
uses `use crate::options::…` and calls `utils::preprocessor::preprocess(…)`,
paths that only resolve inside the library crate.

Git history establishes how long this has been true. `src/preprocessor.rs` never
existed on `main` — it lived only on the pre-squash `2.0.0 dev` branch. When
that branch landed as `e7b2dbd`, the code arrived already at `tests/utils/`.
The blob hash is identical (`c6444907`) at tag `2.0.0` and tag `2.1.6`.

**So this code has been dead since 2.0.0 shipped on 2026-04-15 — across all
eleven releases of the 2.x line — and was never compiled in a published
version.** No compiler has ever warned about it, because no compiler has ever
seen it. RFC 001's gate would not catch it either: a gate only checks what
compiles.

The handoff bundle's account of a `src/preprocessor.rs` emitting five dead-code
warnings describes the pre-squash branch state, not anything that shipped.

## 3. Applicable requirements

RFC 004 §Goals: one preprocessing implementation, not two; no unreachable code
masquerading as tests; the behavioural intent in those 115 lines preserved where
it will actually run.

## 4. Change scope

| Path | Change |
|---|---|
| `tests/utils/preprocessor.rs` | Delete |
| `tests/utils/preprocessor/tests.rs` | Delete |
| `tests/utils/` | Remove the now-empty directory |
| `src/utils.rs` lines 78–99 | Delete the commented-out `is_void_element` block |
| RFC 004 (this RFC's file) | Append the harvested test inventory, per §5 |

## 5. Do this first — the harvest is blocking

**Do not delete anything until the inventory is complete.**

`tests/utils/preprocessor/tests.rs` is the only surviving specification of what
the eight `ConversionOptions` fields were *meant* to do. Six of those eight
fields are currently inert — they are read nowhere in `src/`, and every
conversion mode produces byte-identical output for attribute-bearing HTML. RFC
005 exists to implement them, and it will start from this inventory.

Transcribe **every** assertion, not a sample. For each `#[test]` function
record:

| Field | Content |
|---|---|
| Test name | The `fn` name verbatim |
| Input HTML | The exact input string |
| Mode / options | Which `ConversionMode` or field values it exercises |
| Asserted behaviour | What it requires to be true, in one sentence |
| Currently implemented? | Yes / No — check against `src/traversal.rs` and `src/renderer.rs` |

Known starting points, from review: script dropping regardless of mode;
`class`/`style` removal under `Balanced`. There are more — the file has 115
lines. Enumerate them all.

Write the inventory into RFC 004 under a new `## Harvested test inventory`
section, so it lives with the design record rather than in a scratch file that
will be lost.

The "Currently implemented?" column is the valuable part. It becomes RFC 005's
gap list, and it is the deliverable this Handoff most cares about.

## 6. Then delete

Remove the two files and the empty `tests/utils/` directory.

Remove the commented-out `is_void_element` block at `src/utils.rs:78–99`,
including its `// moved to /tests/utils/preprocessor.rs` note, which will
otherwise dangle.

Deleting the code loses no design record: RFC 004, the handoff bundle, and git
history all retain it. RFC 000's "never delete an RFC" principle governs design
documents, not superseded implementation code.

## 7. Non-change scope — do not touch

- **`src/traversal.rs`, `src/renderer.rs`, `src/options.rs`.** Do not implement
  any attribute handling. That is RFC 005. This RFC only removes a competing
  implementation and captures its intent.
- **`docs/`.** `docs/src/design/architecture.md` references
  `tests/utils/preprocessor.rs` in its workspace-layout tree and will be wrong
  the moment you delete it. **Leave it.** RFC 003 owns that edit and is
  explicitly sequenced after this RFC for exactly this reason.
- The rest of `src/utils.rs`. Only the commented block goes; the live tag
  classification functions stay.
- `tests/common.rs` and the six live integration test files.
- Any manifest, workflow, or binding.
- Japanese comments — RFC 007 and RFC 013 own them.

## 8. Required tests

No new tests. `cargo test --workspace` must still report **74 passed,
0 failed** — the count must **not** change, because the deleted tests were never
running.

**A changed count means something else moved. Stop and report rather than
accepting it.**

## 9. Required verification

Use binary enumeration, not source inspection. It is what established the
problem, and it is the only method that distinguishes "declared but empty" from
"not compiled at all":

```
cargo test --workspace --no-run     # must still list exactly 10 executables
cargo test --workspace              # 74 passed, 0 failed
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
grep -rn "preprocessor\|is_void_element" src/ tests/ cli/ node/ python/
```

The final grep must return no live reference. Hits in `docs/` are expected and
correct to leave — RFC 003 handles them.

## 10. Compatibility constraints

None. No public API, no persistent format, no behaviour change. Nothing removed
is reachable from `mdka`, `mdka-cli`, `mdka-node`, or `mdka-python`.

Output must be byte-identical before and after. If any conversion output
changes, you have deleted something live — stop immediately.

## 11. Security constraints

Neutral, marginally positive: removes an unmaintained DOM-manipulation code path
that could otherwise be reused later without review.

## 12. Prohibited shortcuts

- **Do not delete the test file before the inventory is complete and written
  into RFC 004.** This is the single most important instruction here. Those
  cases are unrecoverable in practice once gone — nobody will reconstruct them
  from git archaeology when RFC 005 starts.
- Do not "fix" the module by wiring it into a test target. That would add 115
  tests asserting behaviour the engine does not implement; they would fail.
- Do not implement any attribute filtering to make a harvested assertion pass.
- Do not touch `docs/`.
- Do not delete anything else that looks dead. If you find more unreachable
  code, report it — it is a finding, not a licence to widen scope.

## 13. Known risks

| Risk | If it happens |
|---|---|
| Inventory transcribed as a summary rather than exhaustively | RFC 005 reimplements from scratch and diverges from original intent. This is the failure mode to avoid; completeness is checked at review. |
| Test count changes after deletion | Something live was removed. Stop and report. |
| An assertion describes behaviour that contradicts current engine output | Expected for several. Record it in the "Currently implemented?" column as `No`. Do not resolve the contradiction — that is RFC 005's job. |
| More dead code found nearby | Report as a finding; do not remove under this RFC. |

## 14. Required evidence

1. The harvested inventory, as committed into RFC 004.
2. `cargo test --workspace --no-run` — output showing ten executables.
3. `cargo test --workspace` — 74 passed, 0 failed.
4. `cargo clippy … -D warnings` and `cargo fmt --check` — both exit 0.
5. The grep from §9, showing no live reference outside `docs/`.
6. `git status` showing exactly the deletions in §4 and nothing else.

## 15. Acceptance checklist

- [ ] Inventory complete — every `#[test]` in the deleted file accounted for
- [ ] Inventory includes the "Currently implemented?" assessment per case
- [ ] Inventory written into RFC 004, not a scratch file
- [ ] `tests/utils/` removed in full, directory included
- [ ] `src/utils.rs:78–99` commented block removed
- [ ] `cargo test --workspace` reports 74 passed, 0 failed
- [ ] `cargo test --workspace --no-run` still lists ten executables
- [ ] clippy and fmt both clean
- [ ] No live reference to the removed module outside `docs/`
- [ ] `docs/` untouched
- [ ] No change to `src/traversal.rs`, `src/renderer.rs`, or `src/options.rs`
- [ ] No file outside §4 modified

## 16. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 004 goals, by number)
3. Changed files — complete list
4. **The harvested inventory, and your confidence that it is exhaustive**
5. **Any assertion whose behaviour the engine does not currently implement**
6. Differences from RFC 004, if any, and why
7. Executed tests and results
8. Evidence per §14
9. Unresolved issues
10. Known limitations
11. Requested review focus

Items 4 and 5 are the substance of this review. The deletion itself is trivial;
the inventory is the deliverable.

## 17. Escalate rather than decide

Stop and raise it if you find: the test count changes; conversion output changes
in any way; an assertion that appears to describe *currently shipping* behaviour
(which would mean the code is not as dead as established); or further
unreachable code elsewhere in the tree.
