# Developer Handoff — RFC 017 · `<pre>` fence newline reset

**Governing RFC.** [RFC 017](../../done/017-pre-fence-newline-reset.md) — Proposed
**Release.** Patch `2.1.8`, alongside RFC 016
**Prepared.** 2026-08-08

This Handoff directs execution of RFC 017. It does not redefine it. If
implementation uncovers a conflict, stop and raise it.

---

## 1. Purpose

Fix the `"pre"` closing-fence defect **you found** while checking every arm
against `push_raw`'s invariant after RFC 016.

You reported it as an unverified candidate. It was verified at review: it
reproduces, and it is more severe than the `<hr>` bug you had just fixed.

## 2. What the verification showed

You were right that the shape matched. What you could not know without testing
is how bad the worst case is:

```
<pre><code>x\n\n</code></pre><p>B</p>   →   "```\nx\n\n```B\n"
```

```` ```B ```` on one line is not a valid closing fence — CommonMark requires
the closing line to contain only backticks and whitespace. So **the code block
never closes**, and everything after it is rendered as code to the end of the
document.

Severity scales with trailing newlines in the `<pre>` content: none is fine, one
loses a blank line, two or more swallows the document tail. Hand-written fixtures
use the no-trailing-newline shape, which is why nothing caught it in twelve
releases.

## 3. The fix

`src/renderer.rs`, the `"pre"` arm in `leave_element`:

```rust
-                self.output.push_str("```");
+                self.push_raw("```");
```

Identical in shape to RFC 016's. **Already verified at review**, applied and
reverted — six cases, in RFC 017 §Verified.

Verify it yourself rather than trusting that table.

## 4. Change scope

| Path | Change |
|---|---|
| `src/renderer.rs` | One line, the `"pre"` arm in `leave_element` |
| `tests/` | New regression tests |

## 5. Non-change scope — do not touch

- **Any other arm in `renderer.rs`.** You already checked them all; this is the
  last instance. Do not re-sweep, and do not "make the others consistent."
- The opening fence, or `extract_code_lang`.
- How `<pre>` content itself is emitted. Content is preserved verbatim by design;
  this changes only the separator *after* the closing fence.
- `src/traversal.rs`, `src/options.rs`, `src/utils.rs`.
- `ConversionOptions` — RFC 005.
- `docs/` — documented behaviour is already correct; the code simply did not
  match it.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml` — still do not commit it

Same hazard as RFC 016. It remains untracked and is **not gitignored**, so
`git add -A` would sweep it in. Stage by explicit path:

```
git add src/renderer.rs tests/
git status            # confirm create-release.yaml is still untracked
```

You handled this correctly on RFC 016. Same again.

## 6. Required regression tests

Six cases, spanning the severity range — the defect is invisible at zero trailing
newlines, so a test suite that only covers that shape proves nothing:

| Case | Input | Expected |
|---|---|---|
| No trailing NL | `<pre><code>fn main(){}</code></pre><p>B</p>` | `"```\nfn main(){}\n```\n\nB\n"` |
| One trailing NL | `<pre><code>fn main(){}\n</code></pre><p>B</p>` | `"```\nfn main(){}\n```\n\nB\n"` |
| **Two trailing NL** | `<pre><code>x\n\n</code></pre><p>B</p>` | `"```\nx\n\n```\n\nB\n"` |
| Then `<hr>` | `<pre><code>x\n</code></pre><hr><p>B</p>` | `"```\nx\n```\n\n---\n\nB\n"` |
| `<pre>` alone | `<pre><code>x</code></pre>` | `"```\nx\n```\n"` |
| Language class | `<pre><code class="language-rust">fn f(){}\n</code></pre><p>B</p>` | `"```rust\nfn f(){}\n```\n\nB\n"` |

The **two-trailing-newline** case is the one that matters — it is the corruption.
The **language-class** case guards the opening-fence path, which writes through a
different branch and must not be disturbed. Do not drop either.

The `<hr>` case cross-checks against RFC 016's fix, since both now route through
`push_raw` and land in the same `end_block()`.

## 7. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Baseline is 107** — verified at `HEAD` after RFC 016 landed. Expect 107 plus
your additions, all passing.

RFC 017 §Risks anticipates that no existing test encodes the buggy output —
`tests/block_elements.rs` uses the unaffected shape. **If one does fail, stop and
report.** It may be a characterisation case worth updating rather than a
regression, and that is a review decision, not an implementation one.

## 8. Prohibited shortcuts

- Do not skip the two-trailing-newline case for being awkward to write. It is the
  entire point.
- Do not skip the language-class case.
- Do not fix this in `end_block()`. The defect is that `"pre"` fails to record
  what it wrote.
- Do not touch other arms.
- Do not commit `create-release.yaml`.

## 9. Required evidence

1. `cargo test --workspace --locked` — full output, new total stated and
   reconciled against 107 + additions.
2. fmt and clippy, both exit 0.
3. `git diff src/renderer.rs` — one line.
4. The six cases from RFC 017 §Verified, run by you.
5. `git status` after commit, showing `create-release.yaml` still untracked.

## 10. Acceptance checklist

- [ ] `src/renderer.rs` changed by exactly one line
- [ ] All six RFC 017 §Verified cases produce the stated output
- [ ] Six regression cases present, including two-trailing-newline and language-class
- [ ] Test count reconciles: 107 + additions
- [ ] No pre-existing test changed behaviour
- [ ] fmt and clippy clean
- [ ] `create-release.yaml` still untracked
- [ ] No file outside §4 modified

## 11. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 017 acceptance criteria, by number)
3. Changed files
4. **The six verification cases, run by you**
5. Any pre-existing test that changed, and why
6. Differences from RFC 017, if any
7. Executed verification and results
8. Evidence per §9
9. Unresolved issues
10. Known limitations
11. Requested review focus

## 12. Evidence standard

Standing: if a count does not reconcile, say so explicitly.

## 13. After this lands

`2.1.8` carries both RFC 016 and RFC 017. A release handoff follows once this is
approved.

That release is also the proving run for `release-crates.yaml` — the crates.io
automation that has never executed. Two one-line fixes are a far better place to
discover a problem in it than a feature release.

## 14. Escalate rather than decide

Stop and raise it if: a pre-existing test changes behaviour; the language-class
case misbehaves; or the fix appears to need more than the single line.

## 15. Credit where it is due

This RFC exists because, having fixed the arm you were asked to fix, you checked
the other twenty-odd against the same invariant instead of stopping. That found a
live data-corruption defect which had survived twelve releases, every review, and
a purpose-built characterisation matrix.

Reporting it as an unverified candidate rather than a confirmed defect was also
right. Overstating it would have cost more than staying quiet.
