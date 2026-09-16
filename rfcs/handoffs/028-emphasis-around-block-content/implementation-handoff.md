# Developer Handoff — RFC 028 · Emphasis wrapping block content

**Governing RFC.** [RFC 028](../../accepted/028-emphasis-around-block-content.md)
**Milestone.** M3 → `2.3.0`
**Priority.** P0
**Sequencing.** After [RFC 025](../025-output-validity-harness/implementation-handoff.md) lands. **Do not start before M2b ships.**
**Prepared.** 2026-09-16

---

## 1. Purpose

An emphasis element containing block children emits its delimiters around those
blocks, producing stray `**` lines that are not emphasis in any Markdown dialect.

## 2. Reproduce it first

```
<strong><p>x</p><p>y</p></strong>              →  "**\n\nx\n\ny\n\n**\n"
<b style="font-weight:normal;"><p>one</p><p>two</p></b>
                                               →  "**\n\none\n\ntwo\n\n**\n"
```

Identical in every mode. **Note the first case**: plain `<strong>`, no style
attribute, no Google Docs. The defect is structural, not a vendor quirk — the
reporter framed it partly around Google Docs and the sharper framing is ours.

## 3. Mechanism — read, not inferred

| `src/renderer.rs` | |
|---|---|
| `:297` | `"strong" \| "b" =>` … `self.output.push_str("**")` |
| `:303` | `"em" \| "i" =>` … `self.output.push('*')` |
| `:399` | leave `"strong" \| "b" =>` `self.output.push_str("**")` |
| `:403` | leave `"em" \| "i" =>` `self.output.push('*')` |

Unconditional on both sides. **There is no `is_block`-style predicate anywhere in
`renderer.rs` or `utils.rs`** — I checked. So an inline arm currently has no way
to ask what it contains, and adding that capability is most of this slice.

## 4. The required behaviour — and why it is the less obvious one

**When an emphasis element has block children, emit no delimiters for it.**

```
<strong><p>x</p><p>y</p></strong>   →   "x\n\ny\n"
```

The obvious alternative is to distribute the emphasis over each block child
(`**x**\n\n**y**`), which preserves information rather than discarding it.
**That is wrong here, and RFC 028 records why:** Google Docs wraps its entire
clipboard payload in `<b style="font-weight:normal">`, so distributing would
make *every Google Docs paste entirely bold*. Today's stray `**` at least looks
like a bug; fully bold output looks intentional and nobody reports it.

**If you conclude distribution is right after all, stop and report** rather than
choosing. The decision turns on which real-world producer dominates, and the
corpus (§7) is the evidence that would change it.

## 5. Scope

Both pairs — `strong`/`b` and `em`/`i` — plus nesting of one inside the other.

**The block predicate must be shared with, not duplicated from, the arms that
actually emit blocks.** Two disjoint lists that must agree is the
`figure`/`figcaption` defect from RFC 003, and it has already cost this project
once.

Cases that must be defined and tested, not left to fall out:

- **Mixed children** — `<b>text<p>para</p></b>`. Real in clipboard HTML.
- **Nested emphasis around blocks** — `<b><em><p>x</p></em></b>` must emit
  neither delimiter.
- **Emphasis around purely inline content must be byte-identical to `2.2.1`.**
  This is the overwhelmingly common case and must not move.

### Scope boundary, per RFC 027 Rule 2

This handoff covers the structural defect only: emphasis elements whose children
are blocks.

**Not covered:** whether `<b style="font-weight:normal">` should mean "not bold"
around *inline* content. That needs reading the `style` attribute, which is a
behaviour change and belongs with the inline-`style` candidate recorded in
`ROADMAP.md`. Fixing §4 makes the Google Docs case correct without reading
`style` at all — that is why the two can be separated.

Also not covered: `A-06` (`<li>` with block children), which is RFC 009.

## 6. Required verification

Per RFC 027 Rule 3, state for each whether it ran against the workspace tree or
an installed artifact.

1. The §2 cases, before and after.
2. Mixed inline-and-block children — behaviour stated and tested.
3. Nested emphasis around blocks.
4. **Emphasis around inline content byte-identical to `2.2.1`**, over the whole
   existing corpus. Show it, do not assert it.
5. The RFC 025 block-inside-inline cells un-`#[ignore]`d and passing.
6. `cargo test --workspace --all-features --locked`, fmt, clippy `-D warnings`.
7. Count reconciled.

## 7. Verification input — the corpus, if it has arrived

bekoedit offered a corpus of real clipboard HTML (browsers, Google Docs, Word,
LibreOffice). **If it has arrived by the time you start, use it** — it is the
input that exposed this defect and the input we cannot generate for ourselves.

If it has not arrived, say so in your review request rather than waiting; the §2
cases are sufficient to implement against.

## 8. Prohibited shortcuts

- Do not distribute the emphasis over block children without reporting first.
- Do not read the `style` attribute — out of scope.
- Do not write a second block-tag list.
- Do not let the inline-only case move.

## 9. Known risks

| Risk | If it happens |
|---|---|
| The block predicate disagrees with the arms that emit blocks | Derive it from the same classification. If that classification does not exist in a reusable form, creating it *is* the work — report the shape you chose. |
| html5ever restructures the tree before we see it | Possible for invalid HTML. If `<strong><p>` does not reach the renderer as you expect, report what it does reach as. |
| The inline case shifts by a byte | Stop. That is the common path and a regression there outweighs this fix. |
| Distribution turns out to be right | Report before implementing — see §4. |

## 10. Acceptance checklist

- [ ] `<strong><p>x</p><p>y</p></strong>` → `"x\n\ny\n"`
- [ ] The Google Docs shape → `"one\n\ntwo\n"`
- [ ] `em`/`i` behave the same
- [ ] Nested emphasis around blocks emits neither delimiter
- [ ] Mixed children defined and tested
- [ ] Inline-only emphasis byte-identical to `2.2.1`, demonstrated
- [ ] Block predicate shared, not duplicated
- [ ] RFC 025's block-inside-inline cells pass
- [ ] CHANGELOG entry showing before and after
- [ ] Count reconciles; fmt and clippy clean

## 11. Escalate rather than decide

Stop and raise if: distribution looks right; html5ever's tree differs from the
assumption in §3; the block predicate cannot be shared without restructuring; or
inline-only output moves at all.

## 12. Where this came from

A downstream GUI editor building paste-as-Markdown, not from our own testing. A
56-finding external audit missed it, and RFC 025's composition matrix as
originally specified would have missed it too — every cell put an inline
construct inside a container, and this is a block inside an inline.

Worth knowing while you work on it: **the matrix had a direction, and the defect
was on the other side of it.** If you notice a third direction we are not
testing, that is worth more than this fix.
