# Developer Handoff — RFC 028 · Inline elements around block content, and emphasis negated by its own style

**Governing RFC.** [RFC 028](../../accepted/028-emphasis-around-block-content.md) — **its two accepted amendments and the consolidated acceptance criteria at its end govern**
**Milestone.** M3 · Output validity → `2.3.0`
**Priority.** P0
**Prepared.** 2026-09-16. **Rewritten 2026-09-16, before dispatch**, for the owner's scope decisions.

---

## 0. Preconditions — met (2026-09-17)

| Precondition | Status |
|---|---|
| RFC 025 harness landed | ✅ `7338b17` |
| Slice `025c` approved | ✅ met 2026-09-16 |
| RFC 024 approved | ✅ met 2026-09-17 (`1de7f2c`) — the sink exists; `src/renderer.rs` line numbers have all moved |
| Slice `024b` approved | ✅ met 2026-09-17 |
| Slice `024c` approved | ✅ met 2026-09-17 — RFC 024 complete |

**All preconditions are met. This handoff is an instruction to start.** **Once handed over, this file is frozen**; changes arrive as dated
addenda.

## 1. Purpose

An inline element whose children include blocks emits its delimiters or link syntax around
those blocks, producing stray `**` lines, empty list items, emptied headings or lone
backticks. And Google Docs wraps every copy in a `<b>` whose own style says *not bold*, so
a single-paragraph paste comes out entirely bold.

## 2. Reproduce it first — re-derived 2026-09-16; re-derive again when you start

**Emphasis around blocks:**

| Input | Output today |
|---|---|
| `<strong><p>x</p><p>y</p></strong>` | `**\n\nx\n\ny\n\n**` |
| `<em><p>x</p><p>y</p></em>` | `*\n\nx\n\ny\n\n*` — lone `*` lines parse as **empty list items** |
| `<b>text<p>para</p></b>` | `**text\n\npara\n\n**` |
| `<b><em><p>x</p></em></b>` | `***\n\nx\n\n***` |
| `<b><div>x</div></b>`, `<b><ul><li>a</li></ul></b>`, `<b><h2>head</h2></b>` | stray `**` around each |

**Google Docs** (public captures: ProseMirror #459, MarkText #4688):

| Input | Output today |
|---|---|
| `<b style="font-weight:normal"><p>one</p><p>two</p></b>` | `**\n\none\n\ntwo\n\n**` |
| `<b style="font-weight:normal;" id="docs-internal-guid-x"><span style="font-weight:400">Hello world</span></b>` | `**Hello world**` — **entirely bold** |

**`<a>` and `<code>` around blocks — outputs changed by RFC 024's sink** (re-derived at `1de7f2c`). The
sink now routes a block's line breaks **into** the link or code capture:

| Input | 2.2.3 | After RFC 024 |
|---|---|---|
| `<a href="/x"><h2>Title</h2></a>` | `## \n\n[Title](/x)` | `[## Title\n\n](/x)` |
| `<a href="/out"><p>x</p><p>y</p></a>` | `[xy](/out)` | `[x\n\ny\n\n](/out)` — **no longer parses as a link** |
| `<code><p>x</p><p>y</p></code>` | `` `\n\nx\n\ny\n\n` `` | `` `x\n\ny\n\n` `` |

All still known defects, correctly marked. **Do not adopt the "collapse block breaks to a space inside a
capture" rule** RFC 024's implementer measured (`[x y](/out)`, `` `x y` ``): it contradicts this RFC's
accepted designs — link each block's content, and code around blocks keeps the blocks. The interim
regression in the second row is accepted only because **`2.3.0` cannot be cut with RFC 024 and without this
RFC**.

**Key on block classification, not on `p`.** A fix written against `<p>` alone passes the
first case and leaves `div`, `ul` and headings broken.

## 3. The required behaviour

The RFC's consolidated criteria are the specification. In short:

| Element | Children include blocks | Own style negates it | Otherwise |
|---|---|---|---|
| `<strong>`/`<b>` | no delimiters, blocks kept | no delimiters | `**…**` unchanged |
| `<em>`/`<i>` | no delimiters, blocks kept | no delimiters | `*…*` unchanged |
| `<code>` | no delimiters, blocks kept | — | unchanged |
| `<a href>` | **link each block's content**: `## [Title](/x)` | — | unchanged |

### 3.1 Why not distribute emphasis (option B) — unchanged, and now stronger

Distributing `**` over each block would make every multi-paragraph Google Docs paste
entirely bold. **Style negation (§3.2) is not a reason to revisit that**: genuine `<strong>`
around blocks is invalid HTML, and output that looks intentionally bold is never reported.
If you conclude otherwise, **stop and report**.

### 3.2 Style negation — exactly this, nothing more

- `<b>`/`<strong>`: no delimiters if **its own** `style` declares `font-weight` as `normal`
  or a number **≤ 500**. `lighter`/`bolder` → unchanged.
- `<i>`/`<em>`: no delimiters if its own `style` declares `font-style: normal`.
- Parse: split on `;`; name before `:` trimmed and case-folded; value trimmed, case-folded,
  `!important` stripped; **last declaration of the property wins**; no other property read.
  No inheritance, no stylesheets.
- **Never adds emphasis.** `<span style="font-weight:700">` stays plain — a separate,
  unscheduled candidate.
- This is the first `style` reading in mdka. Keep it a small, separately tested function.

### 3.3 `<a>` around blocks — linking each block

- Paragraph, heading, list item, blockquote content: wrap **that block's inline content** in
  `[…](dest "title")`.
- `<pre>` inside `<a>`: a code block holds text only (RFC 024 criterion 3) — **not linked**.
  Test that explicitly.
- A block with no text and no image: no link (RFC 024's empty-link rule).
- An inner link inside the block content (html5ever usually splits nested anchors): define
  and test; do not produce `[[x](/in)](/out)`.

## 4. Mechanism — choose after RFC 024

**Route 1 — tree query.** `traversal.rs` computes a "has block descendant" flag, bottom-up,
and the renderer consumes it. `enter_element` takes `&scraper::node::Element`, which carries
no children, so the query cannot live in the renderer.

**Route 2 — buffer and decide on leave.** Capture the element's content as the link path
does, then emit with or without delimiters based on what was captured. RFC 024 generalises
that machinery; after it lands this route may be nearly free. `<a>`-around-blocks needs to
place link syntax **inside** each block, which likely favours knowing the answer on enter —
weigh that.

**State which route you chose and why.**

### Requirements that bind either route

- **O(n) over the document**, not per element. One `<b>` wrapping an entire Google Docs payload
  is exactly the quadratic case.
- **Descendant, not child.** `<b><em><p>x</p></em></b>` has only an inline direct child.
- **One block-tag list, shared** with the arms that emit blocks — in `utils.rs` beside
  `is_structural_tag` and friends. Two lists that must agree is the `figure`/`figcaption`
  defect from RFC 003.

## 5. First commit — re-label the harness

RFC 025 marks these cells UNOWNED; the owner has assigned them to this RFC. **Before any
`src/` change**, change their `defect(...)` owner to RFC 028 — **no expectation edits**:

- the 10 `<a>`/`<code>` × block cells in `block_in_inline.rs` — `p_in_a`, `ul_in_a`,
  `blockquote_in_a`, `pre_in_a`, `heading_in_a`, `p_in_code`, `ul_in_code`,
  `blockquote_in_code`, `pre_in_code`, `heading_in_code` (names as of `7338b17`; the
  harness is authoritative if they have moved);
- `field_reports::google_docs_bold_wrapper_inline` — the single-paragraph Google Docs cell — and **replace the "pending the owner's decision" wording in its reason**: the owner accepted RFC 028's style amendment on 2026-09-16.

After re-labelling, RFC 028 owns **22** cells. Confirm the count.

**Also in this first commit — the harness's model of negated emphasis.** The harness's
`[link-content]` property counts every `<b>`/`<strong>` as `strong` and every `<i>`/`<em>` as
`em` (`tests/output_validity/harness/properties.rs`, around line 168). After this RFC, a
`<b style="font-weight:normal">` inside a link correctly emits no emphasis, and the property
would flag it. Update the HTML-side model so an element whose **own** style negates its
emphasis — by §3.2's rule — is not counted.

- **Implement it in the harness from §3.2's written rule. Do not call mdka's parser for it.**
  A harness that reuses the code it checks cannot catch that code being wrong.
- Prove it both ways: a link holding `<b style="font-weight:normal">x</b>` expects no `strong`;
  a link holding `<b>x</b>` still expects `strong`.
- `[link-content]` is the only property that maps these elements to an inline kind (checked
  at `d5d64cd`). If that is no longer true, cover the others the same way.
- **No expectation edits.** This is a model change, reviewed as part of this RFC.

## 6. Scope boundary, per RFC 027 Rule 2

- **Where bytes go** inside links and code — RFC 024 (done by then).
- **Escaping** — RFC 010.
- The `docs-internal-guid` id anchor Balanced mode emits — `preserve_ids`; recorded as evidence
  for a future option, **not here**.
- Recovering real emphasis from `<span style>` — not here.

## 7. Required verification

Per RFC 027 Rule 3, label what each ran against.

1. §2 cases, before and after, release build.
2. **All 22 RFC 028 cells** have markers removed and pass, under CommonMark and GFM, in all
   five modes — listed.
3. **No other harness cell changes state**, or each change reported with its owner.
4. Style parser edge cases: `!important`, upper case, whitespace, repeated declaration (last
   wins), other properties present, `font-weight: 500` vs `600`, `lighter`/`bolder`.
5. **Inline-only emphasis and links byte-identical to 2.2.3** across every existing test and
   the runner fixtures — except elements whose own style negates emphasis. Show the diff scope.
6. O(n): a benchmark or a test on deeply nested emphasis around a large payload, before and after.
7. `cargo test --workspace --all-features --locked --no-fail-fast`; fmt; clippy per CI.

## 8. Prohibited shortcuts

- Do not distribute emphasis over block children without reporting first.
- Do not read any `style` property other than the two in §3.2, or add emphasis from style.
- Do not write a second block-tag list.
- Do not edit a harness expectation.

## 9. Acceptance checklist

- [ ] §0 honoured — started after RFC 024's approval
- [ ] §5 re-label first; 22 cells owned
- [ ] Consolidated criteria 1–13 of RFC 028, each addressed in the review request
- [ ] Route chosen and justified
- [ ] §7.3 no other cell changed state
- [ ] §7.5 inline-only byte-identity, with the one stated exception
- [ ] CHANGELOG with before/after, including style negation

## 10. Report back

`.git-exclude/review-request/028-inline-around-blocks/README.md`, evidence under `evidence/`,
exit codes inside the files.
