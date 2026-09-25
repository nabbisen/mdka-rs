# Developer Handoff — `2.7.0` release preparation

**Authorised.** Owner, 2026-09-25 — the release is authorised; **preparation is yours, the cut is not.**
**Milestone.** The `2.7.0` release.
**Prepared.** 2026-09-25
**Baseline.** `148b158` — 621 Rust (`cargo test --workspace`; plain `cargo test` gives 594 and is the wrong command), 42 Node, 25 loader, 91 Python; eight workflows green
**Scope.** Version, `CHANGELOG.md`, RFC lifecycle moves, `ROADMAP.md`. **No code, no test, no workflow change.**
**Checklist.** `RELEASE-CHECKLIST.md` §1 — tracked in the repository since RFC 047. This handoff covers §1 only.

---

## 1. This is the first time prep is yours

The owner split it on 2026-09-25: **the dev team prepares, the owner and architect cut.** So this handoff
covers everything up to the pre-tag checkpoint, and **nothing past it**.

**Not yours, and not by accident:** creating the tag, pushing it, dispatching or re-running any release
workflow, publishing to any registry, creating a GitHub release. If prep looks finished and the tag has not
appeared, that is correct — the checkpoint is a deliberate pause.

## 2. What `2.7.0` contains

**Minor, because conversion output changes.** Twelve commits since `2.6.0`
(`git rev-list --count 2.6.0..HEAD`).

| RFC | What a user sees |
|---|---|
| **044** | `<b><em>q</em>a</b>` no longer loses the italic and leaves a literal `_`. Was `**_q_a**`, parsing as `strong("_" "q_a")`; now `***q*a**`. **Measured 0 of 424** bold openings on 42 pages of published prose — real, silent, and rare |
| **046** | Nothing directly. The manual consumer pass is retired and replaced by the `release artifact gate` |
| **047** | The README's prebuilt-binary Quick Start now has **per-platform blocks** — Linux, macOS, Windows — and the gate executes each one verbatim against the published archives. A Windows reader previously had no command at all |

RFC 039 and RFC 041 stay in `rfcs/accepted/`: both are `3.0` and unscheduled.

## 3. The work

1. **`sh version.sh --update 2.7.0`.** Confirm its own post-update assertion reports no manifest retaining
   `2.6.0`, and that `node/index.js` was rewritten (52 occurrences last time).
2. **`CHANGELOG.md`** — a dated `## [2.7.0] - <date>` section describing **user-visible effect**, not commit
   subjects. Lead with RFC 044, since it is the only behaviour change; say what the old output was and what
   it parsed as, because a consumer needs that to diff against. RFC 047's README blocks are worth a line;
   RFC 046 is internal and needs at most one.
3. **RFC moves to `rfcs/done/`** — **044, 046, 047**, with `rfcs/README.md` updated **in the same commit**
   and the "Shipped in" column reading `2.7.0`. Sweep inbound references
   (`grep -rn '04[467]-[a-z]' rfcs/ ROADMAP.md docs/ README.md`). **Handoffs are frozen** — they keep the
   `rfcs/accepted/…` paths they were written with, as RFC 040's did; do not edit them.
4. **`ROADMAP.md`** — what `2.7.0` contains, and strike through the carried-forward rows it closes rather
   than deleting them.
5. **Everything in one push**, so CI runs against the commit that will be tagged.

**There are no held documentation patches this time.** The three under `.git-exclude/review-request/*/` are
renamed `docs-changes-APPLIED-at-*.patch` and are already in the tree; do not re-apply them.

## 4. What to expect, and what would be a finding

**`release artifact gate` should stay GREEN.** It is red on a prep commit that changes `README.md`'s Quick
Start, `artifact-contract.toml` or the build matrix — this prep changes none of them. `RELEASE-CHECKLIST.md`
§1 records that as a standing explanation; **this is the first release where it should not fire, and if it
goes red that is a finding, not the explanation.** Say so rather than waving it through.

**`npm install gate` describes the last published release**, so it will report `2.6.0`. That is the other
standing explanation and it is expected.

**Everything else green**, including `crates package gate`, which has no standing exception.

## 5. Criteria

1. `sh version.sh --list` shows all four crates at **2.7.0**, and the script's assertion reported clean.
2. `CHANGELOG.md` has exactly one dated `## [2.7.0]` section, describing effect.
3. 044, 046 and 047 in `rfcs/done/`, index updated **in the same commit**, references swept, `proposed/` and
   the remaining `accepted/` entries correct (039 and 041 stay).
4. `ROADMAP.md` reflects the release.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **All eight workflows green on the full SHA** — `gh run list --commit <full-sha>`; an abbreviated SHA
   silently returns empty and has bitten this project twice.
7. Test counts unchanged, each with its command: `cargo test --workspace` **621**, `node test.js` 42,
   `node test-loader.js` 25, `pytest test_mdka.py` 91.

## 6. Report, then stop

Report to `.git-exclude/review-request/2.7.0-prep/README.md`, leading with §5.6 — the full SHA and the eight
conclusions, since that is what the checkpoint is decided on.

**Then stop.** The architect raises the pre-tag checkpoint with the owner; the owner gives the go-ahead; the
architect tags. You will see the tag appear without being asked to do anything.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval.

**Use `git commit -F - -- <paths>`, not `git add <paths> && git commit`.** In a shared working tree the
second form commits the whole index, not the paths you named — that is how RFC 047's unreviewed work reached
`main` under a commit message about RFC 044 on 2026-09-25. That was the architect's mistake, and this is the
rule that prevents it.
