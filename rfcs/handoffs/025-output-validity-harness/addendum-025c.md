# Addendum — RFC 025 · slice `025c`

**To.** Implementer (mid-capability model)
**From.** Architect
**Date.** 2026-09-16
**RFC.** [RFC 025](../../accepted/025-output-validity-harness.md)
**Base.** The harness at `7338b17`, approved: `.git-exclude/reviewed/025-output-validity-harness/README.md`
**Size.** Small–medium. `tests/output_validity/**` only.

---

## 0. Why this is a separate file

You built from the handoff as it stood before `18c1911`. That was my error: I kept
editing the handoff in place after it was ready to hand over. From now on a handoff is
**frozen once handed over**, and later changes arrive as a dated addendum like this one.
Nothing in this file asks you to redo work; it carries what `18c1911` added, plus
decisions from the review.

**This addendum is an instruction to start.** No `src/` change.

## 1. The single-paragraph Google Docs cell

Public captures (ProseMirror #459, 2016; MarkText #4688, 2026) show Google Docs wraps
copied content in a non-bold `<b style="font-weight:normal" id="docs-internal-guid-…">`.
You have the **multi-paragraph** shape (bekoedit item 3). Add the **inline** shape,
written by you in that form — do not paste third-party HTML:

```html
<b style="font-weight:normal;" id="docs-internal-guid-x"><span style="font-weight:400">Hello world</span></b>
```

Intent: a paragraph `Hello world` with **no** emphasis. Today: `**Hello world**`
(minimal), `**<a id="docs-internal-guid-x"></a>Hello world**` (balanced).

Owner: **UNOWNED**, with a comment that it is pending the owner's decision on RFC 028's
proposed style amendment. It will be re-labelled by that decision, not by you.

## 2. Provenance

1. **Move** `corpus/clipboard-fragment.html` and `corpus/article.html` out of `corpus/`
   into a directory for **hand-written runner fixtures** (e.g. `runner_fixtures/`).
   `corpus/` is where bekoedit's real captures will land, each with metadata; a
   hand-written "Word-style clipboard fragment" beside them would make the directory's
   provenance unreliable. Keep `corpus_proof/` as it is. Leave `corpus/` absent or empty
   with a `README` saying what belongs there — **and adjust the runner test so an empty
   `corpus/` is not reported as "nothing was checked"**: the fixtures test runs against
   the new directory; the corpus directory is exercised by `025b`.
2. **Comments at the bekoedit cells:** say the reproductions were **hand-written by
   bekoedit** and vendored with their permission (letter of 2026-09-16). For the Google
   Docs wrapper, cite the **public captures** above, not bekoedit.
3. Consider renaming `real_world.rs` → `field_reports.rs`: its cells are reproductions
   from reports, not real-world input. Your call; say which.

## 3. Owner re-labels decided at review

| Cells | From | To | Why |
|---|---|---|---|
| `img_in_code`, `strong_in_code`, `em_in_code`, `a_in_code` (Q3) | UNOWNED | **RFC 024** | a code span holds text only; inline composition |
| `img_in_blockquote`, `strong_in_blockquote`, `em_in_blockquote`, `code_in_blockquote`, `a_in_blockquote` (Q7) | UNOWNED | **RFC 024** | mechanism verified: inline arms bypass `emit_pending_prefix()` |

Change the `defect(...)` owner and reason text. **Do not change any expectation.**

The `<a>`/`<code>` × block cells (Q4–Q6) **stay UNOWNED** until the owner decides on
extending RFC 028.

## 4. Link content survives (Q10)

Add an intent-free property: for every `<a href>` outside code, the parsed `Link` with
that destination **contains the link's text and any image** the HTML put inside it.
Name it (e.g. `[link-content]`).

Prove it: red on `img_in_a` and on `strong_in_a` through the **corpus runner** (a
fixture file, not only the tree cell), green on a correct link. Report how many
existing cells it newly catches.

## 5. Parse under GFM as well (Q8d)

Most Markdown readers people use are GFM, and text mdka emits unescaped can become
syntax there — `~~x~~` as strikethrough, a line of `|`-separated text as a table.

Run **every property and every tree assertion under both** plain CommonMark and a GFM
option set (`pulldown-cmark` 0.13.4: tables, strikethrough, task lists, and whatever else
it offers under GFM — **list what you enabled**). A cell passes only if both readings
are right; the failure message says which reading failed.

⚠ **Tree expectations may need a GFM-aware form only where GFM legitimately parses the
same text differently** — for example a strikethrough event. **Do not edit an existing
expectation to make a GFM failure pass.** If the GFM reading differs because mdka failed
to escape, that is a defect: mark it `known_defect(Rfc010Planned, …)` with a reason naming
GFM.

Add at least these cells, expectation from the HTML's meaning:

- `<p>~~not struck~~</p>` → paragraph with that literal text
- `<p>a | b | c</p><p>--- | --- | ---</p>` → two paragraphs of literal text
- `<p>[ ] not a task</p>` inside `<li>` → list item with that literal text

Report every new failure, both readings.

## 6. Out of scope

- The per-file known-defect sidecar for the corpus (Q11) — specified in RFC 025 for
  `025b`, not built now.
- Any renderer change.

## 7. Acceptance checklist

- [ ] §1 Google Docs inline cell, UNOWNED with the pending-decision comment
- [ ] §2 hand-written fixtures moved out of `corpus/`; empty `corpus/` handled honestly; bekoedit and public-capture provenance in comments; `real_world.rs` naming decided
- [ ] §3 nine cells re-labelled to RFC 024; **no expectation changed**
- [ ] §4 `[link-content]` property, red through the runner, count of newly caught cells
- [ ] §5 dual CommonMark/GFM parsing; GFM options listed; three GFM cells; every new failure reported with the reading that failed
- [ ] Inventory re-issued with owners after these changes
- [ ] `cargo test --workspace --all-features --locked --no-fail-fast` green; count reconciled against 258
- [ ] fmt; clippy per CI

## 8. Report back

`.git-exclude/review-request/025c-harness-addendum/README.md`, evidence under
`evidence/`, exit codes inside the files.
