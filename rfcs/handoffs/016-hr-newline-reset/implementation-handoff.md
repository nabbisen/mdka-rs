# Developer Handoff — RFC 016 · `<hr>` newline reset

**Governing RFC.** [RFC 016](../../done/016-hr-newline-reset.md) — Proposed
**Release.** Patch `2.1.8`
**Prepared.** 2026-08-08

This Handoff directs execution of RFC 016. It does not redefine it. If
implementation uncovers a conflict, stop and raise it.

---

## 1. Purpose

Fix the `<hr>` defect you found in RFC 005 Slice A. Your diagnosis was correct
and complete; this is the fix and its regression coverage.

## 2. The fix

`src/renderer.rs`, the `"hr"` arm:

```rust
-                self.output.push_str("---");
+                self.push_raw("---");
```

That is the whole source change. `push_raw` writes the string and sets
`newlines_emitted = 0` / `at_line_start = false` — exactly the state update
`push_str` omits, which is why `end_block()` saw a stale count and emitted
nothing.

**Already verified at RFC drafting**, applied and reverted. Eight cases produce
correct output, including blockquote nesting and lists. Exactly one existing
test fails: `void_element_hr`, the one you deliberately marked. See RFC 016
§Verified for the full table.

You should still verify it yourself rather than trusting that table.

## 3. Change scope

| Path | Change |
|---|---|
| `src/renderer.rs` | One line, the `"hr"` arm |
| `tests/characterisation_elements.rs` | Update `void_element_hr` |
| `tests/` | New regression tests |

## 4. Non-change scope — do not touch

- **Any other arm in `renderer.rs`.** Several also use `output.push_str` with
  explicit state updates afterwards; those are correct. Do not "make them
  consistent."
- `src/traversal.rs`, `src/options.rs`, `src/utils.rs`.
- `ConversionOptions` in any form — RFC 005 owns it.
- `docs/` — nothing here changes documented behaviour; the docs already describe
  `<hr>` → `---` correctly. It simply did not do that.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml` — do not commit it

An untracked file is sitting in the working tree:

```
?? .github/workflows/create-release.yaml
```

It is RFC 015 Slice 2, **deliberately parked** — the project owner deferred that
work indefinitely on 2026-08-08, because it relies on `GITHUB_TOKEN` creating a
release, which does not trigger the publishing workflows.

**It is not gitignored**, so `git add -A` or `git add .github/` would sweep it
into your commit. Stage explicitly by path instead:

```
git add src/renderer.rs tests/
git status            # confirm create-release.yaml is still untracked
```

If it lands, a workflow that is known not to work becomes live on `main`.

## 5. `void_element_hr` — update, do not delete

It currently reads:

```rust
assert_matrix("void <hr>", "<p>A</p><hr><p>B</p>", ["A\n\n---B\n"; 5]);
```

Change the expectation to `"A\n\n---\n\nB\n"`, and **keep a comment** recording
that this test previously encoded the bug, pointing at RFC 016.

That comment is the point. A future reader seeing a plain corrected assertion
learns nothing; one seeing "this used to assert the buggy output, fixed by RFC
016" learns that the characterisation suite did its job.

## 6. Required regression tests

Six positions, because the bug was position-dependent and only some positions
were broken:

| Case | Input | Expected |
|---|---|---|
| First | `<hr><p>B</p>` | `"---\n\nB\n"` |
| Last | `<p>A</p><hr>` | `"A\n\n---\n"` |
| Middle | `<p>A</p><hr><p>B</p>` | `"A\n\n---\n\nB\n"` |
| Consecutive | `<p>A</p><hr><hr><p>B</p>` | `"A\n\n---\n\n---\n\nB\n"` |
| In blockquote | `<blockquote><p>Q</p><hr><p>R</p></blockquote>` | `"> Q\n\n> ---\n\n> R\n"` |
| After a list | `<ul><li>a</li></ul><hr><p>B</p>` | `"- a\n\n---\n\nB\n"` |

The blockquote case matters most — it exercises the deferred-prefix path
(`emit_pending_prefix`), which is where a naive fix would most plausibly go
wrong. Do not drop it for being awkward.

The first and last cases were **already correct** before the fix. Include them
anyway: they guard against a fix that repairs the middle case by breaking the
ends.

## 7. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Expected count: 101 + your new tests, all passing.** The 101 includes
`void_element_hr`, which now asserts corrected output rather than buggy output —
so the count does not change from that test, only from your additions.

If any test other than `void_element_hr` changes behaviour, **stop and report**.
Nothing else should be affected by a state reset on `<hr>`.

## 8. A finding to report, not fix

RFC 016 §Risks records that other `renderer.rs` arms writing via
`output.push_str` are worth auditing for the same state-desync defect.

**Do not audit them under this RFC.** But if you notice one while working, report
it — that is a finding, and it would justify its own small RFC.

## 9. Prohibited shortcuts

- Do not delete `void_element_hr` and write a fresh test. Update it.
- Do not fix this in `end_block()`. The defect is that `"hr"` fails to record
  what it wrote, not that `end_block` misreads correct state.
- Do not touch other arms.
- Do not skip the blockquote regression case.

## 10. Known risks

| Risk | If it happens |
|---|---|
| A test other than `void_element_hr` fails | Stop and report — unexpected coupling |
| Blockquote case produces the wrong prefix | The fix interacts with deferred prefixing; report with the actual output rather than adjusting the expectation |

## 11. Required evidence

1. `cargo test --workspace --locked` — full output, with the new total stated
   and reconciled against 101 + additions.
2. fmt and clippy, both exit 0.
3. `git diff src/renderer.rs` — one line.
4. The eight cases from RFC 016 §Verified, run by you.

## 12. Acceptance checklist

- [ ] `src/renderer.rs` changed by exactly one line
- [ ] All eight RFC 016 §Verified cases produce the stated output
- [ ] `void_element_hr` updated with a comment pointing at RFC 016
- [ ] Six regression cases from §6 present, blockquote included
- [ ] Test count reconciles: 101 + additions
- [ ] No test other than `void_element_hr` changed behaviour
- [ ] fmt and clippy clean
- [ ] No file outside §3 modified

## 13. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 016 acceptance criteria, by number)
3. Changed files
4. **The eight verification cases, run by you**
5. **Any other-arm state-desync candidates noticed but not fixed** (§8)
6. Differences from RFC 016, if any
7. Executed verification and results
8. Evidence per §11
9. Unresolved issues
10. Known limitations
11. Requested review focus

## 14. Evidence standard

Standing: if a count does not reconcile, say so explicitly. Relevant here — you
are adding tests to a suite whose total you should be able to state exactly.

## 15. After this lands

`2.1.8` follows. That release is deliberately small, and is the **proving run for
`release-crates.yaml`** — the crates.io automation from RFC 015 that has never
executed. A one-line patch is a far better place to discover a problem in it than
a feature release.

Release mechanics are not yours under this Handoff; a separate release handoff
will follow once the fix is approved.

## 16. Escalate rather than decide

Stop and raise it if: a test other than `void_element_hr` changes; the blockquote
case misbehaves; or the fix appears to need more than the single line.
