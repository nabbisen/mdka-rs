# Developer Handoff — RFC 010 · Escaping and text round-trip

**Governing RFC.** [RFC 010](../../done/010-escaping-and-text-round-trip.md) — **§3 is the specification**
**Milestone.** M3 · Output validity → `2.3.0`
**Priority.** P0
**Prepared.** 2026-09-16

---

## 0. Preconditions — met (2026-09-17)

| Precondition | Why |
|---|---|
| RFC 024 approved | escaping writes through the sink; block-start context (§3.6) comes from the sink's prefix state |
| RFC 028 approved | it changes the same emphasis and link arms; working both at once means one rebases the other |
| **RFC 035 approved** | accepted by the owner 2026-09-17 — block-start escaping (§3.6) needs correct list-item and quote structure to know where a block starts; RFC 035 fixes that structure |
| `025c` approved | ✅ met 2026-09-16 — GFM parsing and 4 GFM cells owned by this RFC |

| RFC 035 approved | ✅ met 2026-09-17 (`a90307d`) |
| Slice `024d` approved | ✅ met 2026-09-17 (`e317a72`) |
| Slice `024e` approved | ✅ met 2026-09-17 (`e6d39ff`) — RFC 024 complete |

**All preconditions are met. This handoff is an instruction to start.** §2 was re-derived on a release build of `e6d39ff`
before handover: every example reproduces exactly. Baseline test count: **370**. **Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. Purpose

Make every piece of text mdka emits parse back to exactly the source text, under CommonMark
and GFM, in every context. Several of today's failures **destroy content**, not just add
backslashes.

## 2. Re-derive before you start

Everything below was true at `7338b17`. RFC 024 and 028 will have changed the renderer by the
time you read this. **Re-derive each fact** and state what moved.

- **All prose escaping is `write_normalised` in `src/utils.rs`**, one per-character table:
  backslash, `*`, `_`, backtick, `[`, `]`, `!` always; `#`, `>`, `+`, `-` and **digits** only
  when `at_line_start`. Consequences:
  the digit is escaped instead of the delimiter (`\1986.`); after a `- ` or `> ` prefix
  `at_line_start` is false so nothing is escaped (`1986.` becomes a list); the same table runs
  inside code (`snake\_case` in a code span).
- Destinations and titles are written **raw** by the `<a>`/`<img>` arms.
- **Block-start context after RFC 035** — the sink now holds a container stack (`containers: Vec<Container>`, list items and quotes),
  flushes pending newlines with the prefix of exactly the containers still open, and tracks when a line holds **only markers**
  (`only_markers`) after `- `, `1. ` or `> `. That state is precisely §3.6's "block start after any prefix". Use it; do not rebuild it.
  Verify these names when you start.
- Nothing handles `<`, `&`, `~~~`, or code-span delimiter length.
- Examples, current output:

```
<p>1986. A great year</p>                →  \1986. A great year
<ul><li>1986. A great year</li></ul>     →  - 1986. A great year      (nested list, number lost)
<p>~~~</p><p>after</p><h2>later</h2>     →  ~~~\n\nafter\n\n## later   (fence swallows the rest)
<p>&lt;div&gt; hidden</p>                →  <div> hidden              (HTML block, text vanishes)
<p>&amp;copy; &amp;#42;</p>              →  &copy; &#42;              (renders as © *)
<a href="/a b.html">x</a>               →  [x](/a b.html)            (not a link)
<p><code>snake_case</code></p>           →  `snake\_case`
```

## 3. The work — RFC 010 §3, context by context

Implement **escaping chosen by the context being written**, replacing the single table. RFC 010
§3.1–3.8 gives the rule for each context; follow it. Summary:

| Context | Rule |
|---|---|
| Code span (§3.1) | no escapes; delimiter one backtick longer than the longest run inside; pad with a space if content starts/ends with a backtick |
| Fenced block (§3.2) | verbatim; fence longer than any backtick run at a line start inside; drop a language hint containing a backtick |
| Destination (§3.3) | `<…>` form when it has a space, control char or unbalanced parens; otherwise bare with `\` and unbalanced parens escaped; **balanced parens stay unescaped** |
| Title (§3.4) | `"`-delimited with `"` and `\` escaped (or another delimiter that avoids escapes — state which) |
| Link text / alt (§3.5) | brackets escaped only when they would unbalance |
| Block start (§3.6) | escape what would open a construct: ordered-list **delimiter** (`1986\.`, `1\)`), bullet, ATX heading, `>`, fence openers incl. `~~~`, thematic break, HTML block start, setext underline — **after** any list/quote prefix |
| Inline (§3.7) | `*`/`_` only where they could form a delimiter run (CommonMark §6.2 flanking; intraword `_` unescaped); `!` only before `[`; `<` before tag/autolink shapes; `&` before entity shapes; GFM `~~` **and single `~`** |
| GFM table delimiter row (§3.6) | a row of `-`/`:` cells separated by `\|`, under a line containing `\|` — **with or without outer pipes, with or without `:`**. Today only a line-leading `-` happens to be escaped, so `\| --- \|` and `:-- \| --:` become tables (`025c`) |
| Adjacent emphasis (§3.8) | `**a***b*` must not form an ambiguous run — use `_` for one, or separate |

**Minimal escaping is a requirement** (A-10). The harness guards the other side: an escape
removed wrongly turns a cell red.

## 4. Harness

1. **Owned cells:** 28 at `d5d64cd` — the 24 original plus 4 GFM cells from `025c`
   (`strikethrough_like_text`, `single_tilde_strikethrough_like_text`,
   `table_like_lines_with_pipes`, `table_like_lines_with_alignment`). **Take the list from the harness at the time you start**; state the count.
2. **Add cells before changing behaviour they would cover:** adjacent emphasis (§3.8 — strong/em,
   em/strong, strong/strong); `snake_case_here` with no escapes (A-10); `!` not before `[`; a
   setext-underline line; a thematic-break line of text; `1)` in a blockquote. Expectations from
   the HTML's meaning. Mark each `known_defect(Rfc010, …)` if it fails today, so its fix is
   observed.
3. **Re-label** `Rfc010Planned` → `Rfc010` (the RFC now exists). No expectation edits.

## 5. Documentation — D-05

After the implementation passes, rewrite `docs/src/api/text-processing.md`'s escaping section
to describe **contexts**, matching §3. Every example on the page must pass the docs example gate
and agree with the harness.

## 6. Scope boundary, per RFC 027 Rule 2

- Where bytes go — RFC 024. Inline elements around blocks — RFC 028. Both done by then; **if a
  change here would alter either's cells, stop and report**.
- NBSP collapse (A-14) — not here.
- Emitting tables or strikethrough — RFC 008/009, M4.

## 7. Required verification

Per RFC 027 Rule 3, label what each ran against.

1. §2 examples before and after, release build.
2. **Every RFC 010 cell** — markers removed, passing under CommonMark **and** GFM, all five modes —
   listed.
3. **No other harness cell changes state**, or each reported with its owner.
4. Code spans and fenced blocks contain **no** backslash escapes — a property or a test over the
   runner fixtures, not a spot check.
5. Source noise: count backslashes emitted over the runner fixtures and the existing test corpus,
   before and after. Expect fewer; explain any increase.
6. Benchmarks (`cargo bench`) before and after on the same machine; within noise, or the
   regression stated.
7. Docs example gate green on the rewritten `text-processing.md`.
8. `cargo test --workspace --all-features --locked --no-fail-fast`; fmt; clippy per CI.

## 8. Prohibited shortcuts

- Do not edit a harness expectation.
- Do not add a blanket escape to make a cell pass — minimal escaping is a requirement.
- Do not special-case a cell's literal input.
- Do not keep a second, global character table alongside the contexts.

## 9. Escalate rather than decide

Stop and raise if: the sink does not expose block-start context after prefixes; a CommonMark
and a GFM reading require contradictory escapes; a fix would change another RFC's cells; or
benchmarks regress beyond noise.

## 10. Acceptance checklist

- [ ] §0 honoured
- [ ] §2 facts re-derived; what moved stated
- [ ] §3 contexts implemented; single table removed
- [ ] §4 cells added first, re-labelled, all RFC 010 cells passing under both readings
- [ ] §7.3 no other cell changed state
- [ ] §7.4 no escapes inside code; §7.5 backslash count before/after
- [ ] §7.6 benchmarks
- [ ] §5 `text-processing.md` rewritten; docs gate green
- [ ] RFC 010 criteria 1–8 addressed
- [ ] CHANGELOG with before/after examples

## 11. Report back

`.git-exclude/review-request/010-escaping-and-text-round-trip/README.md`, evidence under
`evidence/`, exit codes inside the files.
