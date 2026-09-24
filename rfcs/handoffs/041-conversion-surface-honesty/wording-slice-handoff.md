# Developer Handoff — RFC 041 §6.2 · Say what the modes actually do, and close the orphaned docs findings

**RFC.** `rfcs/accepted/041-conversion-surface-honesty.md` §9 — owner answered §6 on 2026-09-24
**Priority.** P2. Nothing here is a regression; all of it is text that is untrue in effect and **published**
**Prepared.** 2026-09-24
**Baseline.** `86b51ca` — 583 Rust, 42 Node, 25 loader, 90 Python; seven workflows green
**Scope.** Documentation only. **No code, no options, no modes, no version, no CHANGELOG.** Nothing here
changes a byte of conversion output.

---

## 0. Why this is worth a slice of its own

The structural half of RFC 041 — actually collapsing the surface — is a `3.0` change, unscheduled, and bound
to RFC 039 Half B. **This half is not.** It corrects text that is wrong today and is live on crates.io, on
PyPI, in the README and on the docs site, and it costs nothing to be honest in the meantime.

It also closes four findings from the `2.3.0` consumer pass that were homed to a *"Docs slice"* which was
never opened. **That is the actual lesson here: a home that is a category rather than a document becomes
nowhere.** They are in `ROADMAP.md` now; this slice is the document.

## 1. The mode wording 🛑

`Strict` and `Preserve` differ from `Balanced` only in fields that are inert **permanently** — Markdown has
no attribute syntax — and `Semantic` joined them at `2.4.2`. They are not modes that happen to coincide;
there is **no mechanism by which they could diverge.** Every line below promises otherwise.

### `README.md`, the "Why mdka?" bullet (≈ line 42)

> …let you tune what gets kept or stripped, **from noise-free LLM input to maximum retention.** Four of the
> five currently produce identical output…

`Preserve` retains nothing more than `Balanced`. Proposed:

> Five [conversion modes](…), of which **two convert differently**: `Balanced` (the default) and `Minimal`,
> which strips to body text and structure for LLM input. The other three are aliases of `Balanced` — see
> [Conversion Modes](…).

### `README.md`, the Conversion Modes section (≈ lines 179–191)

Replace the five-row table and the paragraph under it with:

```markdown
| Mode | Use when |
|---|---|
| `Balanced` | General use — the default |
| `Minimal` | LLM input, text extraction — the only mode that converts differently |
| `Strict`, `Semantic`, `Preserve` | Aliases of `Balanced`; kept for compatibility |

**`Balanced`, `Strict`, `Semantic` and `Preserve` produce identical output, and cannot differ.** They vary
only in the defaults of options that have no effect on Markdown — the format has no syntax for HTML
attributes — so there is no mechanism by which they could diverge. Choosing between them changes nothing.
```

**Do not write that they will be removed.** The decision to collapse is made, but it is unscheduled and tied
to another RFC; announcing a removal we have not scheduled would replace one false statement with another.

### `docs/src/api/modes.md`

The same correction, in that page's own voice. Line 34 currently says the four *"remain distinct API, are not
merged, and **may diverge again** if…"* — that conditional cannot be satisfied, and the page is where a
reader goes for the full explanation, so it should carry the reasoning: the distinguishing fields are inert
permanently, because Markdown has no attribute syntax.

Also correct the individual mode descriptions on that page — `Strict` *"for debugging and comparison"*,
`Semantic` *"for SPAs and accessibility"*, `Preserve` *"for archiving and auditing"* — each promises a
fidelity difference that does not exist. `Minimal`'s description is accurate and stays.

## 2. The four orphaned findings 🛑

All four verified still undone on 2026-09-24.

| # | Change |
|---|---|
| 1 | **`README.md` Node Quick Start** showcases `htmlToMarkdownWithAsync`. That is **the one path that cannot emit a deprecation warning** — a napi-rs constraint (`node/src/lib.rs:42–49`) — so a reader following our own example, using a deprecated option, is warned about nothing. Add a short caveat pointing at the sync form. `docs/src/getting-started/usage-nodejs.md:132` is already correct; match it, do not invent a second explanation |
| 2 | **`docs/src/api/elements.md`**, the `<strong>, <b>` row, never mentions that mdka reads inline `style` to **suppress** emphasis (`font-weight:normal` emits no delimiters) — nor that bold carried *only* by `<span style="font-weight:700">` is lost. Both are real behaviour; the page is the reference for exactly this |
| 3 | **`README.md`**: *"They remain distinct API"* → *"APIs"* — moot if §1 replaces the sentence, but check the corrected text reads correctly |
| 4 | **`README.md`** mixes link styles: two links use `api/modes.html`, two use `api/modes`. Pick one and use it throughout — note that other pages are referenced both ways too, so sweep the file rather than fixing the four you are told about |

## 3. Verify, do not assume

Every claim you write must be checked against a **`2.5.1` build**, not against this handoff:

- The mode-identity claim: convert a corpus in all five modes and confirm `Balanced` ≡ `Strict` ≡ `Semantic`
  ≡ `Preserve`, and that `Minimal` differs. `tests/output_validity/mode_identity.rs` already asserts exactly
  this — **cite it rather than re-deriving it**, and say in the report that you did.
- Finding 1: confirm for yourself that the async path emits no warning and the sync path does.
- Finding 2: confirm both behaviours — `font-weight:normal` suppressing emphasis, and `font-weight:700`
  being lost.

## 4. Not in this slice

- **Any code, option, mode or default.** The structural collapse is `3.0` and belongs with RFC 039 Half B.
- **`<sup>` fallback notation** and **`py.typed` stubs** — both decided, both getting their own RFCs.
- **`mdka_python` in the public namespace** — recorded against RFC 041's structural half.
- Version, CHANGELOG, release prep.

## 5. Criteria

1. No sentence in `README.md` or `docs/src/` states or implies that `Strict`, `Semantic` or `Preserve`
   converts differently from `Balanced`, or may come to.
2. No sentence claims a removal or deprecation that has not been scheduled.
3. The four findings in §2 are closed, each verified against a `2.5.1` build.
4. `mdbook build` succeeds; the docs example gate passes; seven workflows green.
5. **No change to any file outside `README.md` and `docs/src/`.**

## 6. Report back

`.git-exclude/review-request/041-wording-slice/README.md`. For each claim you changed, say what you ran to
confirm the replacement is true. This slice exists because published text drifted from behaviour; a report
that asserts the new text is correct without showing it repeats the original fault.

## 7. Committing and pushing

Only work that is yours and approved. Report first — approval comes through the review, and a green run is
not approval. Tagging, releasing and triggering release workflows are not yours.

**One sequencing point:** `docs.yaml` deploys on every push to `main`. Everything in this slice is a
correction of something already untrue, so publishing it early makes the site *more* accurate — unlike RFC
040 and 042, there is nothing to hold back here.
