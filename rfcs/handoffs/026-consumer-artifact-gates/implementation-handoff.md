# Developer Handoff — RFC 026 · Consumer-artifact verification gates

**Governing RFC.** [RFC 026](../../accepted/026-consumer-artifact-gates.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P0
**Depends on.** [RFC 020](../020-npm-distribution-repair/implementation-handoff.md) — it builds the npm gate; this handoff builds the other three
**Prepared.** 2026-08-31

---

## 1. Purpose

All four CI jobs verify the source tree. None verifies what a user installs.
Close that for PyPI, crates.io and the documentation examples.

## 2. The gap, stated precisely

| Job | Runs | Blind to |
|---|---|---|
| `node` | `npm run build`, `node test.js` | A published package that cannot resolve its binding — the binding is present by construction |
| `python` | `maturin develop`, `pytest` | Anything missing from the **wheel**, including `py.typed` |
| `rust` | `cargo test` in-workspace | A crate that fails to build from its packaged form |
| — | nothing | Documentation examples |

Neither job is wrong. Both answer a different question than the one that
mattered, and they answered it green for twelve releases while `npm install`
was broken.

## 3. Scope

**Scope boundary, per RFC 027 Rule 2.** This handoff covers the **PyPI wheel
gate, the crates.io package gate, and the docs-example gate**. The npm gate
belongs to RFC 020, which is landing first because npm is actively broken. No
part of this handoff changes product code — only CI.

## 3.5 ⚠ Each gate goes in its own workflow file

**Not in `ci.yaml`.** RFC 020's gate landed there, per my instruction, and
deadlocked the 2.2.1 release: `ci.yaml` is what `verify-ci` inspects, so a red
gate blocked release creation and every publisher — including the release that
would have made the gate pass.

One file per gate. Same triggers, same behaviour, same red-when-broken. Only the
file differs. Making any of them release-blocking is a separate decision, taken
only after it has been seen green against a real release.

## 4. The gates

### 4.1 PyPI — build a wheel, install it, import it

```
maturin build            # NOT develop
python -m venv /tmp/v && /tmp/v/bin/pip install target/wheels/mdka-*.whl
/tmp/v/bin/python -c "import mdka; assert '# x' in mdka.html_to_markdown('<h1>x</h1>')"
```

Then assert **inside the installed package**:

- `py.typed` is present — **only once RFC 023 decides to ship it.** Coordinate:
  if RFC 023 removes the claim instead, this assertion must not exist. Do not
  guess which way it went; check.
- Later, once RFC 007 lands, no Japanese text in the published surface. Not now.

The venv must be outside the workspace and must not have the project installed
by any other means.

### 4.2 crates.io — build from the packaged crate

`cargo package` each of the four crates, then build from the packaged output —
`cargo publish --dry-run` is an acceptable implementation if it genuinely builds
the packaged form.

This catches a crate that compiles in-workspace but fails standalone through a
missing `include`, a path dependency, or a file absent from the package.

### 4.3 Documentation examples — execute them

Extract every runnable example from `docs/src/` and run it. `D-12` and `D-13`
shipped examples that fail on their first line.

**Fragments must be marked, and the marker is what excludes them** — not their
absence from a list maintained elsewhere. Pick a convention (an info-string
marker on the fence is the obvious one), apply it, and document it in
`CONTRIBUTING`.

Coordinate with RFC 023, which is correcting those examples. **If 023 lands
first, this gate must fail against pre-023 content** to prove it works — capture
that before rebasing.

## 5. ⚠ Every gate must be seen failing

For **each** of the three gates, deliberately break something, observe the gate
fail, capture the output, restore. A gate that has only ever been green is a
claim, not a control.

Suggested breakages: delete `py.typed` from the wheel input; remove a file from a
crate's `include`; introduce a syntax error into one docs example.

**This is a required deliverable.** A review request without three captured
failures is incomplete.

## 6. The residual gap — state it, do not hide it

CI can prove the Linux artifacts install. It cannot prove the macOS and Windows
npm packages resolve, because those only exist after publication.

Record this in `ROADMAP.md` as a known limitation with release-time verification
as the mitigation. **Do not describe these gates as complete coverage.** Writing
down what a control does not cover is the point of the exercise.

## 7. Required verification

Per RFC 027 Rule 3, say for each whether it ran against the tree or an artifact.

1. Three gates added, each observed failing, output captured.
2. Each gate passing on clean `main`.
3. CI wall-clock before and after, with numbers.
4. `cargo tree -e normal` unchanged — no gate may add a runtime dependency.
5. Existing suites unchanged: 136 tests, fmt, clippy.

## 8. Prohibited shortcuts

- Do not install from the workspace tree in any gate.
- Do not use `maturin develop` — that is the blind spot.
- Do not add `continue-on-error` or `|| true` anywhere.
- Do not assert `py.typed` before checking which way RFC 023 went.
- Do not claim platform coverage the gates do not have.

## 9. Known risks

| Risk | If it happens |
|---|---|
| CI time grows materially | Report the numbers. If a gate is slow, propose moving it to release-time rather than deleting it. |
| `cargo package` fails today | That is a finding, not a blocker — report it. |
| Docs-example extraction is fiddly | Keep it simple: fenced blocks with a language tag, minus marked fragments. Do not build a framework. |
| A gate passes when it should fail | Then it is not testing what you think. §5 exists to catch exactly this. |

## 10. Acceptance checklist

- [ ] Wheel gate: `maturin build`, install into a fresh venv outside the workspace, import, convert
- [ ] Package gate: each crate builds from packaged output
- [ ] Docs gate: every runnable example executes; fragments marked by convention
- [ ] **All three observed failing, output captured**
- [ ] Residual platform gap recorded in `ROADMAP.md`
- [ ] No runtime dependency added
- [ ] CI timing reported

## 11. Escalate rather than decide

Stop and raise if: a gate cannot be made to fail on a deliberate break; RFC 023's
`py.typed` decision is not yet made; `cargo package` reveals a packaging defect
(that is its own slice); or CI time grows enough to need a trade-off decision.
