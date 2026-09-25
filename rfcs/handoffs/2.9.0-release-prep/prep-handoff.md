# Developer Handoff — `2.9.0` release preparation

**Authorised.** Owner, 2026-09-25. **Preparation is yours; the cut is not.**
**Milestone.** `2.9.0` — RFC 048's precondition. **`3.0` follows immediately** (owner, 2026-09-25), which shapes §3.
**Prepared.** 2026-09-25
**Baseline.** `ff29255` — 636 Rust (`cargo test --workspace`), 52 Node, 25 loader, 101 Python; eight workflows green
**Scope.** Version, `CHANGELOG.md`, `ROADMAP.md`. **No code, no test, no workflow, and no RFC moves — §4.**
**Checklist.** `RELEASE-CHECKLIST.md` §1, tracked in the repository. This handoff covers §1 only.

---

## 1. Same split, third time

You prepare; the owner and I cut. **Not yours:** creating or pushing the tag, dispatching or re-running a
release workflow, publishing, creating a GitHub release. `git commit -F <file> -- <explicit paths>`, because
`version.sh` runs `git add` itself.

## 2. What `2.9.0` contains

7 commits since `2.8.0` at the time of writing — **re-run `git rev-list --count 2.8.0..HEAD` yourself**; my
number has been stale on both previous preps and you caught it both times.

**RFC 048's `2.9.0` precondition, in two slices, both already reviewed and approved:**

- **The deprecation.** `unwrap_unknown_wrappers` is deprecated on all four surfaces, with a note that says
  it is *removed from the `3.0` surface* — **not** the attribute-syntax reason, which would be false for
  this field. The string-form behaviour of a mode is now pinned by tests on every surface.
- **The API reference is tri-lingual.** `docs/src/api/` now lists all 9 Rust, 10 Node and 9 Python
  functions with the asymmetries visible, covers errors for all three bindings, and no longer claims a
  completeness it did not have.

**Nothing is removed and no conversion output changes.**

## 3. The CHANGELOG has an unusual job this time 🛑

**`3.0` ships immediately after this release.** A user who upgrades to `2.9.0` will meet `3.0` within days,
and `2.9.0`'s deprecation warning is the only notice some of them will ever get for a removal that is
already scheduled.

So the entry must do two things a normal entry does not:

1. **Say that `3.0` is next and what it removes** — the three alias modes, the five attribute options and
   `unwrap_unknown_wrappers`. Name them. A reader deciding whether to upgrade needs to know this is a
   stepping stone, not a resting place.
2. **Say plainly that `2.9.0` changes no output and removes nothing**, so upgrading to it is free and is the
   cheapest way to find out whether `3.0` will touch their code: fix what warns, and `3.0` is a smaller
   step.

Point at RFC 048 for the shape of `3.0`; **do not describe the new API in detail** — it is not built, and a
CHANGELOG that promises a surface is a promise we have to keep exactly.

**Also worth a line:** the API reference now covers all three bindings. It is documentation, but it is the
document a user will read while migrating.

## 4. No RFC moves — again, and for a different reason than last time 🛑

`2.8.0` moved none because RFC 041 was unfinished. **`2.9.0` moves none because RFC 048 is a `3.0`
document** and this release ships only its precondition. RFC 039, 041 and 048 all stay in
`rfcs/accepted/`; the Implemented table gains no row.

**Update RFC 048's index row** so a reader can see that its `2.9.0` precondition shipped and the rest is
`3.0`. One row, no move. If that feels wrong, say so in the report rather than moving anything.

## 5. The work

1. `sh version.sh --update 2.9.0`; confirm the post-update assertion and that `node/index.js` was rewritten.
2. `CHANGELOG.md` — one dated `## [2.9.0]` section, doing §3's two jobs.
3. `ROADMAP.md` — what `2.9.0` ships, strike through what it closes, and **leave open**: RFC 048's
   criteria, the `FileOutcome` repr item, and the four carried audit items
   (`architecture.md`'s missing `src/table.rs`, the `2.3.0` performance measurement, the two `version()`
   examples, `usage-python.md`'s missing `version()`).
4. One push, so CI runs against the commit that will be tagged.

## 6. What to expect, and what would be a finding

- **`release artifact gate` should be GREEN**, third release running. This prep changes no README Quick
  Start block, no artifact contract, no build matrix. **If it is red, that is a finding**, not the standing
  explanation in `RELEASE-CHECKLIST.md` §1.
- **`npm install gate`** describes the last published release, `2.8.0`. Expected.
- Everything else green, `crates package gate` included.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **2.9.0**; assertion clean.
2. `CHANGELOG.md`: exactly one dated `## [2.9.0]`, doing both jobs in §3 and **promising no API detail**.
3. **`rfcs/done/` unchanged**; 039, 041 and 048 all still in `accepted/`; RFC 048's index row updated.
4. `ROADMAP.md` reflects the release, with §5.3's items left open.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — `gh run list --commit <full-sha>`; an abbreviated SHA
   silently returns empty.
7. Test counts unchanged, each with its command: `cargo test --workspace` **636**, `node test.js` **52**,
   `node test-loader.js` **25**, `pytest test_mdka.py` **101**.

## 8. Report, then stop

`.git-exclude/review-request/2.9.0-prep/README.md`, leading with §7.6. **Then stop** — I raise the
checkpoint, the owner answers, I tag. **Name who approved anything you push.**
