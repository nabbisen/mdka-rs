# Developer Handoff — RFC 003 · Architecture documentation reconciliation

**Governing RFC.** [RFC 003](../../proposed/003-architecture-doc-reconciliation.md) — Proposed
**Milestone.** M1 · Trustworthy baseline → `2.1.7`
**Position in M1.** Last. RFC 004 has landed (`0eadf0a`), which unblocks this.
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 003. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

---

## 1. Purpose

Correct the documentation statements that are factually wrong about the current
engine — and only those. Leave everything RFC 005/006 will rewrite.

## 2. Background

Every claim below was verified against the current build on 2026-08-02, after
RFC 004 landed. These are not inherited notes; they were re-run.

The headline problem: `docs/src/design/architecture.md` documents a five-step
pipeline with a pre-processing stage that serialises to an intermediate HTML
string and a re-parse. **Neither step exists.** `html_to_markdown_with`
(`src/lib.rs:94-97`) parses once and traverses once, with preprocessing applied
inline during traversal (`src/traversal.rs:47-67`). It is the first page a new
maintainer reads, and it describes a pipeline that was removed from the engine
but never from the docs.

## 3. Change scope

Six files. Only the specific claims enumerated in §5.

| Path | Change |
|---|---|
| `docs/src/design/architecture.md` | Pipeline section; workspace layout tree |
| `docs/src/api/elements.md` | Comments claim; always-removed list |
| `docs/src/api/modes.md` | Preserve comments claim; Minimal shell wording |
| `docs/src/api/options.md` | `drop_interactive_shell` prose only |
| `docs/src/design/philosophy.md` | Two separate-pass assertions |

## 4. Non-change scope — do not touch

- **Any `preserve_*` or `drop_presentation_attrs` field description.** These
  describe behaviour that does not exist today but *will* after RFC 005.
  Rewriting them now means rewriting them twice. **RFC 006 owns them.**
- **`options.md:73` and `options.md:97`** specifically. Both contain
  "pre-processed DOM" / "during pre-processing" mechanism language, and both sit
  inside `preserve_ids` / `drop_presentation_attrs` descriptions that RFC 006
  rewrites wholesale. Leave them. Yes, they are inaccurate. They are not yours.
- **The phrase "pre-processing" used as a category of behaviour.** Mode presets
  genuinely do tune preprocessing; that framing is fine. Only claims of a
  separate *pass*, *stage*, *intermediate DOM*, or *re-parse* are wrong.
- Tables, navigation, `SUMMARY.md`, new pages.
- `docs/src/getting-started/installation.md` — its MSRV line was already
  corrected under RFC 001 Slice 1b. Do not revisit.
- **`docs/book/`** — generated output, gitignored. Never hand-edit.
- Any code. This RFC changes no `.rs` file, no manifest, no workflow.
- Japanese comments — RFC 007 and RFC 013 own them.

## 5. Required corrections

Line numbers are as of 2026-08-02 and **will shift as you edit**. Anchor on
content, not on the number.

### 5.1 `architecture.md` — pipeline section

Replace the five-step diagram with the three steps that exist:

```
HTML string
    │
    ▼
[1] Parse        scraper::Html::parse_document()
    │             → html5ever DOM (tolerant HTML5 parsing)
    ▼
[2] Traverse     traversal::traverse(&doc, opts)
    │             → non-recursive DFS over ego-tree, Enter/Leave events
    │             Preprocessing is applied inline during this traversal:
    │               · drops script/style/head/svg/… unconditionally
    │               · drops shell elements when opted in
    │               · unwraps generic wrappers when opted in
    │             Drives MarkdownRenderer
    ▼
[3] Finalise     renderer.finish()
                  → trim trailing whitespace, single trailing newline
```

State **explicitly** that there is no intermediate HTML serialisation and no
second parse. That is a deliberate design property worth recording — the removed
round trip is exactly what the old text described, and the next person to read
this page should not have to rediscover that it is gone.

### 5.2 `architecture.md` — workspace layout tree

Remove the `tests/ └── utils/preprocessor.rs` entry (RFC 004 deleted it). The
actual tree as of `0eadf0a`:

```
src/alloc_counter.rs   src/lib.rs        src/options.rs
src/renderer.rs        src/traversal.rs  src/traversal/tests.rs
src/utils.rs           src/utils/tests.rs

tests/block_elements.rs  tests/common.rs      tests/compat.rs
tests/file_conversion.rs tests/inline_elements.rs tests/robustness.rs
```

Verify against the repository rather than transcribing the above.

### 5.3 `elements.md` — comments claim

Current text is false:

> HTML comments are removed in all modes **except `Preserve`**, where they are
> retained as `<!-- … -->` in the pre-processed DOM

Comments are dropped in **every** mode, `Preserve` included. `src/traversal.rs`
has no `Node::Comment` arm at all; comments fall into the catch-all branch,
which only walks children.

Verified 2026-08-02: `<p>A</p><!-- note --><p>B</p>` → `"A\n\nB\n"` under both
Balanced and Preserve.

### 5.4 `elements.md` — always-removed list

The list omits `<svg>` and `<head>`. Both are removed unconditionally —
`src/utils.rs::is_skip_tag` includes them.

Verified: `<p>X</p><svg><circle/></svg><p>Y</p>` → `"X\n\nY\n"`;
`<html><head><title>T</title></head><body><p>Z</p></body></html>` → `"Z\n"`.

### 5.5 `modes.md` — Preserve comments claim

Same false claim as §5.3, in the Preserve section: "Retains HTML comments in the
pre-processed output." Correct it consistently with `elements.md`.

### 5.6 `modes.md` — Minimal shell wording

"Optionally removes shell elements … when `drop_interactive_shell` is `true`" is
misleading. `for_mode(Minimal)` sets it `true` already.

### 5.7 `options.md` — `drop_interactive_shell` prose

The file contradicts itself. The table is **right**; the prose is **wrong**.

- Table: Minimal ✅, all others ❌ — matches `src/options.rs`.
- Prose under `### drop_interactive_shell`: "Disabled by default in all modes;
  opt in explicitly." — false for Minimal.

Verified: `balanced=false strict=false minimal=true semantic=false preserve=false`.

Fix the prose. Leave the table. **Change nothing else in this file.**

### 5.8 `philosophy.md` — separate-pass assertions

Two claims assert a pass that does not exist:

- "…for every tree traversal — both in the pre-processing pipeline and in the
  Markdown conversion step." There is **one** traversal, not two.
- "They are applied in a pre-processing pass…" There is no separate pass;
  preprocessing is inline in the single traversal.

Correct both to describe inline preprocessing within one traversal. Do not
rewrite the surrounding argument — the *point* being made (non-recursion, mode
presets) is correct; only the mechanism is misdescribed.

## 6. Required verification

For **every** changed claim, execute the documented case against the library and
record input and actual output. Do not assert from this handoff — re-run it.

```
cargo run --example <scratch>     # or a scratch example, deleted afterwards
cd docs && mdbook build           # must succeed
```

Then confirm no stale references remain:

```
grep -rn "preprocessor" docs/src/          # expect: none
grep -rniE "re-parse|reparse|pre-processing pass|pre-processed DOM" docs/src/
```

The second grep will still return `options.md:73` — that is **expected and
correct**, per §4. Anything else it returns is either something you missed or a
new finding: report it, do not silently fix it.

> **CORRECTION 2026-08-02, at review.** This originally predicted the grep would
> return `options.md:73` **and** `options.md:97`. That was wrong. `:97` reads
> "…during pre-processing", which matches none of the four literal patterns
> (`re-parse`, `reparse`, `pre-processing pass`, `pre-processed DOM`). Only `:73`
> matches. `:97` is still correctly left untouched — it is RFC 006 territory
> either way — but it was never going to appear in this grep's output.
>
> Raised by the implementer, confirmed at review by re-running the grep.
> A verification step that predicts the wrong result is worse than one that
> predicts nothing, because a correct outcome then looks like a discrepancy.

## 7. If you find further drift

The RFC 003 sweep found four claims beyond the original three
(`modes.md:96`, `philosophy.md:38`, `philosophy.md:50`, plus the `elements.md`
always-removed omission). More may exist.

**Report additional findings; do not fix them under this RFC.** A doc-correction
RFC that expands to cover whatever the implementer notices becomes unreviewable,
and the reviewer can no longer tell specified work from discretionary work. List
them in your review request and they will be scoped properly.

The exception is the same one established under RFC 004: a dangling reference
created by *your own* edit in this RFC is part of that edit.

## 8. Compatibility and security

No API, behaviour, output, or artifact change. Documentation only.

One incidental security-adjacent improvement: readers currently told that
`Preserve` retains comments might assume comment content survives conversion
into their output. It does not. Correcting §5.3 and §5.5 removes a false
expectation about what reaches a downstream consumer.

## 9. Prohibited shortcuts

- Do not "fix" attribute documentation by describing today's behaviour. RFC 005
  changes it; RFC 006 documents it. Touching it now creates a third version of
  the same text.
- Do not delete the mode or option pages to resolve inconsistency.
- Do not hand-edit `docs/book/`.
- Do not expand scope to drift you discover — report it (§7).
- Do not touch any `.rs` file.

## 10. Known risks

| Risk | If it happens |
|---|---|
| Further undocumented drift exists | Expected. Report, do not fix. |
| A correction seems to require an attribute-semantics statement | Stop and raise it — that is the RFC 003 / RFC 006 boundary, and it is exactly where scope creep starts |
| `mdbook build` fails | Report; do not work around by removing content |

## 11. Required evidence

1. `mdbook build` — succeeds, no broken links.
2. For each changed claim: the input HTML and the actual observed output.
3. The two greps from §6, with `options.md:73`/`:97` shown as expected residue.
4. `git diff --stat` — five files, `docs/src/` only.
5. `cargo test --workspace` — 74 passed, 0 failed (unchanged; nothing compiled changed).

## 12. Acceptance checklist

- [ ] `architecture.md` describes three steps, no pre-process, no re-parse
- [ ] `architecture.md` states explicitly that no intermediate serialisation occurs
- [ ] Workspace layout tree matches the repository after RFC 004
- [ ] `elements.md` states comments are removed in all modes
- [ ] `elements.md` lists `<svg>` and `<head>` among always-removed
- [ ] `modes.md` Preserve section no longer claims comments are retained
- [ ] `modes.md` Minimal shell wording corrected
- [ ] `options.md` prose agrees with its own table and `src/options.rs`
- [ ] `philosophy.md` no longer asserts a separate pass or second traversal
- [ ] No `preserve_*` or `drop_presentation_attrs` description modified
- [ ] `options.md:73` / `:97` left untouched
- [ ] `docs/book/` untouched; no `.rs` file touched
- [ ] `mdbook build` succeeds
- [ ] No file outside §3 modified

## 13. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 003 acceptance criteria, by number)
3. Changed files — complete list
4. **Per-claim verification: input, observed output, corrected text**
5. **Additional drift found but deliberately not fixed** (§7)
6. Differences from RFC 003, if any, and why
7. Executed verification and results
8. Evidence per §11
9. Unresolved issues
10. Known limitations
11. Requested review focus

Items 4 and 5 are the substance. The edits are easy; demonstrating that each
corrected statement is now *true*, and being disciplined about what you left
alone, is the deliverable.

## 14. Evidence standard

Standing standard: if a captured transcript or count does not reconcile, say so
explicitly, even when it does not change the conclusion you were asked to reach.

Applies here to enumeration in particular — if you state "N claims corrected,"
make sure the list has N entries, and that no item appears in two categories.

## 15. Escalate rather than decide

Stop and raise it if you find: a correction that cannot be made without stating
something about attribute semantics; a documented behaviour that contradicts the
engine in a way not listed in §5 and not obviously in RFC 006's territory; or
`mdbook build` failing for a reason predating this change.

## 16. After this lands

RFC 003 is the last RFC in M1. Once approved, `2.1.7` becomes cuttable. Two
things belong to that release commit, not to this RFC:

- Sweep `CHANGELOG.md`'s `Unreleased` section against the full commit range —
  it currently omits RFC 004 and will omit RFC 003.
- Move RFCs 001, 002, 003, 004, 014 to `rfcs/done/` with
  `Implemented (2.1.7)`, updating `rfcs/README.md` in the same commit.
