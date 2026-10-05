# Developer Handoff — `3.3.0` release preparation

**Authorised.** Owner, 2026-10-06. **Preparation is yours; the cut is not.**
**Milestone.** `3.3.0` — a minor. One new option, **off by default, output unchanged when off.**
**Prepared.** 2026-10-06
**Baseline.** `5a88739` — **675** Rust, 61 Node, 25 loader, 95 Python; **eight workflows green on
the full SHA**; tree clean, `HEAD` and `origin/main` agree.
**Scope.** Version, `CHANGELOG.md`, **one RFC move**, `ROADMAP.md`. **No code, no test, no doc
rewrite** — RFC 052's slice is reviewed and accepted as it stands.
**Checklist.** `RELEASE-CHECKLIST.md` §1.

---

## 1. The version number is not open

**`3.3.0`.** A new public field, flag and keyword on four surfaces — additive, nothing removed.
Minor. **Option off is byte-identical to `3.2.0`**: 108 comparisons against the published binary, 0
differing. Say that in the entry; it is what an upgrading reader needs first.

## 2. One RFC moves to `done/`

**RFC 052 → `rfcs/done/052-backslash-hard-breaks.md`, release `3.3.0`.** Index updated in the same
commit, references swept. Afterwards **`accepted/` and `proposed/` are both empty** — state it.

## 3. The CHANGELOG's jobs

One dated `## [3.3.0]`, doing four things:

1. **The option**, named on all four surfaces: `backslash_hard_breaks` (Rust, Python),
   `backslashHardBreaks` (Node), `--backslash-hard-breaks` (CLI). Off by default; with it off,
   output is unchanged from `3.2.0`.
2. **The trade-off, both directions** — two trailing spaces are stripped by many editors and flagged
   by `markdownlint` MD009; the backslash form is **not understood by Python-Markdown**, the engine
   behind MkDocs. A reader must be able to choose. Point at
   `docs/src/api/options.md`; do not restate the whole table.
3. **The run case, which is the substantive part.** `a<br><br>b` under the default writes a
   whitespace-only line, which CommonMark reads as a blank line, so **the run splits into two
   paragraphs and every break is lost.** With the option on, the run stays one paragraph with every
   break kept. **Say plainly that the default is unchanged** and that this is therefore a fix
   available only to callers who opt in.
4. **Where it does not apply:** headings, end of block, table cells, `<pre>`, code spans.

**Credit bekoedit** in the register `3.1.0`/`3.1.1`/`3.2.0` used — woven into the prose, no "thanks
to" line invented. This closes the last item on their list.

## 4. `ROADMAP.md`

- **`**Current version.**` → `3.2.0`, tag `3ba2cf9`** — the latest *shipped* release, per
  `RELEASE-CHECKLIST.md` §1. 🛑 **Use the commit, not `git rev-parse 3.2.0`**, which returns the
  annotated tag object (`e934c2a`) and is not what the line means. `git rev-parse '3.2.0^{}'`.
- A `3.3.0` paragraph in the established "prepared …, not yet tagged" form.
- **Add a new open item**, which is not from this slice:
  > **`clippy::clone_on_copy` in `python/src/lib.rs` is a future-CI timebomb.** It fires on clippy
  > `0.1.99` and not on the GitHub runner's current stable, which is why `main` is green. A runner
  > image update will turn CI red on an unrelated push. Pre-existing, reproduced at `a8f8776` in a
  > clean worktree.
- **Keep open:** the default's run defect (`a<br><br>b` → two paragraphs with the option off —
  pre-existing, pinned by `the_default_run_is_unchanged`, needs its own RFC), RFC 012 §9's
  performance-page standing, and the `<div style=bold><h2>` and `1<sup><i>st</i></sup>` observations.

## 5. What is not yours

- **The bekoedit letter** — their last item is closed and their list is empty. Architect's to draft,
  owner's to send.
- The tag, the publishers, the registries, the GitHub release.
- Any code, test or documentation change. **If prep turns up a defect, report it** — do not fix it.

## 6. What to expect

- **`crates package gate` green** through the bump. A red is something new.
- **The allocation gate is version-independent** and must not move. It earned its keep this round by
  catching a real `+48 B` default-path regression; if a version bump moves it, report rather than
  re-baseline.
- **`release artifact gate` may be RED during prep**, acceptable only for contract/README mismatches
  from this same commit.
- `npm install gate` and `pypi published gate` describe published `3.2.0`. Expected.

## 7. Criteria

1. `sh version.sh --list`: all four crates at **3.3.0**; the assertion clean, and
   `node/package-lock.json` shows a **4-line** diff (both version fields), as at `3.2.0`.
2. One dated `## [3.3.0]` doing all four jobs in §3, crediting bekoedit.
3. RFC 052 in `rfcs/done/` with `3.3.0`; index updated in the same commit; **`accepted/` and
   `proposed/` both empty**, stated.
4. `ROADMAP.md` per §4, including the corrected `3.2.0` commit SHA and the new clippy item.
5. Tree clean; `HEAD` and `origin/main` agree.
6. **Eight workflows green on the full SHA** — an abbreviated SHA returns empty silently.
7. Counts, each with its command: **675** / 61 / 25 / 95.

## 8. Report, then stop

`.git-exclude/review-request/3.3.0-prep/README.md`, leading with §7.6. **Then stop** — I raise the
checkpoint, the owner answers, and the owner and I cut.
