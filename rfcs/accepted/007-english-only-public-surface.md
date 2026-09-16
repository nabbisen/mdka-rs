# RFC 007 — English-only public surface

**Status.** Accepted 2026-09-16 — implementer may start
**Tracks.** M2b → `2.2.2`
**Priority.** P1
**Touches.** `cli/src/main.rs`, `node/src/lib.rs` (+ regenerated `node/index.d.ts`), `python/mdka/__init__.py`, `src/options.rs`, `src/lib.rs`.
**Source.** Project rule, `project-instructions-general-common.md` — *"Use **English** for all documentation and code comments."* Reserved since the original roadmap; drafted 2026-09-16 after the owner updated the rules.
**Prepared.** 2026-09-16

## Summary

Every surface a user of `mdka` can see carries Japanese. Translate those to
English. Internal comments are RFC 013.

## The measured facts

Counted across tracked files, 2026-09-16:

| Surface | doc lines | **total CJK** | Seen by |
|---|---|---|---|
| `cli/src/main.rs` — `const USAGE` and `//!` | 12 | **38** | every run of `mdka --help` |
| `src/options.rs` | 38 | **42** | docs.rs — `pub mod options` |
| `src/lib.rs` | 25 | **36** | docs.rs — crate root |
| `node/src/lib.rs` | 10 | **16** | **npm**, via generated `index.d.ts`, in editor tooltips |
| `python/src/lib.rs` | 16 | **20** | PyO3 `__doc__` |
| `python/mdka/__init__.py:2` | 0 | **1** | `help(mdka)` on PyPI |
| | | **153** | |

**Corrected 2026-09-16.** This table first totalled 128, by pulling `cli`'s
*total* and the other four's *doc-comment* counts from two different scans
without labelling either. The file list was right; the arithmetic was not, and
translating to the stated figure would have left 25 lines behind. The
implementer worked to the measured figure and reported the gap.

**A table assembled from two scans needs a column saying which.**

**Not in scope** — private modules `renderer`, `traversal`, `utils`. Verified
against `src/lib.rs`: only `options` and `alloc_counter` are `pub mod`, so no
private-module doc comment reaches docs.rs. They are RFC 013.

`alloc_counter` is excluded: it is `#[deprecated]`, regains `#[doc(hidden)]`
under the RFC 022 correction, and is removed at `2.4.0`. Translating it would be
work with a two-release lifespan.

### The scope boundary, corrected 2026-09-16

This RFC's handoff first set the boundary as *"can a user of the published
artifacts see it?"* **That is unusable.** The PyPI **sdist** contains essentially
the whole source tree — 22 files with Japanese, ~304 lines, including `src/`,
`tests/`, `benches/`, `examples/`, `python/test_mdka.py` and `pyproject.toml` —
so taken literally it drags all of RFC 013 in here.

The workable test is **"is it rendered to a user by a tool in normal use?"** —
`--help`, docs.rs, editor tooltips, `help(mdka)`, the PyPI page.

Measured against the published `2.2.1` **wheel**, which is what `pip install`
actually fetches: its only two CJK occurrences are `mdka/__init__.py` (in scope
here) and `METADATA`'s `"ka" means "化 (か)"`, which is the compliant etymology.
After this RFC the wheel carries zero non-compliant Japanese.

So **`python/pyproject.toml`, `python/example.py` and `examples/*.rs` are
RFC 013** — they ship in the sdist but are never rendered.

## ⚠ Two traps

### 1 · `README.md:20` and `docs/src/introduction.md:3` are correct

> `"ka" means "化 (か)" pointing to conversion.`

Japanese *as subject matter*, in an English sentence, explaining the project's
name. **Do not touch these.** A CJK grep-and-replace sweep would delete the
etymology, and a sweep is the obvious way to approach this RFC.

### 2 · `node/index.d.ts` is generated

It comes from `node/src/lib.rs` via napi codegen. Editing it directly is undone
by the next `npm run build`. Fix the Rust doc comments and regenerate.

That regeneration is the same path that carries the R-02 version-string risk, so
it takes the same discipline: **regenerate in its own commit and read the whole
diff**, confirming nothing but the translated comments changed.

## Design

Translation only. **No behaviour change, no API change, no restructuring.**

- **Meaning over literalism.** Several comments explain *why*, and a mechanical
  translation that loses the reasoning is worse than the Japanese.
- **The CLI `--help` output is user-facing copy**, not a comment. It should read
  as English written for users, not as translated Japanese. Option descriptions
  should match `docs/src/getting-started/usage-cli.md`, which RFC 023 is
  correcting in the same release — coordinate so the two agree.
- Where a comment documents something already described in `docs/src/`, prefer
  the wording already published there over a fresh translation.

## Compatibility

`cli --help` output changes, and doc text changes on docs.rs, npm and PyPI. No
API, no behaviour, no output change from the conversion engine.

Patch-appropriate: documentation only.

## Risks

| Risk | Mitigation |
|---|---|
| A sweep deletes the name etymology | Named above. The two lines are compliant. |
| `index.d.ts` edited directly and lost at next build | Fix `node/src/lib.rs`; regenerate deliberately. |
| Regeneration drags in unrelated churn | Own commit, whole diff read — the R-02 lesson. |
| Translation loses the reasoning a comment carried | Translate meaning, not words. Where a comment is load-bearing — the `anchor_before` note, `emit_pending_prefix`'s contract — the English must carry the same warning. |
| `--help` and `usage-cli.md` diverge | RFC 023 touches the same content in the same release. Diff them against each other before finishing. |

## Acceptance criteria

1. Zero Japanese in `cli/src/main.rs`, `node/src/lib.rs`, `python/src/lib.rs`,
   `python/mdka/__init__.py`, `src/options.rs`, `src/lib.rs`.
2. `node/index.d.ts` regenerated, in its own commit, diff confirmed to contain
   only the translated comments.
3. `mdka --help` output is English and matches `usage-cli.md` exactly.
4. `README.md:20` and `docs/src/introduction.md:3` **unchanged**.
5. Private modules untouched — they are RFC 013.
6. No behaviour change: conversion output byte-identical, test count unchanged.
7. fmt and clippy clean; `mdbook build` clean.

## Verification

A CJK scan over the in-scope files returning zero, plus the two excluded lines
shown still present:

```
grep -rnP '[\x{3040}-\x{30ff}\x{4e00}-\x{9fff}]' \
  cli/src/main.rs node/src/lib.rs python/src/lib.rs \
  python/mdka/__init__.py src/options.rs src/lib.rs
```
