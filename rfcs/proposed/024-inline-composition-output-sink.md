# RFC 024 — Inline composition: route every writer through the output sink

**Status.** Proposed
**Tracks.** M3 · Conversion fidelity → `2.3.0`
**Priority.** P0
**Blocked on.** [RFC 025](./025-output-validity-harness.md) — the harness lands first
**Touches.** `src/renderer.rs`.
**Source.** External audit 2026-08-31 — `A-01` (High), `A-02` (High).
**Prepared.** 2026-08-31

## Summary

The renderer redirects output into a buffer while capturing a link's text. That
redirection is honoured by **one** writer and bypassed by the rest, so any inline
element inside `<a>` escapes the link. Make the redirection structural instead of
conventional.

## The defect, reproduced

```
<a href="/page"><img src="i.png" alt="pic"></a>  →  "![pic](i.png)[](/page)\n"
<a href="/x"><strong>b</strong></a>              →  "****[b](/x)\n"
<a href="/x"><em>i</em></a>                      →  "**[i](/x)\n"
```

The first is CommonMark's own canonical example of a linked image. mdka turns it
into an image followed by an **empty link**, which is not what the document said
and is not recoverable by a reader. The second leaks a literal `****` into prose.

Linked images are ubiquitous — every thumbnail, logo and card in ordinary HTML.

## Root cause — measured, not inferred

`src/renderer.rs` contains **37 direct `self.output.push*` calls**. Exactly one
place — inside `push_raw`, at line 126 — checks `InlineCapture::Link` and
redirects into the link's buffer:

```rust
if let InlineCapture::Link { buf, .. } = &mut self.inline_capture { ... }
```

So the invariant *"while capturing, write to the buffer, not the output"* is
enforced in a single function and silently violated by every arm that writes to
`self.output` directly. The `<img>` and `<strong>` arms are two such arms.

**This is a structural defect, not a set of leaf bugs.** Every future element
handler is one `self.output.push` away from reintroducing it, and nothing in the
type system or the tests objects.

## Design

### Make the sink the only way to write

Introduce a single accessor that returns the current destination:

```rust
fn sink(&mut self) -> &mut String {
    match &mut self.inline_capture {
        InlineCapture::Link { buf, .. } => buf,
        InlineCapture::None => &mut self.output,
    }
}
```

Route **every** writer through it. `self.output` should not be written directly
anywhere in element handling once this lands.

The goal is that adding a new element handler cannot get this wrong without
deliberately reaching around the accessor. Making the field harder to reach by
accident — a narrower module boundary, a wrapper type, or at minimum a comment
at the field declaration plus a grep-able convention — is in scope. **Prefer a
mechanism over a convention**: the current mechanism *is* a convention, and it
failed for both `<img>` and `<strong>`.

### Bookkeeping travels with the sink

`push_raw` also maintains `newlines_emitted`, `at_line_start` and
`last_was_space`. Those must stay correct when writing into a capture buffer —
this is exactly the state whose desync produced RFC 016 and RFC 017. Where
line-oriented state is meaningless inside an inline capture, say so explicitly
rather than leaving it ambiguous.

### `A-02` — `<pre>` without `<code>`

```
<pre>plain</pre><p>After</p>  →  "plain\n```\n\nAfter\n"
```

The closing fence is emitted on leaving `<pre>`, but the opening fence is emitted
by the `<code>` arm — so a `<pre>` with no `<code>` child produces a **closing
fence with nothing opened**. Everything after is swallowed into a code block by
any parser.

Open the fence on entering `<pre>`, not on entering `<code>`, so the two are
emitted by the same element and cannot become unbalanced. `<pre><code>` must stay
byte-identical; it is the overwhelmingly common shape and is currently correct.

Grouped with `A-01` because both are the same class — an invariant split across
two places that can disagree — and both are in the same file.

### Nested links

Once inline elements are captured correctly, `<a>` inside `<a>` becomes
reachable. HTML forbids it and html5ever's tree builder generally flattens it,
but the renderer must not corrupt output if it appears. Define the behaviour and
test it, even if the answer is "inner link contributes its text only".

## Not in scope

Escaping of destinations, titles and text — RFC 010. `A-01` is about *where*
bytes go; `A-04` is about *which* bytes. Separate seams, separate RFCs.

Element coverage (`<li>` block children, blockquote continuity, list indent) —
`A-06`, `A-07`, `A-08`, RFC 009.

## Compatibility

Output changes for any document with an inline element inside a link. Every such
change is from broken output to correct output, so no caller can be depending on
the old form except by having worked around it.

Minor version — `2.3.0` — and a CHANGELOG entry showing before and after, since
anyone diffing generated Markdown will see movement.

## Risks

| Risk | Mitigation |
|---|---|
| Routing 37 call sites misses one | RFC 025's matrix is the detector. A missed site shows as a failing composition cell. |
| Borrow-checker friction — `sink()` borrows `self` mutably | Expected. If it forces restructuring a handler, that is the design working. Raise it if a handler cannot be expressed. |
| Line-state bookkeeping desyncs inside a capture | RFC 016/017 are the precedent. Cover blank-line and fence-adjacent cases explicitly. |
| `<pre><code>` regresses | Byte-identical assertion on the existing fixtures, plus the bare-`<pre>` case. |

## Acceptance criteria

1. `<a href="/page"><img src="i.png" alt="pic"></a>` parses as a link containing
   an image.
2. `<a href="/x"><strong>b</strong></a>` parses as a link whose text is strong
   emphasis. No stray `*` in the output.
3. `<pre>plain</pre><p>After</p>` produces a balanced fence, and `After` is a
   paragraph outside it.
4. `<pre><code>…</code></pre>` output is byte-identical to 2.2.0.
5. No direct `self.output` write remains in element handling; the count is
   reported in the review request.
6. The RFC 025 composition cells for inline-in-link are un-`#[ignore]`d and pass.
7. Nested-link behaviour is defined and tested.
