# Developer Handoff — `3.1.1` release preparation

**Authorised.** Owner, 2026-10-01 — RFC 050 accepted and `3.1.1` authorised in the same decision.
**Preparation is yours; the cut is not.**
**Milestone.** `3.1.1` — a patch. One defect fix, inside an option that is off by default.
**Prepared.** 2026-10-01
**Baseline.** `fe23c79` — **654** Rust, 60 Node, 25 loader, 93 Python; **eight workflows green on the
full SHA**, tree clean, `HEAD` and `origin/main` agree.
**Scope.** Version, `CHANGELOG.md`, **one RFC move**, `ROADMAP.md`. **No code, no test, no doc
rewrite** — RFC 050's slice is reviewed and accepted as it stands.
**Checklist.** `RELEASE-CHECKLIST.md` §1.

---

## 1. The version number is not open

**`3.1.1`.** A patch, and the patch-ness is the point: with `emphasis_from_style` off — the default —
**this release cannot change anyone's output.** I verified that against the published `3.1.0` binary,
not only against goldens: 108 comparisons across 54 documents in both modes, 0 differing.

**Say that in the CHANGELOG in those terms.** It is the whole reason this is a separate release
rather than part of `3.2.0`.

## 2. One RFC moves to `done/`

**RFC 050 → `rfcs/done/050-tag-defaults-and-computed-style.md`, release `3.1.1`.** Index updated in
the same commit, references swept, `Milestone.` line already reads `3.1.1`.

🛑 **RFC 051 stays in `accepted/`.** It is `3.2.0` and **must not appear in this release in any form** —
not in the CHANGELOG, not as a "coming next" line. See §5.

## 3. The CHANGELOG's jobs

One dated `## [3.1.1]`, doing three things:

1. **What was wrong**, concretely: with `emphasis_from_style` on, a `style` that merely **restated a
   tag's own default** was read as new emphasis — so `<h1 style="font-weight:700">H</h1>` became
   `# **H**`, and `cite`/`address`/`var`/`dfn` with `font-style: italic` became italic. Give the
   heading example; it is the one that matters.
2. **Why it mattered**, in one sentence: clipboard HTML carries **computed** styles, so every element
   arrives restating its own defaults — the exact input `emphasis_from_style` exists for.
3. **That nothing else moved**: option off is byte-identical to `3.1.0`, with the 108-comparison
   figure above if you want a number.

**Credit bekoedit.** They raised it as *"a heads-up, not a report"* before we had seen it. Name them
the way the project has named outside reports before — check how `2.4.2` and `2.7.0` did it and match.

**Do not** describe the ordinal change, which is not in this release.

## 4. `ROADMAP.md`

Reflect the release. Check RFC 050's criteria off. **Leave open:** RFC 051 (`3.2.0`), RFC 012 §9's
performance-page standing, and the `<div style=bold><h2>` observation recorded in
`.git-exclude/reviewed/050-tag-defaults/` — that one is an **observation, not a commitment**; if you
name it, name it as such.

## 5. What is not yours

- **RFC 051 / `3.2.0`.** Its handoff is blocked until this tag exists, and nothing about it belongs
  in this release.
- **The bekoedit letter** telling them the fix shipped. Architect's to draft, owner's to send.
- The tag, the publishers, the registries, the GitHub release.
- Any code, test or documentation change. **If prep turns up a defect, report it** — do not fix it.

## 6. What to expect

- **`crates package gate` should be GREEN.** The version bump gives every `target/package/` path a
  new name, and the stale-source defect is fixed and proven. **A red here is something new — stop and
  report.**
- **The allocation gate is version-independent.** A bump must not move it. If it does, that is a real
  finding; report it rather than re-baselining. It is also the first version bump since the
  minimum-over-5 fix.
- **`release artifact gate` may be RED during prep**, as at most releases, and only acceptably so for
  contract/README mismatches arising from this same commit.
- `npm install gate` and `pypi published gate` describe published `3.1.0`. Expected.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **3.1.1**; the assertion clean — **and it should now be
   genuinely clean**, since the lockfile scoping fix landed. If it false-positives again, that is a
   regression in that fix; report it.
2. One dated `## [3.1.1]` doing all three jobs in §3, crediting bekoedit, silent on ordinals.
3. RFC 050 in `rfcs/done/` with `3.1.1`; index updated in the same commit; **RFC 051 still in
   `accepted/`**; `proposed/` empty, stated.
4. `ROADMAP.md` per §4.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — `gh run list --commit <full-sha>`; an abbreviated SHA
   returns empty silently.
7. Counts, each with its command: **654** / 60 / 25 / 93.

## 8. Report, then stop

`.git-exclude/review-request/3.1.1-prep/README.md`, leading with §7.6. **Then stop** — I raise the
checkpoint, the owner answers, and the owner and I cut.
