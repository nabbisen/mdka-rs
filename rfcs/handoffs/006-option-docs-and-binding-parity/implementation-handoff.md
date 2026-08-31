# Developer Handoff — RFC 006 · Option docs and binding parity

**Governing RFC.** [RFC 006](../../done/006-option-docs-and-binding-parity.md)
**Depends on.** [RFC 005](../../done/005-conversion-options-semantics.md) — complete, approved
**Milestone.** M2 · closes it
**Prepared.** 2026-08-12
**Baseline.** 132 tests on `main` (`8978ed4`), green.

---

## 1. Purpose

RFC 005 made the option code honest. The docs and the bindings are still not.
Four independent slices. **You may land them separately** — nothing here is
sequenced except the Node caveat in §5.2.

## 2. Facts you can rely on — verified, do not re-derive

Checked against source on 2026-08-12:

- The **preprocessor does not exist**. Deleted under RFC 004. Any doc sentence
  about a "pre-processed DOM" or acting "during pre-processing" is describing
  deleted machinery.
- **`preserve_aria_attrs` is read nowhere in `src/`.** The claim in
  `options.md` that Semantic mode's logic uses it is false, not merely stale.
- **Balanced, Strict and Preserve are byte-identical**, measured across the
  RFC 005 Slice A matrix.
- **`unwrap_unknown_wrappers` appears zero times** in `cli/src`, `node/src` and
  `python/src`.
- The only two guard-mutation sites in `renderer.rs` are `:267`
  (`in_pre = true`) and `:315` (`capture_depth += 1`).

If any of these turns out to be wrong, **stop and tell me** — several are load
bearing for the wording you will write.

## 3. Slice A — `docs/src/api/options.md`

### 3.1 Field reference

Rewrite each entry to describe what the code does.

- **`preserve_ids`** — emits an escaped `<a id="…"></a>` anchor for elements
  carrying a non-empty `id`. Document the placement rule: the anchor is leading
  content of the element (`## <a id="x"></a>Install`), except on `<a>` and
  `<pre>`, where it precedes the element. Document that an `id` on a descendant
  of a link or a code block is deliberately **not** emitted, and why.
- **The five no-ops** — `preserve_classes`, `preserve_data_attrs`,
  `preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs`.
  State plainly: no effect on output, deprecated since 2.2.0, because Markdown
  has no attribute syntax. Point at RFC 005. Do not soften this into "limited
  effect" or "reserved for future use."
- **`drop_interactive_shell`, `unwrap_unknown_wrappers`** — these work. Verify
  the current descriptions against `src/utils.rs` before keeping them;
  `unwrap_unknown_wrappers`'s tag list must match `is_wrapper_tag`, and the
  interaction with `is_structural_tag` matters (see §4.2).

Remove every reference to pre-processing.

### 3.2 The defaults table

Keep it — the defaults are real — but add an effect column, or mark the five
no-op rows inline. A reader must not come away believing the table describes five
axes of behaviour.

## 4. Slice B — `modes.md` and `elements.md`

### 4.1 `modes.md`

`Strict`'s stated goal is *"Preserve as much of the original HTML information as
possible"* and `Preserve` offers *"round-trip fidelity."* Neither is true.

State plainly that **Balanced, Strict and Preserve currently produce identical
output**, and why: they differ only in the defaults of five fields that have no
effect.

Two things to get right:

- Say it is a description of **today**, not a deprecation. The three remain
  distinct API and may diverge later. Do not suggest removing any mode.
- The claim must be **backed by the Slice A characterisation suite**, not
  restated from the RFC. Cite the test. If the suite does not currently prove it
  for the exact fixtures you cite, add a test that does.

Revisit the "which mode should I use?" decision list at the end — it currently
routes users to modes that do the same thing.

### 4.2 `elements.md` — `figure` / `figcaption`

The Block Elements table groups `<div>`, `<article>`, `<section>`, `<main>`,
`<figure>`, `<figcaption>` in one row claiming all six unwrap in
Minimal/Semantic. **False for the last two.**

Verify in `src/utils.rs`: `is_wrapper_tag` excludes both; `is_structural_tag`
includes both, which blocks unwrapping regardless. Split the row and describe
both behaviours. Confirm the other four actually do unwrap before asserting it.

## 5. Slice C — binding parity

### 5.1 Add `unwrap_unknown_wrappers` to all three bindings

Follow each binding's existing convention rather than inventing one:

| Binding | Pattern to match |
|---|---|
| CLI | Existing flags are `--preserve-ids`, `--drop-shell`. Pick a consistent short form and add it to both the `--help` text and the module doc block at the top. |
| Node | Existing keys are `preserveIds`, `dropInteractiveShell`. Add the field to the options struct and `to_rust_opts`. |
| Python | Add to the struct, the plumbing, **and** the `#[pyo3(signature = …)]` list. |

One test per binding proving the option changes output. Use a **bare-sibling-text
fixture** — RFC 005 Slice A found that block-element fixtures cannot discriminate
this field, and a naive fixture will pass while proving nothing.

### 5.2 ⚠ Node: this forces the `node/index.js` regeneration

Adding a Node option means running `npm run build`, which regenerates **both**
`node/index.d.ts` and `node/index.js`.

`node/index.js` currently hardcodes `expected 2.0.2` in four napi-rs version
checks while `package.json` is `2.1.8` — stale by six releases, shipped to npm.
That is roadmap finding **R-02**. Regenerating will change those strings.

**That change is expected and wanted. Do not revert it this time** — the
previous instruction to revert applied to a slice where regeneration was out of
scope.

But review it deliberately:

1. Commit the regeneration **separately** from your hand-written changes, so the
   generated diff is reviewable on its own.
2. Read the whole diff. `npm run build` output depends on the locally installed
   napi-rs, so it may carry churn unrelated to the version strings.
3. **Report anything in that diff beyond the version strings and the new
   option.** If the local napi-rs differs from what CI uses, the regeneration
   could introduce drift in the other direction.
4. Confirm the four `2.0.2` occurrences are gone.

If the diff looks larger or stranger than that, **stop and raise it** rather than
committing it.

### 5.3 Deprecation warnings

`#[deprecated]` does not cross FFI — Node and Python callers currently see
nothing.

Emit a runtime deprecation warning when one of `preserve_classes`,
`preserve_data_attrs` or `preserve_aria_attrs` is **explicitly passed**, using
each ecosystem's normal mechanism (Node: `process.emitWarning` with
`DeprecationWarning`; Python: `warnings.warn` with `DeprecationWarning`).

**Passing nothing must stay silent.** The fields are `Option<bool>` in both
bindings, so `Some(_)` is the trigger — not the value. Warning on defaults would
make every call noisy and get the warning suppressed wholesale.

For the CLI, mark the three flags deprecated in `--help` text. No runtime warning
needed.

Test both: explicit pass warns, omission is silent.

## 6. Slice D — renderer drift guard

`enter_element` uses `anchor_before = matches!(tag, "a" | "pre")`, duplicating
knowledge at `renderer.rs:267` and `:315`.

1. A comment at **both** mutation sites naming `anchor_before` and saying that
   adding a third site requires updating it. That is where a future editor will
   be looking.
2. A test that asserts **the observable**, not the list: for each tag mdka
   handles specially, an element carrying a non-empty `id` produces an anchor
   somewhere in the output under a mode where `preserve_ids` is on.

Requirement 2's wording matters. A test that asserts `matches!(tag, "a" | "pre")`
is still the right set would pass forever and detect nothing. The test must fail
when a new tag starts setting a guard.

## 7. Non-change scope

- **Do not remove any option** from any surface, including the deprecated five.
  Breaking for npm and PyPI consumers; no major version planned.
- Do not revive attribute preservation.
- Do not merge, rename or remove any mode.
- Japanese comments in `cli/src/main.rs` — RFC 007 and RFC 013.
- Table support — RFC 008.
- `src/options.rs` — RFC 005 settled it. Docs only, unless §3.1 turns up a doc
  comment that contradicts the code, in which case fix the comment and say so.

### ⚠ `.github/workflows/create-release.yaml`

Untracked, not gitignored. Stage by explicit path. This handoff does not touch
release tooling.

## 8. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd node && npm run build && node test.js
cd python && <the project's usual test invocation>
```

**Baseline 132.** Expect 132 plus your additions.

If `docs/` builds in CI, confirm it still builds.

## 9. Prohibited shortcuts

- Do not soften a no-op into "limited effect" or "reserved."
- Do not assert a docs claim you have not run. This RFC exists because that
  happened four times.
- Do not warn on default values.
- Do not use a block-element fixture for `unwrap_unknown_wrappers`.
- Do not bundle the `node/index.js` regeneration into a hand-written commit.
- Do not write a drift test that asserts the tag list.

## 10. Known risks

| Risk | If it happens |
|---|---|
| The three-mode identity claim is not actually provable from the current suite | Add the test. If it turns out they are **not** identical for some fixture, **stop and report** — that contradicts RFC 005's basis. |
| `npm run build` produces a large or surprising diff | Stop and raise it. Do not commit a generated diff you cannot explain. |
| A doc claim you go to verify turns out false in a new way | Report it. Do not fix silently — the catalogue of what was wrong is the milestone's output. |
| Adding a CLI flag collides with in-flight RFC 007/013 work | Sequence Slice C after them; say so rather than merging blind. |

## 11. Required evidence

1. Every corrected claim, with the source location proving the new wording.
2. The `figure`/`figcaption` behaviour, run — not read from `utils.rs`.
3. Slice C: the new option exercised through **each** binding, with output.
4. The `node/index.js` regeneration diff, reviewed, with the four `2.0.2`
   occurrences shown gone and anything else in that diff explained.
5. Deprecation warnings: explicit pass warns, omission silent, per binding.
6. Slice D: proof the drift test fails when a guard site is added — simulate it.
7. Count reconciled against 132; fmt, clippy, node and python suites clean.

## 12. Acceptance checklist

- [ ] No option description references pre-processing or deleted machinery
- [ ] Five no-ops marked in the field reference **and** the defaults table
- [ ] `modes.md` states the three-mode identity, backed by a cited test
- [ ] Mode selection guidance no longer routes users to identical modes
- [ ] `figure`/`figcaption` row corrected, behaviour verified by running it
- [ ] `unwrap_unknown_wrappers` reachable from CLI, Node and Python, tested each
- [ ] Bare-sibling-text fixture used for that field
- [ ] `node/index.js` regenerated in its own commit, diff reviewed, `2.0.2` gone
- [ ] Deprecation warnings on explicit pass only, tested both ways
- [ ] Comments at `renderer.rs:267` and `:315`
- [ ] Drift test asserts the observable and demonstrably fails on a new guard site
- [ ] No option removed anywhere
- [ ] Count reconciles; all suites clean; `create-release.yaml` untracked

## 13. Required review-request format

Standard eleven parts. The substance:

4. **Each corrected claim with its verifying source location**
5. **The `node/index.js` regeneration diff, explained**
6. **Anything you went to verify that turned out false in a way this handoff did
   not predict** — that list is the most valuable thing you can return, and it
   has been in each of the last three rounds

## 14. Escalate rather than decide

Stop and raise it if: the three modes are not identical for some fixture; the
regeneration diff is not explainable; a deprecated field turns out to have an
effect after all; or removing an option looks necessary.

## 15. After this lands

**M2 closes.** Its exit criterion — every public option demonstrably does what it
says, on every surface — is met.

Remaining M2-adjacent items are already homed: R-01 and R-03 in M4, RFC 007 and
RFC 013 for the Japanese comments, RFC 008 for tables.
