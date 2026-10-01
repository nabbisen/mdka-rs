# RFC 050 — A computed style that restates a tag's own default is not new information

**Status.** Implemented — shipped in `3.1.1`, 2026-10-01
**Author.** Architect
**Created.** 2026-10-01
**Milestone.** `3.1.1`.
**Source.** bekoedit, 2026-10-01, raised as *"a heads-up, not a report"* — and measured here before
they ran it: **they are right, and it is wider than the case they named.**
**Touches.** `src/renderer.rs` (`own_emphasis`), `docs/src/api/options.md`, tests.

---

## 1. The defect

`emphasis_from_style` (RFC 049, shipped `3.1.0`) adds emphasis whenever an element's inline `style`
says bold or italic. It knows that `<b>`/`<strong>` are already bold and `<i>`/`<em>` already italic —
`own_emphasis`'s `tag_default` is exactly those four — so for them a `font-weight: 700` restates what
the tag already meant and nothing doubles.

**It knows nothing about any other element whose default rendering is bold or italic.** For those, the
same restatement is read as *new* emphasis and is added to the output.

Measured on the **published** `3.1.0` CLI, `Balanced --drop-shell --no-preserve-ids`, option off vs on:

| Input | off | on | |
|---|---|---|---|
| `<h1 style="font-weight:700">H</h1>` | `# H` | `# **H**` | ❌ |
| `<h3 style="font-weight:bold">H</h3>` | `### H` | `### **H**` | ❌ |
| `<cite style="font-style:italic">C</cite>` | `C` | `*C*` | ❌ |
| `<address style="font-style:italic">A</address>` | `A` | `*A*` | ❌ |
| `<var style="font-style:italic">v</var>` | `v` | `*v*` | ❌ |
| `<dfn style="font-style:italic">d</dfn>` | `d` | `*d*` | ❌ |
| `<b style="font-weight:700">b</b>` | `**b**` | `**b**` | ✅ known default |
| `<em style="font-style:italic">i</em>` | `*i*` | `*i*` | ✅ known default |
| `<th style="font-weight:700">H</th>` | `\| H \|` | `\| H \|` | ✅ **by accident** |

`<th>` is spared only because RFC 049 never reaches a table cell — a documented *gap* is the one thing
preventing a defect. That is not a safety property; it is a coincidence, and it would evaporate if the
cell boundary were ever widened.

## 2. Why this is not a corner case

**It is the option's own motivating input.** bekoedit's probe of a real paste found WebKitGTK rewrites
clipboard HTML before the page sees it: 84 characters of source HTML arrived as 1,469, *"beginning
`<h1 style="caret-color: rgb(0, 0, 0); …` — every element carries its computed style inline."*

A computed style is the **flattened** result of the UA default stylesheet plus the page's own CSS. It
cannot distinguish *"the author made this bold"* from *"this is an `<h1>`."* So in exactly the input
RFC 049 exists to serve, **every heading gains `**…**`**.

RFC 049 §2 decided seven cases. None involved a tag whose own default is bold other than
`<b>`/`<strong>`. The rule *"mdka reads the element, not the environment"* is right and is not in
question here — the element's **tag** is part of the element, and the current rule ignores half of it.

## 3. What markdown can and cannot say

For `<b>`, bold is expressible and `**` is the right output. **For a heading it is not:** `#` already
carries the whole meaning, and `# **H**` is not "a bold heading" — it is a heading containing bold
text, which the source did not say. There is no markdown for "heading, and also bold", nor any need.

`<cite>`, `<address>`, `<var>` and `<dfn>` are a different shape: markdown has no native form, and
mdka renders them **plain** today. So the option makes their output depend on *whether the clipboard
happened to inline the UA default* — the same document, pasted from two browsers, converts
differently. **The inconsistency is the defect there, more than the italics.**

## 4. Proposal

Extend `own_emphasis` with the full set of elements whose **UA default stylesheet** is bold or italic,
and make a style that merely **restates** that default contribute nothing:

- **Bold by default:** `b`, `strong`, `h1`–`h6`, `th`
- **Italic by default:** `i`, `em`, `cite`, `address`, `var`, `dfn`

Then:

| the style says | the tag's default | result |
|---|---|---|
| bold | already bold | **no change** — restatement, not information |
| bold | not bold | adds emphasis (today's behaviour, unchanged) |
| not bold | already bold | **removes** it — today's behaviour, unchanged, shipped since `3.0.0` |
| nothing | already bold | the tag's own meaning, unchanged |

For a heading the first row means: no `**` added, and `#` keeps carrying it. For `<cite>` it means the
output is the same whether or not the clipboard inlined the default — which is the property that
matters.

**Nothing about the negation path changes.** `<b style="font-weight:400">` still un-bolds, as it has
since `3.0.0`.

## 5. Decided

1. **`<th>` — settle it on purpose.** Today it is protected by the table-cell gap. Should `th` be in
   the bold-default set regardless, so the protection survives if that gap is ever closed?
   **Decided: yes.** One entry, and it removes a dependency on an unrelated boundary.
2. **`<cite>`/`<address>`/`<var>`/`<dfn>`: suppress, or render them italic always?** This RFC proposes
   **suppress** — keep today's plain output and make it independent of the clipboard. Rendering them
   italic unconditionally is defensible but is a separate output change for every user, not a fix.
   **Decided: suppress.** Rendering them italic unconditionally is a separate output change for every user, not a fix; propose it separately if anyone asks.

## 6. Urgency

**No shipped user is affected.** The option is opt-in, off by default, and bekoedit has explicitly not
turned it on yet. But it is broken for the input it was built for, and they are about to run exactly
this test. **They should be told before they spend that effort**, which is a letter, not a release.

## 7. Criteria

1. Every row of §1's table: option on, output **identical to option off** for the ✅ and ❌ rows alike.
2. The negation path unchanged — `<b style="font-weight:400">`, `<em style="font-style:normal">`.
3. A `<span style="font-weight:700">` inside a heading still works: the span is not a heading, and
   `# <span style=bold>H</span>` should still give `# **H**` — **an authored bold inside a heading is
   real.** Decide and test this explicitly; it is the one case where the fix could over-reach.
4. A realistic WebKitGTK-shaped document — every element carrying a computed `style` — converts with
   no emphasis the source did not mean.
5. Option off stays byte-identical to `3.1.0`; `mode_identity`'s goldens untouched.
