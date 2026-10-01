# Developer Handoff — `3.1.0` release preparation

**Authorised.** Owner, 2026-10-01 — ship RFC 049 next, as its own small release. **Preparation is
yours; the cut is not.**
**Milestone.** `3.1.0` — one new opt-in option, plus two test/CI repairs.
**Prepared.** 2026-10-01
**Baseline.** `faf305e` — 648 Rust (`cargo test --workspace`), **eight workflows green on the full
SHA**, tree clean, `HEAD` and `origin/main` agree.
**Scope.** Version, `CHANGELOG.md`, **one RFC move**, `ROADMAP.md`. **No code, no test, no doc
rewrite** — RFC 049's implementation and documentation are both already reviewed and accepted.
**Checklist.** `RELEASE-CHECKLIST.md` §1.

---

## 1. The version number is not open

**`3.1.0`.** `emphasis_from_style` is a new field on a public struct and a new flag on three
surfaces — additive, nothing removed, nothing renamed. Minor. **Prep picks the date, not the number.**

## 2. One RFC moves to `done/`

**RFC 049 → `rfcs/done/049-inline-style-emphasis.md`, release `3.1.0`.** Index updated **in the same
commit**, references swept. After the move `accepted/` is empty again, as it was after `3.0.0`;
`proposed/` is already empty. **State that in the report** — an empty stage folder is absent from a
fresh clone, which `rfcs/README.md` already notes.

The index entry currently reads *"**Implemented** (`96c9a28`, docs `8517c4d`); awaits a release"* —
that phrasing goes away with the move.

## 3. The CHANGELOG's jobs

One dated `## [3.1.0]`, doing four things:

1. **`emphasis_from_style`, named on every surface it has** — `emphasis_from_style` (Rust, Python),
   `emphasisFromStyle` (Node), `--emphasis-from-style` (CLI). Say **off by default**, and say that
   with it off **output is byte-identical to `3.0.0`** — that is the promise a reader upgrading cares
   about most, and 397 byte-identity tests hold it.
2. **The two boundaries, briefly**, pointing at `docs/src/api/options.md` rather than repeating it:
   table cells and bare-text list items do not receive a container's declaration, and **`Minimal`
   unwraps the wrapper that would carry it**, so the option does nothing for a Google Docs paste in
   that mode.
3. **The allocation gate** — a new test, not a user-facing change. One line.
4. **The `crates package gate` repair** — likewise one line. Worth recording because it is the kind
   of thing a future reader will want dated: the gate could previously verify source that was not in
   the tree.

**Do not claim a performance change.** Nothing in this release was measured as faster or slower, and
the one measurable property (allocation) is now gated, not improved.

## 4. `ROADMAP.md`

Reflect the release. Check RFC 049's criteria off. The items that survive it and should **stay** open:
the performance page's standing (RFC 012 §9), and bekoedit's letter, which is still owed and is not
yours.

## 5. What is not yours

- **The bekoedit letter.** Still owed from `3.0.0`, and `3.1.0` changes what it must say — their top
  gap is closed, *but only in `Balanced`*. **Architect's to draft, owner's to send.**
- The tag, the publishers, the registries, the GitHub release.
- Any code, test or documentation change. **If prep turns up a defect, report it** — do not fix it.

## 6. What to expect

- **`crates package gate` should be GREEN**, and this is the **first version bump since its repair**.
  The bump changes the workspace members' versions, so every `target/package/` path is new and the
  delete step has less to do. **If it goes red, stop and report** — do not assume it is the old
  staleness; that mechanism is now fixed and proven, so a red here would be something else.
- **`release artifact gate` may be RED during prep** and that is expected at most releases; acceptable
  only for contract/README mismatches arising from this same commit. The post-publish dispatch settles
  it.
- **The allocation gate is version-independent** — it measures bytes, not versions. A bump must not
  move it. If it does, that is a real finding; report it rather than re-baselining.
- `npm install gate` and `pypi published gate` describe published `3.0.0`. Expected.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **3.1.0**; the assertion clean.
2. One dated `## [3.1.0]` doing all four jobs in §3.
3. RFC 049 in `rfcs/done/` with `3.1.0`; index updated **in the same commit**; references swept;
   `accepted/` and `proposed/` both empty, stated.
4. `ROADMAP.md` reflects the release, with §4's survivors still open.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — `gh run list --commit <full-sha>`; an abbreviated SHA
   returns empty silently.
7. Counts, each with its command: **648** Rust, 60 Node, 25 loader, 93 Python.

## 8. Report, then stop

`.git-exclude/review-request/3.1.0-prep/README.md`, leading with §7.6. **Then stop** — I raise the
checkpoint, the owner answers, and the owner and I cut. **Name who approved anything you push.**
