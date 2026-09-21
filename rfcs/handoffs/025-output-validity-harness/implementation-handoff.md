# Developer Handoff — RFC 025 · Markdown output-validity harness

> **Frozen 2026-09-16 — implemented at `7338b17`.** Later changes arrive as dated addenda, never as edits here: [`addendum-025c.md`](./addendum-025c.md).

**Governing RFC.** [RFC 025](../../done/025-output-validity-harness.md) — including its 2026-09-16 amendments
**Milestone.** M3 → `2.3.0`
**Priority.** P0 — first of the engine work: **025 → 024 → 028**
**Prepared.** 2026-08-31. **Revised 2026-09-16** — see §0.

---

## 0. Preconditions — met. This handoff is an instruction to start.

The original said *"Do not start until M2b has shipped."* M2b (2.2.2) and M2c
(2.2.3) have shipped, and the control repairs 030–034 are approved.

**What the revision changed, so nothing is silently different:**

| # | Change | Why |
|---|---|---|
| 1 | §4 matrix now has **both directions** | RFC 025 was amended 2026-09-16 to require block-inside-inline; the original handoff was never updated and still specified only one direction — the gap RFC 028's defect lives in |
| 2 | Baseline **139** tests, not 136 | re-derived 2026-09-16: `cargo test --workspace --all-features --locked --no-fail-fast` → 139 passed, 0 ignored |
| 3 | Known-defect cells are **strict expected failures**, not `#[ignore]` | §5 — an ignored cell never runs, so it cannot tell RFC 024/028 when their fix has landed; RFC 025 amended to match |
| 4 | Owners named precisely: **RFC 024, 028 (accepted); RFC 010 (planned — ROADMAP row, no file yet)** | an owner must be something a reader can find |
| 5 | bekoedit's reproductions become cells now; the **corpus** is a later slice | §7 — requested 2026-09-16, not yet received |
| 6 | Verification commands match CI as it now runs | RFC 032 |
| 7 | **Revised again, 2026-09-16 (later):** §7 retitled — bekoedit's reproductions are **hand-written**, the corpus **does not exist yet**; two public Google Docs clipboard shapes added as cells (§7.1) | bekoedit's correction letter; `.git-exclude/reviewed/upstream-bekoedit-2026-09-16-re-corpus/README.md` |

## 1. Purpose

No test verifies that mdka's output is Markdown. Every renderer assertion
compares against a string a human wrote. Add a harness that parses the output
with a real CommonMark parser and checks what comes back.

## 2. Why this is first

139 tests are green today while all of this is live:

```
<a href="/page"><img src="i.png" alt="pic"></a>  →  ![pic](i.png)[](/page)
<a href="/x"><strong>b</strong></a>              →  ****[b](/x)
<pre>plain</pre><p>After</p>                     →  plain\n```\n\nAfter
<code>snake_case</code>                          →  `snake\_case`
<a href="/a b.html">x</a>                        →  [x](/a b.html)
<strong><p>x</p><p>y</p></strong>                →  stray ** lines   (RFC 028)
```

The architect re-derived all six with a release build of the current tree on
2026-09-16 — outputs exactly as shown (the last renders as `**\n\nx\n\ny\n\n**`).
**Re-derive them again when you start**, and record the actual output: your
harness, not this table, is the evidence.

Each has tests asserting the wrong answer or no test at all. **Your deliverable is
not green tests. It is an honest inventory of where the output is not Markdown.**

## 3. The parser

Add `pulldown-cmark` as a **dev-dependency only** of the `mdka` crate. Verify with
`cargo tree -e normal -p mdka` that it is absent from the published graph, **and**
that `cargo package --workspace` (RFC 030's gate) still passes.

Assert on the **parsed event stream**, never on output bytes:

```
HTML → mdka → Markdown → pulldown-cmark → events → assert structure
```

Pin `pulldown-cmark` to an exact version in `Cargo.toml`, and say in a comment
that the harness models that parser's reading of CommonMark. A parser upgrade can
change which cells pass; that should be a reviewed change, as with the mdBook and
TypeScript pins.

## 4. The composition matrix — both directions

### 4.1 Inline inside container

| | in `<a>` | in `<pre>` | in `<code>` | in `<li>` | in `<blockquote>` | in heading |
|---|---|---|---|---|---|---|
| `<img>` | | | | | | |
| `<strong>` | | | | | | |
| `<em>` | | | | | | |
| `<code>` | | | | | | |
| `<a>` | | | | | | |
| text with `_ * [ ] ( )` | | | | | | |

### 4.2 Block inside inline — **required, not optional**

RFC 025's 2026-09-16 amendment:

| | in `<strong>` | in `<em>` | in `<a>` | in `<code>` |
|---|---|---|---|---|
| `<p>` | | | | |
| `<ul>`/`<li>` | | | | |
| `<blockquote>` | | | | |
| `<pre>` | | | | |
| heading | | | | |

Most of these are invalid HTML. **That is why they are here** — real clipboard HTML
is full of invalid markup (bekoedit's item 3 reproduces Google Docs' clipboard wrapper — a shape confirmed independently by public captures, §7.1).

### 4.3 Each cell

Convert, parse, assert the structure matches what the HTML meant.

**What "meant" is, for invalid HTML, is often a judgement.** Where it is genuinely
ambiguous, do not pick — record the question (§10). At minimum every cell can
assert the **intent-free properties** in §6: no stray delimiter text, text content
not lost, links and images not lost.

## 5. Known defects — strict expected failures, not `#[ignore]`

**Changed from the original handoff, and RFC 025 is amended to match.**

An `#[ignore]`d cell never runs. When RFC 024 or 028 fixes the defect, nothing
tells anyone: the cell stays ignored, the inventory goes stale, and "the fix
removed the defect" is unverified.

Instead, write a helper — e.g. `known_defect(owner, cell)` — with these semantics:

| Cell result | Helper |
|---|---|
| structure **does not** match intent | **pass** — the defect is still there, as recorded |
| structure **matches** intent | **fail**: *"known defect now passes — remove the known_defect marker (owner: RFC 0NN)"* |
| conversion panics, parser errors, harness bug | **fail** — never counted as "still defective" |

The third row is the trap. A strict expected failure that also swallows panics
hides real breakage behind a green test. **The helper must distinguish "wrong
structure" from "could not evaluate".**

Every marked cell names its owner:

- **RFC 024** — inline composition (inline-in-link, bare `<pre>`)
- **RFC 028** — emphasis around block content (§4.2 cells)
- **RFC 010 (planned)** — escaping, destinations, fences
- **UNOWNED** — anything else. List these in the review request; they are the
  most interesting output of this RFC.

**Never delete a cell to make the suite pass.**

### 5.1 Prove the helper

- A cell known to fail, marked → passes.
- The same cell with the marker, against a stubbed "correct" output → **fails with
  the message.**
- A cell whose conversion panics, marked → **fails**, not passes.

## 6. Properties — they need no per-cell intent

### 6.1 Escaping round-trip

Whatever mdka escapes must parse back to the **original characters**.
`snake_case` inside a code span must emerge as `snake_case`, not `snake\_case`.
For text: `<p>1986. A great year</p>` (bekoedit item 2) must parse as a paragraph
whose text is `1986. A great year`. Today mdka emits `\1986. A great year` — a
backslash before a digit, which CommonMark does not treat as an escape, so the
backslash survives into the text.

### 6.2 Well-formedness

- A fenced block's opening fence is longer than any backtick run in its content.
- A destination containing a space, `(`, `)` or `"` parses back to the original URL.

### 6.3 Nothing lost, nothing stray

- **No stray delimiter text:** parsed text events contain no leftover `**`, `*`, `_`
  or `` ` `` runs that came from mdka's own emphasis/code output.
- **Links and images survive:** each `<a href>` yields a `Link` event with that
  destination; each `<img src>` an `Image` event.
- **Text survives:** the parsed text, whitespace-normalised, equals the HTML's
  visible text, whitespace-normalised — in a mode that drops nothing (check which;
  do not assume). If the normalisation rules are genuinely ambiguous, **record the
  question** rather than weaken the property.

These properties are what make the harness **corpus-ready** (§7): they need no
hand-written expectation per input.

## 7. External input

### 7.1 Now — bekoedit's reproductions, and Google Docs' documented wrapper

`.git-exclude/upstream/bekoedit/receive/2026-09-16-html-to-markdown-conversion-gaps.md`.
Item **2** (digit-period escape) and item **3** (emphasis around blocks) are
validity defects: add each reproduction as a cell, owners RFC 010 (planned) and
RFC 028.

Items 1, 5, 6 (tables, strikethrough, task lists) are **coverage**, not validity —
the output is valid Markdown that loses structure. Not this harness. Items 4, 7, 8,
9 are options and style. Not this harness.

These are **hand-written by bekoedit**, not clipboard captures — they said so in a
later letter. They are still external to our model. bekoedit has given permission
to vendor them; credit bekoedit in a comment at the cells.

**Do not copy the letter into the repository.** Take the minimal HTML only.

**Two Google Docs shapes, as cells.** Public issue reports quoting real clipboard
captures ([ProseMirror #459](https://github.com/ProseMirror/prosemirror/issues/459),
2016; [MarkText #4688](https://github.com/marktext/marktext/issues/4688), 2026) show
Google Docs wraps copied content in `<b style="font-weight:normal" id="docs-internal-guid-…">`,
with real emphasis only in inner `<span style>`. **Write your own fixtures in both
shapes; do not paste their HTML:**

| Shape | Minimal form | Today (architect, 2026-09-16) |
|---|---|---|
| single paragraph — wrapper around **inline** content | `<b style="font-weight:normal;" id="docs-internal-guid-x"><span style="font-weight:400">Hello world</span></b>` | `**Hello world**` in minimal; `**<a id="docs-internal-guid-x"></a>Hello world**` in balanced |
| multi paragraph — wrapper around **blocks** | `<b style="font-weight:normal" id="docs-internal-guid-x"><p>one</p><p>two</p></b>` | `**\n\none\n\ntwo\n\n**` |

Owner of the multi-paragraph cell: **RFC 028**. Owner of the single-paragraph cell:
**RFC 028 if the owner accepts its proposed amendment, otherwise UNOWNED** — mark it
UNOWNED for now and say so; the review will re-label it.

### 7.2 Later — the corpus: slice `025b`

RFC 025 says to use bekoedit's clipboard corpus. **It does not exist yet.** The
request went out 2026-09-16; bekoedit replied that captures have not been made,
will be taken by hand, and have no date. **`025b` is unscheduled.** Do not wait for
it.

**Build the runner so a directory of `.html` files can be run through §6's
properties with no per-file expectations.** Prove it with two or three
hand-written files. When the corpus arrives, `025b` adds data, not machinery.

## 8. What this is not

Not a CommonMark conformance suite — mdka is a producer. The obligation is that
**what it emits parses to what it meant**.

Not a differential test against another converter. Peer output is not a spec.

**Scope boundary, per RFC 027 Rule 2.** This handoff builds the harness and records
failures. It **fixes nothing** — RFC 024, 028 and 010 do that. If you find yourself
changing `src/renderer.rs`, stop: a harness written by the person fixing the bugs
is the trap that produced the current suite.

## 9. Required verification

Per RFC 027 Rule 3, state what each ran against.

1. `cargo tree -e normal -p mdka` without `pulldown-cmark`; `cargo package --workspace` green.
2. §2's examples re-derived against the current tree.
3. Both matrices, every cell asserting or marked `known_defect` with an owner.
4. §5.1 helper proofs.
5. §6 properties, each shown **failing** on at least one real cell or a deliberate
   break — a property never seen failing is decoration.
6. §7.2 runner over a directory, with one file deliberately violating a property.
7. **The inventory**: every marked cell, owner, and the one-line reason — **the
   primary deliverable**. Plus the UNOWNED list and every recorded question.
8. `cargo test --workspace --all-features --locked --no-fail-fast` green; count
   reconciled against **139** plus additions.
9. fmt; clippy `--workspace --all-targets --all-features --keep-going -- -D warnings`.

## 10. Known risks

| Risk | If it happens |
|---|---|
| A cell's intended structure is genuinely ambiguous | Record the question. Ambiguity is a design input to RFC 024/028/010. |
| The matrix is bigger than expected | Fine. Small string conversions. |
| Cells pass that you expected to fail | Say so. Our model of the defects may be wrong. |
| A property is too strict for legitimate output | Record the case and the question; do not quietly loosen it. |
| The inventory is large enough to change M3's shape | §11. |

## 11. Escalate rather than decide

Stop and raise if: a cell's correct structure is genuinely undecidable and blocks
the helper; a failure looks like it needs a renderer change to even express the
test; the inventory is large enough to change M3's shape; or `pulldown-cmark` would
have to enter the normal dependency graph.

## 12. Acceptance checklist

- [ ] `pulldown-cmark` dev-only and pinned; absent from `cargo tree -e normal -p mdka`; crates package gate green
- [ ] §2 examples re-derived
- [ ] §4.1 **and** §4.2 matrices; every cell asserts or is `known_defect` with an owner
- [ ] §5 helper; §5.1 three proofs, including "panic is not a pass"
- [ ] §6.1–6.3 properties, each seen failing
- [ ] §7.1 bekoedit items 2 and 3 as cells
- [ ] §7.2 directory runner, proven with a violating file
- [ ] **Full inventory in the review request**, with UNOWNED list and questions
- [ ] No renderer change whatsoever
- [ ] CI green; count reconciles against 139

## 13. Report back

`.git-exclude/review-request/025-output-validity-harness/README.md`, evidence under
`evidence/`, exit codes inside the files.
