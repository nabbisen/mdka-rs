# Developer Handoff — RFC 024 · Inline composition: the output sink

**Governing RFC.** [RFC 024](../../done/024-inline-composition-output-sink.md) — including its 2026-09-16 amendment
**Milestone.** M3 → `2.3.0`
**Priority.** P0
**Prepared.** 2026-08-31. **Revised 2026-09-16, before dispatch** — §0.

---

## 0. Preconditions

1. **RFC 025 approved** — met (`7338b17`, review 2026-09-16).
2. **Slice `025c` approved** — ✅ met 2026-09-16
   (`.git-exclude/reviewed/025c-harness-addendum/README.md`). The nine re-labelled
   cells are in place at `d5d64cd`. **This handoff is an instruction to start.**

**Once handed over, this file is frozen.** Later changes arrive as dated addenda.

**What the revision changed:** preconditions; criteria 8, 9, 10 added (8 was in the RFC
but never carried here); markers are strict expected failures, not `#[ignore]`; test
baseline and commands current; the RFC 025 inventory is now the specification of done.

## 1. Purpose

The renderer redirects output into a buffer while capturing link text. That redirection
is honoured by one writer and bypassed by the rest, so inline elements escape links,
emit markup inside code, and cancel blockquote prefixes.

## 2. The defects — re-derived 2026-09-16 on the current tree

```
<a href="/page"><img src="i.png" alt="pic"></a>   →  ![pic](i.png)[](/page)
<a href="/x"><strong>b</strong></a>               →  ****[b](/x)
<a href="/x">Read <strong>more</strong> now</a>   →  ****[Readmore now](/x)     space lost
<pre>plain</pre><p>After</p>                      →  plain\n```\n\nAfter          fence unbalanced
<blockquote><strong>b</strong> rest</blockquote>  →  **b** rest                    quote lost
<code><strong>b</strong></code>                   →  `**b**`                        markup in code
```

**Re-derive them when you start.** The RFC 025 harness is the authoritative list: every
cell owned by RFC 024 — **19** after `025c` — is this RFC's definition of done.

## 3. Root cause — measured

`src/renderer.rs` has **37 direct `self.output.push*` calls** (re-counted 2026-09-16).
Exactly one place — inside `push_raw`, around line 126 — checks the capture state. And
the inline arms that push directly (`strong`/`b`, `em`/`i`, `code`, `img`, around lines
291–335) also set `at_line_start = false` **without calling `emit_pending_prefix()`**,
which is why a blockquote starting with an inline element loses its `> `.

## 4. Design

Per RFC 024 §"Make the sink the only way to write":

- One accessor returns the current destination — the capture buffer while capturing,
  otherwise `self.output`.
- **Every** writer goes through it. **Prefer a mechanism over a convention** — a wrapper
  type or narrower module boundary so a new handler cannot write to `self.output` by
  accident.
- **Bookkeeping travels with the sink:** `newlines_emitted`, `at_line_start`,
  `last_was_space`, **and the pending blockquote prefix**. The sink, not each arm,
  emits the pending prefix before the first content byte. Where line state is
  meaningless inside a capture, say so explicitly.
- **Code context (criteria 3, 9):** while inside `<pre>` or an inline `<code>`, child
  elements contribute **text only** — no `**`, `*`, `![`, `[`, `](`.
- **Empty links (criterion 7):** a link with no text and no image emits nothing.

## 5. Scope boundary

Per RFC 027 Rule 2: **where bytes go**, and the bookkeeping that travels with them.

- **Escaping — which bytes — is RFC 010.** Inside a code span, text should not be
  backslash-escaped; that remains RFC 010's cells (`underscore_in_code_span` etc.). If
  making text-only code spans also changes escaping output, **stop and report** rather
  than fixing RFC 010's cells here.
- **Blocks inside inline elements** (`<a><h2>`, `<code><p>`, `<strong><p>`) are **RFC 028**,
  including its pending extension. Not here.
- Element coverage — `<li>` block children, blockquote *continuity* across blocks, list
  indent (A-06, A-07, A-08) — is RFC 009. Criterion 10 is **not** continuity; it is the
  prefix cancelled by an inline writer.

## 6. Required verification

Per RFC 027 Rule 3, label what each ran against.

1. §2's cases, before and after, on a release build.
2. **Every RFC 024-owned `known_defect` marker removed and the cell passing — under both the
   CommonMark and GFM readings, in all five modes** — list them.
   A marker you cannot remove is a finding, not a skip.
3. **No other cell changes state.** Run the harness before and after; any cell outside
   RFC 024's that flips (either direction) is reported with its owner.
4. `<pre><code>…</code></pre>` output byte-identical to 2.2.3 across every existing test
   and the harness's runner fixtures.
5. Count of remaining direct `self.output` writes in element handling — zero, or each
   with a written reason.
6. Nested-link and empty-link behaviour tested.
7. `cargo test --workspace --all-features --locked --no-fail-fast` green; count reconciled
   against **273** (the post-`025c` baseline at `d5d64cd`).
8. fmt; clippy `--workspace --all-targets --all-features --keep-going -- -D warnings`.

## 7. Prohibited shortcuts

- Do not fix escaping here.
- Do not change `<pre><code>` output.
- Do not leave a direct `self.output` write in element handling without a written reason.
- **Do not edit any RFC 025 expectation.** Remove a marker only for a cell you actually
  made pass.
- Do not emit the blockquote prefix from each inline arm — that is the convention that
  failed. It belongs to the sink.

## 8. Known risks

| Risk | If it happens |
|---|---|
| Routing 37 sites misses one | The harness is the detector — a missed site keeps a marker alive. |
| Borrow-checker friction from the accessor | Expected. If a handler cannot be expressed, raise it. |
| Line-state desync inside a capture | RFC 016/017 are the precedent. Cover blank-line and fence-adjacent cases. |
| Output changes more widely than expected | §6.3 reports it; do not assume it is fine. |
| Text-only code spans interact with escaping | §5 — stop and report. |

## 9. Acceptance checklist

- [ ] §0 — started only after `025c` approval
- [ ] Linked image parses as a link containing an image (1)
- [ ] `<a><strong>` parses as a link with strong text; no stray `*` (2)
- [ ] Bare `<pre>`: balanced fence; text only inside; `After` outside (3)
- [ ] `<pre><code>` byte-identical to 2.2.3 (4)
- [ ] Zero direct `self.output` writes in element handling, or each justified; count reported (5)
- [ ] All RFC 024-owned `known_defect` markers removed, cells passing, listed (6)
- [ ] Nested-link and empty-link behaviour defined and tested (7)
- [ ] `Read <strong>more</strong> now` inside a link keeps both spaces (8)
- [ ] Inline `<code>` holds text only (9)
- [ ] Blockquote beginning with an inline element keeps `>`; the sink emits the prefix (10)
- [ ] No non-024 cell changed state, or each reported
- [ ] CHANGELOG entry with before/after output
- [ ] Count reconciles; fmt and clippy clean

## 10. Escalate rather than decide

Stop and raise if: a handler cannot be expressed through the sink; a fix here would also
change escaping; `<pre><code>` output moves at all; a cell outside RFC 024 changes state;
or the correct structure for a composition case is ambiguous.

## 11. Report back

`.git-exclude/review-request/024-inline-composition-output-sink/README.md`, evidence under
`evidence/`, exit codes inside the files.
