# Developer Handoff — RFC 024 · Inline composition: the output sink

**Governing RFC.** [RFC 024](../../proposed/024-inline-composition-output-sink.md)
**Milestone.** M3 → `2.3.0`
**Priority.** P0
**Blocked on.** [RFC 025](../025-output-validity-harness/implementation-handoff.md) — the harness must land and record failures first
**Prepared.** 2026-08-31

---

## 1. Purpose

The renderer redirects output into a buffer while capturing link text. That
redirection is honoured by one writer and bypassed by the rest, so any inline
element inside `<a>` escapes the link.

## 2. The defect

```
<a href="/page"><img src="i.png" alt="pic"></a>  →  "![pic](i.png)[](/page)\n"
<a href="/x"><strong>b</strong></a>              →  "****[b](/x)\n"
<a href="/x"><em>i</em></a>                      →  "**[i](/x)\n"
```

The first is CommonMark's canonical linked image. We emit an image followed by an
**empty link** — not what the document said, and not recoverable by a reader.
Linked images are in every thumbnail, logo and card on the web.

## 3. Root cause — measured

`src/renderer.rs` has **37 direct `self.output.push*` calls**. Exactly one place —
inside `push_raw`, around line 126 — checks the capture state:

```rust
if let InlineCapture::Link { buf, .. } = &mut self.inline_capture { ... }
```

The invariant *"while capturing, write to the buffer"* lives inside one function
and is violated by every arm writing to `self.output` directly. `<img>` and
`<strong>` are two such arms.

**This is structural.** Every future element handler is one `self.output.push`
away from reintroducing it.

## 4. Design

### 4.1 One sink

```rust
fn sink(&mut self) -> &mut String {
    match &mut self.inline_capture {
        InlineCapture::Link { buf, .. } => buf,
        InlineCapture::None => &mut self.output,
    }
}
```

Route **every** writer through it. After this lands, no element handler writes
`self.output` directly.

**Prefer a mechanism over a convention.** The current mechanism *is* a
convention, and it failed for two separate elements. Consider a narrower module
boundary or a wrapper type so reaching `self.output` directly is awkward rather
than merely discouraged. If you conclude a convention plus a comment is the
pragmatic ceiling, say why.

### 4.2 Bookkeeping travels with the sink

`push_raw` maintains `newlines_emitted`, `at_line_start`, `last_was_space`. These
must stay correct inside a capture buffer. **This exact state produced RFC 016
and RFC 017** — a stale newline count after a direct `push_str`. Where
line-oriented state is meaningless inside an inline capture, say so explicitly
rather than leaving it ambiguous.

### 4.3 `A-02` — bare `<pre>`

```
<pre>plain</pre><p>After</p>  →  "plain\n```\n\nAfter\n"
```

The closing fence is emitted on leaving `<pre>`; the opening fence by the
`<code>` arm. A `<pre>` without `<code>` therefore emits a **closing fence with
nothing opened**, and any parser swallows the rest of the document.

Open the fence on entering `<pre>`, so both fences come from the same element and
cannot disagree. `<pre><code>` must stay **byte-identical** — it is the common
shape and is currently correct.

### 4.4 Nested links

Once inline capture works, `<a>` inside `<a>` becomes reachable. HTML forbids it
and html5ever generally flattens it, but define and test the behaviour anyway.
"Inner link contributes its text only" is an acceptable answer.

## 5. Scope boundary

Per RFC 027 Rule 2: this covers **where bytes go**. Escaping — *which* bytes —
is RFC 010 (`A-03`, `A-04`, `A-05`, `A-09`, `A-10`, `A-11`). Element coverage
(`<li>` block children, blockquote continuity, list indent — `A-06`, `A-07`,
`A-08`) is RFC 009.

If a fix here would also fix an escaping defect, **stop and check** whether it
belongs to RFC 010 first. Overlapping slices produce merge pain and unclear
ownership.

## 6. Required verification

Per RFC 027 Rule 3, state tree-vs-artifact for each.

1. Each §2 case, before and after.
2. `<pre><code>…</code></pre>` byte-identical to 2.2.0 across the whole corpus.
3. The RFC 025 composition cells for inline-in-link un-`#[ignore]`d and passing.
4. Count of remaining direct `self.output` writes in element handling — should be
   zero; report the number.
5. Nested-link behaviour defined and tested.
6. `cargo test --workspace --locked`, fmt, clippy.
7. Count reconciled.

## 7. Prohibited shortcuts

- Do not fix escaping here.
- Do not change `<pre><code>` output.
- Do not leave a direct `self.output` write in element handling without a written
  reason.
- Do not un-`#[ignore]` a RFC 025 cell you have not actually made pass.

## 8. Known risks

| Risk | If it happens |
|---|---|
| Routing 37 sites misses one | RFC 025's matrix is the detector — a missed site shows as a failing cell. |
| Borrow-checker friction from `sink()` | Expected. If a handler cannot be expressed, raise it; that is a design signal, not a workaround cue. |
| Line-state desync inside a capture | RFC 016/017 are the precedent. Cover blank-line and fence-adjacent cases explicitly. |
| Output changes more widely than expected | Report the diff scope before assuming it is fine. |

## 9. Acceptance checklist

- [ ] `<a href="/page"><img …></a>` parses as a link containing an image
- [ ] `<a href="/x"><strong>b</strong></a>` parses as a link with strong text; no stray `*`
- [ ] `<pre>plain</pre><p>After</p>` produces a balanced fence; `After` is a paragraph outside it
- [ ] `<pre><code>` byte-identical to 2.2.0
- [ ] Zero direct `self.output` writes in element handling; count reported
- [ ] RFC 025 inline-in-link cells pass
- [ ] Nested-link behaviour defined and tested
- [ ] CHANGELOG entry showing before/after output
- [ ] Count reconciles; fmt and clippy clean

## 10. Escalate rather than decide

Stop and raise if: a handler cannot be expressed through `sink()`; a fix here
would also change escaping; `<pre><code>` output moves at all; or the correct
structure for a composition case is ambiguous.
