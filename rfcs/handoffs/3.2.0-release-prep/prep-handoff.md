# Developer Handoff — `3.2.0` release preparation

**Authorised.** Owner, 2026-10-01 — RFC 051 accepted for `3.2.0`, kept apart from `3.1.1` on purpose.
**Preparation is yours; the cut is not.**
**Milestone.** `3.2.0` — a minor. **This one changes output for every user with an ordinal.**
**Prepared.** 2026-10-01
**Baseline.** `9d192b9` — **660** Rust, 60 Node, 25 loader, 93 Python; **eight workflows green on the
full SHA**, tree clean, `HEAD` and `origin/main` agree.
**Scope.** Version, `CHANGELOG.md`, **one RFC move**, `ROADMAP.md`. **No code, no test, no doc
rewrite** — RFC 051's slice is reviewed and accepted as it stands.
**Checklist.** `RELEASE-CHECKLIST.md` §1.

---

## 1. The version number is not open

**`3.2.0`.** A minor, and the minor-ness is the point: unlike `3.1.1`, **this release changes output
without anyone opting in.** `1<sup>st</sup>` becomes `1st` where it was `1ˢᵗ`. Say that plainly and
early in the entry — a reader skimming for "does this move my output" must not have to reach
paragraph three.

## 2. One RFC moves to `done/`

**RFC 051 → `rfcs/done/051-typographic-superscripts.md`, release `3.2.0`.** Index updated in the same
commit, references swept. After the move **both `accepted/` and `proposed/` are empty** — state that.

## 3. The CHANGELOG's jobs 🛑

One dated `## [3.2.0]`, doing four things:

1. **What changes, concretely and first.** `1<sup>st</sup>` → `1st` (was `1ˢᵗ`), same for `nd`/`rd`/
   `th` after a digit, and the ordinal indicators `º`/`ª` → `1º 2ª` (was `1^(º)`).
2. **What does not.** Every exponent and every subscript: `x²`, `10⁻⁹`, `10ⁿ`, `2^(n − 1)`, `H₂O`.
   Name `10<sup>n</sup>` specifically — it is the shape a reader will worry about.
3. **The rule**, in its own words: *render notation, flatten typography* — a superscript is notation
   when flattening changes the meaning, typography when it does not.
4. **The French limit**, as deliberate: `1<sup>er</sup>` is unchanged, because `10<sup>e</sup>` is a
   legitimate exponent.

🛑 **Do not cite the 417-occurrence corpus as evidence that this release helps.** It is zero — no
occurrence in it changed — and that is the correct result: encyclopedia and technical prose writes
*"19th century"* as text, never as `<sup>` markup. **Cite it only for what it showed:** that no
technical document's output moves. The reason for the change is the inconsistency — `1ˢᵗ` in English
but `1^(º)` in Spanish for the same construct, decided by Unicode's glyph coverage rather than by
meaning — plus bekoedit's report that ordinals are common in pasted web text. Those carry it; the
corpus does not.

**Credit bekoedit** in the register `3.1.0` and `3.1.1` used. There is still no "thanks to" line
convention in this file and you should not invent one.

## 4. `ROADMAP.md`

Reflect the release. Check RFC 051's criteria off. **Per `RELEASE-CHECKLIST.md` §1, update
`**Current version.**` to the latest *shipped* release — `3.1.1`, tag `f65030f`** — not `3.2.0`,
which is not shipped until we tag it. This is the rule's first prep since it was written.

**Leave open:** RFC 012 §9's performance-page standing; the `<div style="font-weight:700"><h2>`
observation (an observation, not a commitment); and the `1<sup><i>st</i></sup>` → `1ˢᵗ` limit
recorded in `.git-exclude/reviewed/051-typographic-superscripts/` — likewise an observation.

## 5. What is not yours

- **The bekoedit letter** telling them this shipped, **and that the fixture they pinned will now
  fail** — they pinned it deliberately to notice. Architect's to draft, owner's to send.
- The tag, the publishers, the registries, the GitHub release.
- Any code, test or documentation change. **If prep turns up a defect, report it** — do not fix it.

## 6. What to expect

- **`crates package gate` green** through the bump, as at `3.1.1`. A red is something new.
- **The allocation gate is version-independent.** A bump must not move it; if it does, report rather
  than re-baseline.
- **`release artifact gate` may be RED during prep**, acceptable only for contract/README mismatches
  from this same commit.
- `npm install gate` and `pypi published gate` describe published `3.1.1`. Expected.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **3.2.0**; the assertion clean.
2. One dated `## [3.2.0]` doing all four jobs in §3, **without** citing the corpus as benefit.
3. RFC 051 in `rfcs/done/` with `3.2.0`; index updated in the same commit; **`accepted/` and
   `proposed/` both empty**, stated.
4. `ROADMAP.md` per §4, including the `**Current version.**` line at `3.1.1`.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — an abbreviated SHA returns empty silently.
7. Counts, each with its command: **660** / 60 / 25 / 93.

## 8. Report, then stop

`.git-exclude/review-request/3.2.0-prep/README.md`, leading with §7.6. **Then stop** — I raise the
checkpoint, the owner answers, and the owner and I cut.
