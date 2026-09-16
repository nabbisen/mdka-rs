# RFC 028 — Emphasis wrapping block content emits stray delimiters

**Status.** Accepted 2026-09-16 — implementer may start
**Tracks.** M3 · Conversion fidelity → `2.3.0`
**Priority.** P0
**Sequencing.** After **RFC 025 and RFC 024**. Amended 2026-09-16 — see below.
**Touches.** `src/renderer.rs`, `src/traversal.rs`, `src/utils.rs`.
**Source.** bekoedit field report, 2026-09-16, item 3. Reproduced independently.
**Prepared.** 2026-09-16

## Summary

An emphasis element containing block children emits its delimiters around those
blocks, producing stray `**` lines that are not emphasis in any Markdown dialect.

## The defect

```
<b style="font-weight:normal;"><p>para one</p><p>para two</p></b>
  →  "**\n\npara one\n\npara two\n\n**\n"

<strong><p>x</p><p>y</p></strong>
  →  "**\n\nx\n\ny\n\n**\n"
```

Identical in every mode. The second case shows it is **not** specific to Google
Docs or to the `font-weight:normal` attribute — plain `<strong>` around blocks is
enough.

**Amended 2026-09-16: any block child does it, not only `<p>`.** Confirmed during
the sequencing escalation:

| Input | Output |
|---|---|
| `<em><p>x</p><p>y</p></em>` | `"*\n\nx\n\ny\n\n*\n"` |
| `<b>text<p>para</p></b>` (mixed) | `"**text\n\npara\n\n**\n"` |
| `<b><em><p>x</p></em></b>` (nested) | `"***\n\nx\n\n***\n"` |
| `<b><div>x</div></b>` | `"**\n\nx\n\n**\n"` |
| `<b><ul><li>a</li></ul></b>` | `"**\n\n- a\n\n**\n"` |
| `<b><h2>head</h2></b>` | `"**\n\n## head\n\n**\n"` |

This matters for the fix: **key on the block classification, not on `p`.** A fix
written against the `<p>`-only examples above would have passed its own
acceptance criteria while leaving `div`, `ul` and headings broken.

## Mechanism

`src/renderer.rs`:

| Line | Code |
|---|---|
| `:297` | `"strong" \| "b" =>` … `self.output.push_str("**")` |
| `:303` | `"em" \| "i" =>` … `self.output.push('*')` |
| `:399` | leave `"strong" \| "b" =>` `self.output.push_str("**")` |
| `:403` | leave `"em" \| "i" =>` `self.output.push('*')` |

Unconditional on enter and on leave. Nothing inspects the children. The renderer
has no `is_block`-style predicate anywhere, so there is currently no mechanism
for an inline arm to ask what it contains.

## Why it matters more than its output suggests

Google Docs wraps its entire clipboard payload in `<b style="font-weight:normal">`.
**This fires on essentially every Google Docs paste.** A consumer building
paste-as-Markdown hits it immediately and on the most common input they have.

## Design question — and the non-obvious answer

Markdown has no way to express emphasis spanning blocks, so there is no correct
emission. Two candidate behaviours:

**A · Drop the delimiters when children are blocks.** `para one\n\npara two`.

**B · Distribute the emphasis over each block child.** `**para one**\n\n**para two**`.

**Recommendation: A.**

B is the more obvious choice — it preserves information rather than discarding
it, and discarding information is usually the worse default. **It is wrong here,
and the reason is worth recording:**

Under B, *every Google Docs paste becomes entirely bold*, because the wrapper
`<b>` carries `font-weight:normal` and means the opposite of bold. Today's stray
`**` at least looks like a bug. Fully bold output looks intentional, so a user
would not report it — they would just get wrong documents.

A is also recoverable in the direction that matters: a caller who wanted the
emphasis can see the structure in the source HTML, whereas nobody can undo
spurious bold applied to an entire pasted document.

Genuine `<strong>` wrapping block elements is invalid HTML — `<b>`/`<strong>` are
phrasing content — so A's loss applies to a case that should not occur, while B's
damage applies to one that occurs constantly.

### Not in scope: honouring `font-weight:normal`

Whether `<b style="font-weight:normal">` should be treated as not-bold even
around *inline* content is a separate question, filed with the inline-`style`
candidate from the same report. This RFC fixes the structural defect only, and A
makes the Google Docs case correct without reading `style` at all.

## Relationship to the other M3 renderer work

**Not fixed by [RFC 024](./024-inline-composition-output-sink.md).** That routes
writers through the output sink so inline elements inside `<a>` reach the link
capture buffer. It adds no block-awareness, and these arms would still push
delimiters unconditionally. Adjacent code, different defect.

### Sequencing amended 2026-09-16 — after RFC 024 as well

A second mechanism exists: **buffer the emphasis content and decide on leave**,
reusing the `InlineCapture` pattern the link path already uses. It needs no tree
query at all.

RFC 024 generalises exactly that machinery. If RFC 024 lands first the buffer
route may be nearly free; if RFC 028 lands first it builds a parallel mechanism
RFC 024 then has to absorb. The choice between the tree-query and buffer routes
should be made when RFC 024's shape is known.

The cost is that a P0 defect waits longer. That is the correct trade: a P0 defect
fixed twice is worse than a P0 defect fixed once, later.

**Not caught by [RFC 025](./025-output-validity-harness.md) as specified.** Its
composition matrix is inline-construct × container — every cell puts an inline
thing inside a container. This defect is a *block inside an inline*, the other
direction. RFC 025 is amended to cover both.

## Scope

Both element pairs — `strong`/`b` and `em`/`i` — and the nesting of one inside
the other. Whatever predicate decides "has block children" should be usable by
any future inline arm with the same problem, rather than special-cased twice.

## Compatibility

Output changes for documents where an emphasis element wraps block content.
Every such change is from invalid Markdown to valid Markdown. Minor version,
`2.3.0`, with a CHANGELOG entry showing before and after.

## Risks

| Risk | Mitigation |
|---|---|
| The block predicate disagrees with the arms that actually emit blocks | Derive it from the same classification those arms use, not a second list. Two disjoint lists is the `figure`/`figcaption` defect from RFC 003. |
| Emphasis with *mixed* inline and block children | Define and test it. `<b>text<p>para</p></b>` is real in clipboard HTML. |
| Deeply nested emphasis around blocks | `<b><em><p>x</p></em></b>` must not emit either delimiter. Test it. |
| ~~html5ever restructures the tree before the renderer sees it~~ | **Closed 2026-09-16.** It does not: `<strong><p>` reaches the renderer with the `<p>` as a child, demonstrated by the nested and mixed outputs above. |
| A descendant scan per emphasis element is O(subtree), so nested emphasis is O(n²) | **Requirement: O(n) total over the document, not O(n) per emphasis element.** A single bottom-up pass in `traversal.rs` satisfies it. The Google Docs shape — one `<b>` wrapping an entire payload — is exactly the bad input. |
| A is wrong and B was right | The reasoning above turns on Google Docs' wrapper being the dominant real-world producer. If the corpus (below) shows otherwise, **report before implementing.** |

## Acceptance criteria

1. `<strong><p>x</p><p>y</p></strong>` → `"x\n\ny\n"`, no stray delimiters.
2. The Google Docs shape from the report produces `"para one\n\npara two\n"`.
3. `em`/`i` behave the same way.
4. Nested emphasis around blocks emits no delimiters.
5. Mixed inline-and-block children have defined, tested behaviour.
6. **Emphasis around purely inline content is byte-identical to `2.2.1`.** This is
   the overwhelmingly common case and must not move.
7. The block predicate is shared with, not duplicated from, the block-emitting
   arms.

## Verification input

bekoedit has offered a corpus of real clipboard HTML from browsers, Google Docs,
Word and LibreOffice. **That corpus is the right verification for this RFC** — it
is the input that exposed the defect, and the input we demonstrably cannot
generate for ourselves. Sequence this after the corpus arrives if that is not a
long wait.
