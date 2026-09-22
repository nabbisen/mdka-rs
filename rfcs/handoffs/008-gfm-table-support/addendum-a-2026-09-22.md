# Addendum — RFC 008 slice `008a`, 2026-09-22

**Amends.** `slice-a-handoff.md` (frozen; this addendum is the only correct channel)
**Issued after.** the `008a` review, `.git-exclude/reviewed/008a-table-grid-and-prepass/README.md`
**Status.** The slice is right in every other respect. **One required fix before commit.**

---

## 1. 🛑 Pipe escaping does not reach capture contents — four cell-destroying leaks

Criterion 3 holds for plain text and `<strong>`. It fails wherever a **capture** produces the content:

| Cell content | Parses as |
|---|---|
| `<code>a \| b</code>` | `TableCell("`" "a")` — **"b" is lost** |
| `<a href="/x">a \| b</a>` | `TableCell("[" "a")` — link destroyed |
| `<a href="/a\|b">link</a>` | `TableCell("[" "link" "]" "(/a")` |
| `<img alt="a \| b">` | destroyed |

The code-span capture writes verbatim by RFC 024's design, and link text and destinations go through the
link capture; `Dest::escape_pipes` hangs off `escape::decide`, which neither path reaches.

`pipe_in_cell_is_escaped` passes because it uses plain text — the one shape that already worked. **This is
my miss as much as yours:** criterion 3 said "in cell content" without naming the capture contexts, and the
cell I asked for was the one that already passed.

**All four are expressible — verified before asking:** `` `a \| b` ``, `[a \| b](/x)`, `[link](/a\|b)`,
`![a \| b](i.png)` each parse back into one cell with the pipe intact. **`[link](</a|b>)` does not** — the
angle-bracket destination form breaks the cell, so do not reach for it.

**Where the fix belongs.** The cell is the escaping boundary, so escape at the **cell boundary** — everything
flushed into a `Capture::Cell`, whatever sub-capture produced it — rather than on the direct text path.
`Capture::Cell` was the right structure; it simply is not reached by nested captures. If that turns out
wrong once you are in the code, say so with what you found.

**Criteria:** all six rows (including the two that already pass) parse back into one cell with the pipe
intact; a cell for each, all five modes; **non-table output byte-identical to `3110e6d`**, since nothing here
may change escaping outside a cell.

## 2. Everything else is approved

The grid algorithm, the pre-pass, the expressible path, the fallback-by-classification, the container-prefix
and `<pre>` proofs, `<caption>` survival, the `<br>` space fix, and the harness extension all hold — I
re-derived each independently. Two notes, neither blocking:

- **The harness exemption is keyed on the expectation variant, not the output.** Any future cell declared
  `tree_by_reading` silently loses CommonMark property checking, table or not. Gating on *"the GFM structure
  contains a `Table`"* would be self-enforcing. Worth doing when someone is next in that file.
- **Your §5 estimate is accepted as written**, including declining `2.4.0`. I am carrying that to the owner
  as *achievable but not promisable*.

## 3. What I am doing next, so you are not blocked on it

The two design questions your §5 raised are mine. An RFC 008 amendment will settle F1's flattening rules and
F3's header-span semantics before `008b` is handed off.

**One measurement already changed my answer**, and it is worth knowing before you read that amendment: across
four span-heavy pages, **all 18 span-using tables have a span in the header**, and 8 have one *only* there.
So treating a header span as a blocker — which is what I would have written from intuition — would send every
span table to the fallback and make F3 worth nothing. The header label will be repeated across the columns it
spanned, the same rule as body cells.
