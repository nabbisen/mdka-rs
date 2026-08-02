# Developer Handoff — RFC 002 · Governance artifacts

**Governing RFC.** [RFC 002](../../proposed/002-governance-artifacts.md) — Proposed
**Milestone.** M1 · Trustworthy baseline → `2.1.7`
**Position in M1.** Second, after RFC 001.
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 002. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

> The two items in §6 required a project-owner decision. **Both were decided on
> 2026-08-02 and are recorded in place.** Apply them exactly as written; do not
> substitute your own judgement.

---

## 1. Purpose

Add the missing release-history and licensing artifacts, bring the RFC directory
under version control, and make RFC 000 conform to the policy it defines.

## 2. Background

The project has 86 tags spanning 0.1.0 (2024) to 2.1.6 (2026-06-05) and no
changelog. Release history exists only as tags. Downstream consumers on
crates.io, npm, and PyPI have no migration reference. There is no `NOTICE` file,
which the project rules require alongside Apache-2.0. `rfcs/` and `ROADMAP.md`
exist on disk but are untracked, so the design record is not actually under
version control.

## 3. Applicable requirements

RFC 002 §Goals: a changelog covering every published version; a `NOTICE` file;
`rfcs/` under version control; RFC 000's `Status` field conformant with its own
policy.

## 4. Change scope

| Path | Change |
|---|---|
| `CHANGELOG.md` | New file |
| `NOTICE` | New file |
| ~~`LICENSE`~~ | **Removed from scope** — owner already applied it in `d499f65`. Do not touch. |
| git tags | One local deletion only, see §6.2 |
| `rfcs/done/000-rfc-lifecycle-policy.md` | `Status` field only |
| `README.md` | Two rows added to the existing "Learn More" table |
| git index | Track `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, `NOTICE` |

## 5. Non-change scope — do not touch

- All source: `src/`, `cli/`, `node/`, `python/`, `tests/`, `benches/`,
  `examples/`. This RFC changes no code whatsoever.
- All of `docs/` — RFC 003 owns it.
- `tests/utils/` — RFC 004 owns its removal.
- Any workflow file, including the new `ci.yaml` from RFC 001.
- Any manifest: `Cargo.toml`, `package.json`, `pyproject.toml`. No version bump
  under this RFC — `2.1.7` is set at release, not here.
- The body of `LICENSE`. The Apache-2.0 text is verbatim and must stay verbatim.
  Only the copyright line is in scope, and only under §6.1.
- Existing RFC numbering. Do not renumber, reorder, or reformat RFCs 001–004.
- Japanese comments anywhere — RFC 007 and RFC 013 own them.

## 6. Owner decisions

### 6.1 Copyright year — DECIDED: `2024`. LICENSE already done.

**Owner decision, 2026-08-02: use the start year, `2024`.**

> **UPDATE 2026-08-02 — the project owner has already applied this to `LICENSE`
> directly, in commit `d499f65` ("Revert copyright year"). `LICENSE` now reads
> `Copyright 2024 nabbisen`.**
>
> **Do not edit `LICENSE`.** It is correct and out of scope. Verify only:
> `grep 'Copyright 2024 nabbisen' LICENSE` returns the line, and the Apache-2.0
> body text is otherwise untouched.
>
> What remains for you is `NOTICE` alone — it must carry the identical value so
> the two cannot drift apart.

Background, for the record — the repository currently contradicts itself:

| Commit | Date | Change |
|---|---|---|
| `07dfbb3` | — | `Copyright 2024- nabbisen<nabbisen@scqr.net>` → `Copyright 2024 nabbisen`, with the message "Update copyright to start-year-only format. Annual updates not required as copyright is automatic." |
| `a4e9009` | 2026-08-01 | `Copyright 2024 nabbisen` → `Copyright 2026 nabbisen`, message "Update copyright year" |

The most recent commit reversed the convention established one commit earlier.
The project's first commit is `ceffbe2`, dated **2024-01-04**, so under
start-year-only the correct value is `2024`. `LICENSE` currently reads `2026`,
which is neither the start year nor a range.

Change only the copyright line. The Apache-2.0 body text stays verbatim.

### 6.2 Malformed tag — DECIDED: delete locally only

A tag named ``2.0.` `` exists between `2.0.3` and `2.1.0`. Verified 2026-08-02:

| Property | Value |
|---|---|
| Exact name | `2.0.` followed by a backtick — bytes `32 2e 30 2e 60` |
| Type | **Annotated** tag, not lightweight |
| Points at | `d21467f`, 2026-04-15, "fix thiserror version; docs update" |
| Distinct commit? | Yes — not the same commit as `2.0.3` (`34d02f5`) |

It is a real annotated tag on a real commit, most likely produced by a release
command invoked with a malformed argument on the same day as `2.0.0`/`2.0.1`.

**Owner decision, 2026-08-02: delete the local tag only. Do not touch the
remote.**

```
git tag -d '2.0.`'
```

Quote the name — the backtick is shell-significant in some shells.

> **CORRECTION 2026-08-02 — the two consequences below were wrong.**
>
> The implementer checked and reported that the malformed tag was **never on
> the remote**. Verified at review: `git ls-remote --tags origin` returns 114
> refs and none matches it, and `git fetch --tags` did not reinstate it after
> deletion. It was local-only to this machine.
>
> Both numbered points below are therefore void. The *instruction* — delete
> locally, exclude from the changelog — was correct and unchanged; only my
> stated reason for it was false. Retained struck-through so the correction is
> legible rather than silently rewritten.
>
> Closing detail found at review: `refs/tags/2.0.1^{}` dereferences to
> `d21467fc…`, the same commit the malformed tag pointed at. It was a
> mistyped first attempt at tagging `2.0.1`, re-run correctly moments later.
> No release is missing from the changelog.

~~Two consequences to carry into your review request, because they mean the
underlying issue is *not* closed:~~

1. ~~The remote copy remains authoritative, so **the tag returns on the next
   `git fetch --tags`**. This is a local tidy-up, not a removal.~~
2. ~~The malformed tag therefore still appears in any tag-driven reconstruction
   done from a fresh clone.~~ Exclude it from the changelog regardless, and note
   the exclusion explicitly.

Do not delete, create, or modify any other tag, and do not touch the remote
under any circumstance.

## 7. Required implementation

### Slice 1 — `CHANGELOG.md`

Keep a Changelog format, newest first, one section per published version with
its release date.

**Coverage required:**

| Line | Treatment |
|---|---|
| 2.x — 11 releases, `2.0.0` (2026-04-15) through `2.1.6` (2026-06-05) | One section each, in full |
| 1.x — `1.0.0` through `1.6.9` | One consolidated section summarising the line and the 1.x → 2.0.0 transition |
| 0.x — `0.1.0` through `0.5.1`, plus the early `v`-prefixed tags | One brief line. Pre-1.0 history needs no reconstruction. |

Add an `Unreleased` section at the top carrying M1's changes. It becomes the
`2.1.7` entry at release.

**Deriving the entries.** Release-tag dates and commit ranges:

```
git tag --sort=v:refname | grep -E '^2\.[0-9]+\.[0-9]+$'
git log --oneline <prev-tag>..<tag>
```

Commit counts per 2.x release run from 1 to 10. Most are packaging, CI, and
dependency work rather than library changes — reflect that honestly rather than
inflating it into feature language.

**Two places where history is genuinely hard, and honesty is required:**

1. **`2.0.0` is a single squash.** Commit `e7b2dbd` ("2.0.0 dev (#76)") is 122
   files and ~46,900 insertions — the entire 2.x rewrite in one commit. The
   subject line conveys nothing. Derive the entry from the diff and from
   PR #76, or state plainly what could not be determined. Do not invent a
   feature list.

2. **`2.1.0` is a minor bump with no user-facing feature.** Its range
   (`2.0.3..2.1.0`) contains only benchmark configuration, docs, issue
   templates, and version-tooling fixes. Record what actually happened. Do not
   rationalise it into a feature to justify the minor increment.

Per RFC 002: where a version's intent cannot be established from history with
confidence, **say so explicitly**. An entry reading "packaging and CI fixes;
no library changes identified" is correct and useful. A fabricated summary is
neither.

Every entry states user-visible effect, not commit subject. Sections use
`Added` / `Changed` / `Fixed` / `Removed`.

### Slice 2 — `NOTICE`

Apache-2.0 attribution notice: project name, copyright holder `nabbisen`, and
the copyright year from §6.1.

Keep it short. Per project rules, the full licence text lives in `LICENSE` and
must not be duplicated into `NOTICE` or `README.md`.

### Slice 3 — RFC 000 status field

`rfcs/done/000-rfc-lifecycle-policy.md` line 3 currently reads:

```
**Status.** Implemented
```

Change to:

```
**Status.** Implemented (2.1.6)
```

RFC 000 requires implemented RFCs to carry the version in which the work
shipped. This is the policy applied to itself.

Note that file contains several `**Status.**` lines inside fenced example
blocks, which are illustrative samples. **Change only the one on line 3**, the
document's own metadata header. Leave every example untouched.

### Slice 4 — Version control and README

Add to git: `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, `NOTICE`.

Confirm first that `.gitignore` does not exclude them — as of 2026-08-02 it does
not; the tree is simply unadded.

Add two rows to the existing "Learn More" table in `README.md`, linking
`CHANGELOG.md` and `ROADMAP.md`. Table rows only — no new prose sections. The
project rules require the README stay concise.

## 8. Required tests

None. This RFC changes no code.

`cargo test --workspace` must still report **74 passed, 0 failed** — unchanged,
because nothing you touch is compiled. If the count moves, something outside
scope was modified.

## 9. Required documentation updates

The two README table rows in slice 4. Nothing under `docs/`.

## 10. Compatibility constraints

Zero. No API, no behaviour, no artifact contents change. `NOTICE` is not
currently referenced by any packaging manifest, so adding it does not alter
any published artifact under this RFC.

## 11. Security constraints

When reconstructing the changelog you will read several years of commit
subjects. Before reproducing any of them:

- Do not copy a commit subject verbatim without reading it.
- Confirm no credential, token, internal hostname, or private path was recorded
  in a subject line. If you find one, **stop and report it** — do not reproduce
  it in the changelog, and do not attempt to rewrite history yourself.

No secrets are otherwise involved.

## 12. Prohibited shortcuts

- No invented changelog entries. Mark undetermined history as undetermined.
- No generated-from-commits changelog. The history does not follow a
  conventional-commit format; generation would produce noise. RFC 002 records
  this as a rejected alternative.
- No Apache-2.0 licence text pasted into `README.md` or `NOTICE`.
- No renumbering, reordering, or reformatting of existing RFCs.
- No tag operation other than the single local deletion authorised in §6.2. No
  tag creation, no modification, and nothing touching the remote.
- No version bump in any manifest.
- No copyright value other than `2024` per §6.1.

## 13. Known risks

| Risk | If it happens |
|---|---|
| 1.x history too sparse to summarise | Expected. Scope 1.x to one consolidated entry and state that per-patch history was not reconstructed. |
| PR #76's diff is too large to summarise usefully | Summarise at theme level — "complete rewrite: new traversal engine, conversion modes, workspace split" — and say the entry is reconstructed from the diff rather than from release notes. |
| A commit subject contains sensitive data | Stop and report per §11. |
| `.gitignore` turns out to exclude a target path | Report before editing `.gitignore`; that file is not in scope. |

## 14. Required evidence

1. `CHANGELOG.md` rendered, with the section count stated.
2. Output of `git tag --sort=v:refname` cross-referenced against changelog
   sections — every 2.x release tag accounted for, with the ``2.0.` `` exclusion
   noted per §6.2.
3. `git status` showing `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, `NOTICE` staged.
4. `git diff rfcs/done/000-rfc-lifecycle-policy.md` — one line changed.
5. `cargo test --workspace` — 74 passed, 0 failed.
6. Confirmation that every relative link in `README.md`, `rfcs/README.md`, and
   `ROADMAP.md` resolves.

## 15. Acceptance checklist

- [ ] `CHANGELOG.md` exists with an `Unreleased` section
- [ ] All 11 2.x releases have their own dated section
- [ ] 1.x consolidated section present; 0.x noted briefly
- [ ] Undetermined history marked as undetermined, not invented
- [ ] `2.1.0`'s lack of a user-facing feature recorded honestly
- [ ] `NOTICE` exists, short, no licence text duplicated
- [ ] `NOTICE` reads `Copyright 2024 nabbisen`, matching `LICENSE`
- [ ] `LICENSE` unmodified by you — `git diff LICENSE` empty
- [ ] Local tag ``2.0.` `` deleted; remote untouched; return-on-fetch noted
- [ ] RFC 000 line 3 reads `Implemented (2.1.6)`; example blocks untouched
- [ ] `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, `NOTICE` tracked in git
- [ ] Two rows added to the README "Learn More" table; no new sections
- [ ] `cargo test --workspace` reports 74 passed, 0 failed
- [ ] No file outside §4 modified
- [ ] No tag created, deleted, or modified

## 16. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 002 goals, by number)
3. Changed files — complete list
4. Important implementation decisions
5. Differences from RFC 002, if any, and why
6. **Changelog entries you could not determine, and what you wrote instead**
7. Executed tests and results
8. Evidence per §14
9. Unresolved issues
10. Known limitations
11. Requested review focus

Item 6 is not optional. Reconstructing four years of history will produce gaps,
and the gaps are the part most worth reviewing.

## 17. Escalate rather than decide

Stop and raise it if you find: a commit subject containing sensitive data; a
published version with no corresponding tag, or a tag with no published
artifact; `.gitignore` excluding an in-scope path; or any reason the copyright
question in §6.1 is more complicated than it looks.
