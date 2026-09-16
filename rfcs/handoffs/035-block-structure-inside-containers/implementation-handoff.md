# Developer Handoff — RFC 035 · Block structure inside containers

**Governing RFC.** [RFC 035](../../accepted/035-block-structure-inside-containers.md) — §3 design, §3.1 loose/tight rule, §4 cells
**Milestone.** M3 · Output validity → `2.3.0`
**Priority.** P0
**Prepared.** 2026-09-17

---

## 0. 🛑 QUEUED — do not start until RFC 028 is approved

| Precondition | Why |
|---|---|
| `024b` approved | fence and code-context changes in the same files |
| **RFC 028 approved** | it changes the renderer's inline-around-block handling and chooses the look-ahead mechanism (tree query or buffering) that §3.1's tight/loose decision should **reuse**, not duplicate |

**Start when** `.git-exclude/reviewed/028-inline-around-blocks/README.md` exists with an approved verdict.
**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Purpose

Block content inside list items and blockquotes must stay inside them. Audit A-06, A-07, A-08 — validity
defects on common input, including the loose-list shape every CMS produces.

## 2. Current output — re-derived 2026-09-17 on a release build of `1de7f2c`

RFC 024 and 028 change the renderer before you start; **re-derive these** and state what moved.

| Cell | Output |
|---|---|
| `p_in_li` | `- \n\npara` |
| `p_in_each_li` | `- \n\na\n\n- \n\nb` |
| `two_p_in_li` | `- \n\na\n\nb` |
| `pre_in_li` | ``- \n\n```\nx\n``` `` |
| `text_then_pre_in_li` | ``- see\n\n```\nx\n``` `` |
| `blockquote_in_li` | `- \n\n> q` |
| `ol_in_ol` | `1. one\n  1. inner` |
| `ol_in_ol_two_digit` | `10. ten\n  1. inner` |
| `ul_in_ol` | `1. one\n  - inner` — **also broken**: 2 spaces is below `1. `'s content column |
| `two_p_in_blockquote` | `> one\n\n> two` |
| `multiline_pre_in_blockquote` | ``> ```\nl1\nl2\n``` `` |
| `ul_in_blockquote` | `> - a\n> - b` — probably already correct |
| `two_p_in_blockquote_in_li` | `- \n\n> a\n\n> b` |
| `two_p_in_nested_blockquote` | `> > a\n\n> > b` |
| `multiline_pre_in_li_in_blockquote` | ``> - \n\n> ```\nx\ny\n``` `` |
| `tight_list_control` | `- a\n- b` — correct |

**Expect 14 of 16 to fail today.** Confirm with the harness, not by reading this table.

## 3. Order of work

1. **Harness first.** Add a `Rfc035` owner. Add the 16 cells from RFC 035 §4 — in a new
   `tests/output_validity/block_in_container.rs` — **with the expectations exactly as written in the RFC**.
   Run against the base commit; mark every failing cell `known_defect(Rfc035, <harness reason>)`. Commit.
   If an expectation looks wrong to you, **stop and report** — do not adjust it to match output.
2. **Implement** RFC 035 §3: a container prefix stack in the sink.
3. **Remove each marker** as its cell passes.

## 4. Design constraints

- **The prefix stack lives in the sink**, beside RFC 024's per-destination bookkeeping. Push on container
  enter, pop on leave; in debug builds assert it is empty at document end.
- **List item continuation prefix = the item's content column**: marker width plus one space
  (`- ` → 2, `1. ` → 3, `10. ` → 4). Nested lists indent to the **parent's** content column.
- **Every line inside a container gets the full stack prefix, outermost first** — content, blank separator
  lines (a quote's blank line is `>` alone), and **code-block content lines**. This deliberately reverses RFC
  024's "no prefix in `code_block_content`", which deferred exactly this to A-08.
- **Loose/tight per RFC 035 §3.1**: loose iff some item has two or more blocks, a maximal inline run counting
  as one block, nested lists not counted. Decide it **with the mechanism RFC 028 chose**; do not add a second
  look-ahead.
- **Tight plain lists stay byte-identical to 2.2.3** — `tight_list_control` plus every existing list test.

## 5. Scope boundary, per RFC 027 Rule 2

- Escaping at block start — RFC 010, which follows. If a structure fix exposes an escaping failure in an RFC 010
  cell, **report it; do not fix it here**.
- Inline elements around blocks — RFC 028, done by then. Its cells must not change state.
- RFC 024's five blockquote-prefix cells must stay green.
- Tables, definition lists — `2.4.0`.

## 6. Required verification

Per RFC 027 Rule 3, label what each ran against.

1. §2 table re-derived before; after, for every cell.
2. All 16 cells passing under CommonMark and GFM, all five modes; list which were marked and which passed from the start.
3. **No other harness cell changes state.**
4. **Tight-list byte identity**: every existing test and runner fixture with a list — outputs before and after; any change listed and justified by §3.1.
5. Prefix-stack balance assertion present, and shown to fire on a deliberately unbalanced push (in a test, not left in).
6. `cargo test --workspace --all-features --locked --no-fail-fast` — count reconciled against the post-RFC 028 baseline; fmt; clippy per CI.
7. `docs/src/api/elements.md` states §3.1's rule; docs example gate green.
8. CHANGELOG with before/after.

## 7. Prohibited shortcuts

- Do not edit an RFC 035 §4 expectation.
- Do not special-case `<p>` — key on block classification (the shared list RFC 028 uses).
- Do not add a second look-ahead mechanism.
- Do not prefix lines from individual element arms — the sink does it.

## 8. Escalate rather than decide

Stop and raise if: §3.1's rule produces output a reader would clearly not want on a real shape; the prefix
stack cannot be kept balanced across captures (links, code spans); tight plain lists would have to move; or a
fix changes another RFC's cells.

## 9. Acceptance checklist

- [ ] §0 honoured
- [ ] §2 re-derived; what moved stated
- [ ] 16 cells added first, exactly as written; failing ones marked at the base commit
- [ ] Container prefix stack in the sink; every line prefixed, incl. blank and code lines
- [ ] §3.1 tight/loose rule via RFC 028's mechanism
- [ ] All 16 cells passing, both readings, five modes
- [ ] No other cell changed state; tight lists byte-identical
- [ ] Balance assertion shown to fire
- [ ] elements.md rule; docs gate green; CHANGELOG

## 10. Report back

`.git-exclude/review-request/035-block-structure-inside-containers/README.md`, evidence under `evidence/`, exit
codes inside the files.
