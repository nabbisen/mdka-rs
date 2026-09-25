# Developer Handoff — `2.8.0` release preparation

**Authorised.** Owner, 2026-09-25. **Preparation is yours; the cut is not.**
**Milestone.** The `2.8.0` release — RFC 041's deprecation slice, the runway `3.0` needs.
**Prepared.** 2026-09-25
**Baseline.** `5a87914` — 631 Rust (`cargo test --workspace`; plain `cargo test` is the wrong command and gives a different number), 49 Node, 25 loader, 98 Python; eight workflows green
**Scope.** Version, `CHANGELOG.md`, `ROADMAP.md`. **No code, no test, no workflow, and — see §3 — no RFC moves.**
**Checklist.** `RELEASE-CHECKLIST.md` §1, tracked in the repository. This handoff covers §1 only.

---

## 1. Same split as `2.7.0`

You prepare; the owner and the architect cut. **Not yours:** creating or pushing the tag, dispatching or
re-running a release workflow, publishing anywhere, creating a GitHub release. If prep looks finished and no
tag appears, that is correct — the checkpoint is a deliberate pause.

`2.7.0`'s prep was the first under this split and it was clean. The one thing to repeat deliberately:
**commit with `git commit -F <file> -- <explicit paths>`**, because `version.sh` runs `git add` itself.

## 2. What `2.8.0` contains

**Minor.** 7 commits since `2.7.0` (`git rev-list --count 2.7.0..HEAD` — re-run it; mine will be stale by the
time you read this, as it was last release).

**One slice: RFC 041 §9's deprecations.** `ConversionMode::Strict`, `Semantic` and `Preserve` — byte-for-byte
aliases of `Balanced` — are deprecated on all four surfaces, and the CLI's warnings were given one shape.

**Nothing is removed and no conversion output changes.** That is the point: `3.0` cannot remove a public name
nobody was warned about, and this is the warning.

## 3. No RFC moves this release — read this before following habit 🛑

`2.7.0` moved three RFCs to `rfcs/done/`. **`2.8.0` moves none.**

RFC 041 is **not** finished by this release. Its §9 removal and its §10 `parse_mode` decision are `3.0`,
bound to RFC 039 Half B. **RFC 041 and RFC 039 both stay in `rfcs/accepted/`**, and `rfcs/README.md`'s
Implemented table gains no row.

**What to do instead:** update RFC 041's index row so a reader can see which part shipped — the deprecation
slice in `2.8.0`, the structural half still `3.0`. One row, no move.

If that feels wrong, say so in the report rather than moving anything.

## 4. The work

1. **`sh version.sh --update 2.8.0`.** Confirm its post-update assertion reports no manifest retaining
   `2.7.0`, and that `node/index.js` was rewritten.
2. **`CHANGELOG.md`** — a dated `## [2.8.0]` section describing **user-visible effect**. This release's
   effect is *warnings*, so say concretely: which three names, on which four surfaces, that **output is
   unchanged**, that **nothing is removed yet**, and that they go at `3.0`. Mention that the CLI's
   deprecation warnings now share one shape. **Say what a user should do** — use `Balanced`, or `Minimal` if
   they wanted something genuinely different.
   **One thing worth a line of its own:** a mode read from a *string* — a config file, `--mode`, a binding
   argument — **cannot be warned about at compile time**. Those users need to act now and nothing will
   remind them. `docs/src/api/modes.md:133–138` has the wording; do not promise what `3.0` will do.
3. **`ROADMAP.md`** — what `2.8.0` ships, and strike through what it closes. **Leave RFC 041 §10's
   `parse_mode` question open**; it is a `3.0` constraint, not something this release settles.
4. **One push**, so CI runs against the commit that will be tagged.

## 5. What to expect, and what would be a finding

- **`release artifact gate` should be GREEN.** This prep changes no README Quick Start block, no artifact
  contract, no build matrix. The standing explanation in `RELEASE-CHECKLIST.md` §1 should not be needed for
  the second release running. **If it goes red, that is a finding** — say so rather than reaching for the
  explanation.
- **`npm install gate`** reports on the last *published* release, so it describes `2.7.0`. Expected.
- Everything else green, `crates package gate` included — it has no standing exception.

## 6. Criteria

1. `sh version.sh --list`: all four crates at **2.8.0**; the script's assertion clean.
2. `CHANGELOG.md`: exactly one dated `## [2.8.0]`, covering §4.2 including the string-form line.
3. **`rfcs/done/` is unchanged.** RFC 039 and RFC 041 still in `accepted/`; RFC 041's index row says which
   half shipped.
4. `ROADMAP.md` reflects the release; RFC 041 §10 left open.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — `gh run list --commit <full-sha>`. An abbreviated SHA silently
   returns empty and has bitten this project twice.
7. Test counts unchanged, each with its command: `cargo test --workspace` **631**, `node test.js` **49**,
   `node test-loader.js` **25**, `pytest test_mdka.py` **98**.

## 7. Report, then stop

`.git-exclude/review-request/2.8.0-prep/README.md`, leading with §6.6 — the full SHA and the eight
conclusions, since that is what the checkpoint turns on.

**Then stop.** The architect raises the checkpoint, the owner answers, the architect tags.

**And name who approved anything you push.** Part 2 of the deprecation report set that straight and it should
stay straight: "approved" is a fact about a person, not a state a commit is in.
