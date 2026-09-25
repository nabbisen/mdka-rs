# Developer Handoff — `3.0.0` release preparation

**Authorised.** Owner, 2026-09-25 — `3.0` immediately after `2.9.0`. **Preparation is yours; the cut is not.**
**Milestone.** `3.0.0` — the first deliberately staged breaking release this project has made.
**Prepared.** 2026-09-26
**Baseline.** `51354ff` — 629 Rust (`cargo test --workspace`), 59 Node, 25 loader, 91 Python; eight workflows green
**Scope.** Version, `CHANGELOG.md`, **three RFC moves**, `ROADMAP.md`, and one investigation (§3). **No code, no test, no doc rewrite — slice 3 finished the documentation.**
**Checklist.** `RELEASE-CHECKLIST.md` §1.

---

## 1. The version number is not open

**`3.0.0`.** RFC 048 is titled *the `3.0` surface*, the release is breaking, semver leaves no choice, and the
migration guide already says `3.0` throughout. **Prep picks the date, not the number.** (My slice 3 handoff
said otherwise; that sentence was wrong and the slice 3 review corrects it.)

## 2. Three RFCs move to `done/` — the opposite of the last two releases 🛑

`2.8.0` and `2.9.0` each moved **none**, and both handoffs told you so in bold. **This one moves three.**
I am flagging the reversal because "no moves" is now the habit.

| RFC | Why it is complete |
|---|---|
| **048** | All three slices landed: `29d179a`, `d665aea`/`e9ef8c4`, `6875cf8`/`51354ff` |
| **041** | §9's decision was *deprecate in a minor, remove at `3.0`*. Slice 1 removed the three alias modes and the five inert options. **Nothing of it is left** |
| **039** | Half A shipped in `2.4.0`. Half B asked for one result type across the bindings, Rust's bulk using it, and an option surface showing only what works — **slices 1 and 2 delivered all three** |

`rfcs/README.md` updated **in the same commit**, three rows under Implemented reading `3.0.0`, references
swept. **`rfcs/accepted/` and `rfcs/proposed/` both end up empty** — check that and say so; it is the first
time and it is worth noticing rather than discovering later.

Handoffs stay frozen with the `accepted/` paths they were written with, as RFC 040's did.

## 3. Who breaks — the reverse-dependency check 🛑

RFC 022 set the rule when `alloc_counter` was removed: *"That check must be re-run, not cited."* This is a
much larger removal and the check has never been run against it.

**crates.io lists 10 reverse dependencies of `mdka`; four are ours. The six others:**

```
htm_md            bigquery-functions    elvish-core
threadcat         htmlmd-core           zapmyco-tools
```

**For each: does it use anything `3.0.0` removes?** Fetch the published `.crate`, and grep its source for
the three mode names, the six option fields, `ConvertResult`, and the tuple-returning
`html_files_to_markdown`. Report a table: crate, version, what it uses, **breaks or does not**.

**This changes nothing about the release** — we are not holding `3.0.0` for a downstream crate. It changes
what the release notes can honestly say, and it tells us whether the migration guide is being written for
six people or none.

**If one of them breaks, say so plainly and do not contact anyone** — outward communication is the owner's
decision, always.

## 4. The CHANGELOG has the biggest job it has ever had

One dated `## [3.0.0]` section. It must:

1. **Open with what does not change: no conversion output moves, not one byte**, asserted against published
   `2.9.0`. That is the fact that decides how much work a reader has.
2. **Link the migration guide** — `docs/src/migration-3.0.md` — near the top, not in a footnote. The guide
   is the deliverable; the CHANGELOG's job is to route people to it.
3. **List every removal and every changed signature**, grouped by binding, briefly. Not a second migration
   guide: enough that a reader recognises whether they are affected, then the link.
4. **Say that Node callers are touched most**, and why.
5. **`unwrap_unknown_wrappers`**, explicitly, on the assumption the reader never saw its warning — RFC 048
   §8.10. It was deprecated in `2.9.0` and removed a day later.
6. **Carry §3's finding** if there is one: if a named public dependent breaks, the release notes should not
   pretend otherwise.

## 5. What is not yours

- **The bekoedit letter.** The owner deferred all correspondence to this release
  (2026-09-25); it must carry `2.6.0` through `3.0.0` and lead with their call sites, not with conversion.
  **Architect's to draft, owner's to send.**
- The tag, the publishers, the registries, the GitHub release.
- Any code, test or documentation change. **If prep turns up a defect, report it** — do not fix it.

## 6. What to expect

- **`release artifact gate` should be GREEN**, fourth release running: prep touches no README Quick Start
  block, no artifact contract, no build matrix.
- **`crates package gate` should be GREEN** and this is the first release since its cache fix (`e9ef8c4`).
  **It is also the first time the workspace version moves while the gate's key is `-v2-`.** The version bump
  does not change `Cargo.lock`'s dependency set but does change the workspace members' versions — if the
  gate goes red, **stop and report**; do not assume it is the old staleness.
- `npm install gate` and `pypi published gate` describe published `2.9.0`. Expected.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **3.0.0**; the assertion clean.
2. One dated `## [3.0.0]` doing all six jobs in §4.
3. RFC 039, 041 and 048 in `rfcs/done/` with `3.0.0`; index updated **in the same commit**; references swept;
   **`accepted/` and `proposed/` both empty**, stated.
4. `ROADMAP.md` reflects the release. The open items that survive it — the `FileOutcome` repr is **done**, so
   check RFC 048's criteria off; the performance page's staleness and the `ci.yaml` cache hazard remain.
5. §3's table, with the command used to fetch each crate.
6. Tree clean; `HEAD` and `origin/main` agree.
7. **Eight workflows green on the full SHA** — `gh run list --commit <full-sha>`; an abbreviated SHA returns
   empty silently.
8. Counts unchanged, each with its command: 629 / 59 / 25 / 91.

## 8. Report, then stop

`.git-exclude/review-request/3.0.0-prep/README.md`, leading with §7.7 and §3's table. **Then stop** — I raise
the checkpoint, the owner answers, I tag. **Name who approved anything you push.**
