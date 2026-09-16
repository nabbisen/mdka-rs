# Developer Handoff — RFC 007 · English-only public surface

**Governing RFC.** [RFC 007](../../accepted/007-english-only-public-surface.md)
**Rule.** `.git-exclude/rules/project-instructions-general-common.md` — *"Use **English** for all documentation and code comments."*
**Review that produced it.** `.git-exclude/reviewed/rules-compliance-2026-09-16/README.md`
**Milestone.** M2b → `2.2.2`
**Priority.** P1
**Prepared.** 2026-09-16

---

## 0. Ready to start

No preconditions. Independent of RFC 023, 026 and 027 — nothing blocks this and
it blocks nothing. **One coordination point with RFC 023**, §5.

## 1. Purpose

Translate every Japanese comment a **user** can see. Internal comments are
RFC 013, in M4, and are out of scope here.

## 2. What is in scope, measured

Counted across tracked files on 2026-09-16:

| File | Japanese lines | Reaches the user via |
|---|---|---|
| `cli/src/main.rs` | 38 | `mdka --help` (the `const USAGE` string) and the `//!` module docs |
| `src/options.rs` | 38 | docs.rs — `pub mod options` |
| `src/lib.rs` | 25 | docs.rs — crate root |
| `python/src/lib.rs` | 16 | PyO3 `__doc__` |
| `node/src/lib.rs` | 10 | **npm**, via generated `index.d.ts`, in editor tooltips |
| `python/mdka/__init__.py` line 2 | 1 | `help(mdka)` |

**128 lines across six files.**

## 3. ⚠ Two traps — read before running any sweep

### 3.1 `README.md:20` and `docs/src/introduction.md:3` are compliant. Do not touch.

> `"ka" means "化 (か)" pointing to conversion.`

That is Japanese **as subject matter**, inside an English sentence, explaining
where the project's name comes from. It satisfies the rule. A CJK grep-and-
replace would delete the etymology — and a CJK grep is the obvious way to
approach this slice.

### 3.2 `node/index.d.ts` is generated. Editing it is undone at the next build.

It comes from `node/src/lib.rs` through napi codegen. Fix the Rust doc comments,
then regenerate.

**Regenerate in its own commit and read the whole diff.** This is the same path
that carried the stale-`2.0.2` version strings (R-02): `npm run build` output
depends on the locally installed napi-rs, so it can bring churn beyond your
change. Confirm the diff contains only your translated comments.

## 4. Out of scope

- **Private modules** — `renderer`, `traversal`, `utils`. Verified against
  `src/lib.rs`: only `options` and `alloc_counter` are `pub mod`, so no private
  module's doc comments reach docs.rs. **RFC 013, M4.**
- **`src/alloc_counter.rs`** — `#[deprecated]`, regains `#[doc(hidden)]` under the
  RFC 022 correction, removed at `2.4.0`. Translating it is work with a
  two-release lifespan.
- `version.sh`, `python/test_mdka.py`, `tests/`, `benches/`, `examples/`,
  `node/test.js` — repository only. **RFC 013.**
- **Any behaviour change.** This is translation. No API, no logic, no
  restructuring, no renaming.

### Scope boundary, per RFC 027 Rule 2

The boundary is *"can a user of the published artifacts see it?"* Everything on
the user's side is here; everything on ours is RFC 013, which has a milestone
and a number. Nothing in the repository's Japanese is left without an owner.

## 5. Coordination with RFC 023 — same release, same content

RFC 023 is correcting `docs/src/getting-started/usage-cli.md`'s option table.
This slice rewrites `mdka --help`. **They describe the same options and must
agree.**

Whichever lands second must diff the two and reconcile. If RFC 023 has already
landed, take its wording — it will have been reviewed against `--help` as it then
was. Say in your review request which order it happened in.

## 6. How to translate

**Meaning over literalism.** Several of these comments explain *why*, and a
mechanical rendering that loses the reasoning is worse than leaving the Japanese.

- **`--help` is user-facing copy, not a comment.** It should read as English
  written for users, not as translated Japanese.
- Where a comment describes something already documented in `docs/src/`, prefer
  the published wording over a fresh translation.
- Where a comment is **load-bearing** — the `anchor_before` note in
  `renderer.rs`'s neighbourhood, `emit_pending_prefix`'s contract, the
  `version.sh` node_modules warning's equivalents — the English must carry the
  same warning with the same force. These exist because someone got it wrong
  once.

## 7. Required verification

Per RFC 027 Rule 3, state for each item whether it ran against the workspace tree
or an installed artifact.

1. **CJK scan returns zero** on the six in-scope files:

   ```
   grep -rnP '[\x{3040}-\x{30ff}\x{4e00}-\x{9fff}]' \
     cli/src/main.rs node/src/lib.rs python/src/lib.rs \
     python/mdka/__init__.py src/options.rs src/lib.rs
   ```

2. **The two excluded lines are still present** — `README.md:20` and
   `docs/src/introduction.md:3`. Show them.
3. `mdka --help` output, pasted in full, and diffed against `usage-cli.md`.
4. `node/index.d.ts` regenerated in its own commit; diff explained.
5. **Conversion output byte-identical** — the existing corpus, unchanged.
6. Test count unchanged; fmt and clippy clean; `mdbook build` clean.
7. Private modules untouched — `git diff` on `src/renderer.rs`,
   `src/traversal.rs`, `src/utils.rs` shows nothing.

## 8. Prohibited shortcuts

- Do not run an unreviewed CJK sweep. §3.1 is why.
- Do not edit `node/index.d.ts` directly.
- Do not translate private modules — that is RFC 013's scope and taking it here
  makes both slices harder to review.
- Do not change any behaviour, name or signature.
- Do not translate `src/alloc_counter.rs`.

## 9. Known risks

| Risk | If it happens |
|---|---|
| A sweep removes the name etymology | Caught by §7.2. If you notice it before that, say so — it means the sweep was wider than intended. |
| Regeneration brings unrelated churn | Own commit, whole diff read, reported. Same discipline as R-02. |
| `--help` and `usage-cli.md` diverge | §5. Diff them; do not assume. |
| A translated comment loses its warning | §6. If a comment's English feels weaker than its Japanese, say so rather than shipping it. |
| Japanese turns up somewhere not in §2 | Report it. The count was measured on 2026-09-16 and the tree moves. |

## 10. Acceptance checklist

- [ ] Zero CJK in the six in-scope files
- [ ] `README.md:20` and `docs/src/introduction.md:3` unchanged
- [ ] `mdka --help` is English and matches `usage-cli.md`
- [ ] `node/index.d.ts` regenerated in its own commit, diff explained
- [ ] Private modules untouched
- [ ] `src/alloc_counter.rs` untouched
- [ ] Conversion output byte-identical; test count unchanged
- [ ] fmt, clippy, `mdbook build` all clean

## 11. Escalate rather than decide

Stop and raise if: a comment's meaning is unclear enough that translating it
would be guessing; the regeneration diff is not explainable; `--help` and
`usage-cli.md` disagree in substance rather than wording; or you find Japanese in
a published surface §2 does not list.
