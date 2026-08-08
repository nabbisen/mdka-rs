# RFC 017 — `<pre>` closing fence swallows following content

**Status.** Proposed
**Tracks.** Patch `2.1.8`, alongside RFC 016. Correctness defect in shipped output.
**Touches.** `src/renderer.rs` (one line), new regression tests.
**Depends on.** Nothing. RFC 016's implementer found it; the fix is independent
but belongs in the same release.

## Summary

The closing ``` ``` ``` fence of a code block is written without updating the
renderer's newline state, so the block separator that should follow it is
suppressed. When the `<pre>` content ends in two or more newlines, the fence and
the next element merge onto one line — which is not a valid closing fence, so
**the code block never closes and the rest of the document is swallowed into
it.**

Same root cause, same file, same one-line shape, and the same twelve-release age
as RFC 016.

## Motivation

### The defect

Measured on `main` at `6f8772e`:

| Input | Output | Verdict |
|---|---|---|
| `<pre><code>fn main(){}</code></pre><p>B</p>` | `"```\nfn main(){}\n```\n\nB\n"` | correct |
| `<pre><code>fn main(){}\n</code></pre><p>B</p>` | `"```\nfn main(){}\n```\nB\n"` | one newline short |
| `<pre><code>x\n\n</code></pre><p>B</p>` | `` "```\nx\n\n```B\n" `` | **corrupt** |
| `<pre><code>x\n</code></pre><hr><p>B</p>` | `"```\nx\n```\n---\n\nB\n"` | one newline short |

### Why the third case is severe

CommonMark requires a closing code fence to be a line containing only backticks
and optional whitespace. ```` ```B ```` is not that — it is ordinary content
*inside* the block.

So the fenced block never terminates. Everything after it is rendered as code, to
the end of the document. A single `<pre>` whose content happens to end in a blank
line can consume the entire remainder of a converted page.

RFC 016's `<hr>` defect corrupted one line. This one can corrupt everything that
follows.

### Why it stayed hidden

Severity scales with how many trailing newlines the `<pre>` content carries:

- none → unaffected
- one → a missing blank line, cosmetic
- two or more → block never closes

Most hand-written fixtures use `<pre><code>x</code></pre>` with no trailing
newline, which is exactly the unaffected case. Real-world HTML — where the
source is indented and the closing tag sits on its own line — routinely produces
trailing newlines.

`tests/block_elements.rs` covers code blocks, and every case there is the
no-trailing-newline shape.

### Age

The `"pre"` arm in `leave_element` is byte-identical from tag `2.0.0` to `HEAD`.
Shipped in all twelve 2.x releases, 2.0.0 through 2.1.7.

### How it was found

RFC 016's implementer, having fixed the `<hr>` arm, checked **every** other arm
in `enter_element` and `leave_element` against the invariant `push_raw` encodes,
rather than stopping at the assigned defect. This was the one other instance.

They reported it as an unverified candidate rather than a confirmed defect. It
was verified at RFC 016's review.

## Root cause

`src/renderer.rs`, `leave_element`:

```rust
"pre" => {
    if !self.output.ends_with('\n') {
        self.output.push('\n');
    }
    self.output.push_str("```");   // ← writes content without updating state
    self.in_pre = false;
    self.pre_lang = None;
    self.end_block();
}
```

`push_str` writes to the buffer and leaves `newlines_emitted` untouched. Whatever
value the preceding `push_raw` of the code content left behind survives — and for
content ending in newlines, that value is non-zero.

`end_block()` → `ensure_newlines(2)` then reads that stale count, believes some
newlines are already present, and emits fewer than it should. With two trailing
newlines it emits none, leaving the fence and the following content adjacent.

This is precisely the mechanism RFC 016 documented for `<hr>`, in the one place
it also occurs.

## Proposed fix

```rust
-                self.output.push_str("```");
+                self.push_raw("```");
```

`push_raw` writes the string and, for content with no trailing newlines, sets
`newlines_emitted = 0` and `at_line_start = false` — the update that is missing.

### Verified, not proposed

Applied and measured at RFC 016's review, then reverted:

| Input | After fix |
|---|---|
| `<pre><code>fn main(){}</code></pre><p>B</p>` | `"```\nfn main(){}\n```\n\nB\n"` unchanged |
| `<pre><code>fn main(){}\n</code></pre><p>B</p>` | `"```\nfn main(){}\n```\n\nB\n"` **fixed** |
| `<pre><code>x\n\n</code></pre><p>B</p>` | `"```\nx\n\n```\n\nB\n"` **fixed** |
| `<pre><code>x\n</code></pre><hr><p>B</p>` | `"```\nx\n```\n\n---\n\nB\n"` **fixed** |
| `<pre><code>x</code></pre>` | `"```\nx\n```\n"` unchanged |
| `<pre><code class="language-rust">fn f(){}\n</code></pre><p>B</p>` | `"```rust\nfn f(){}\n```\n\nB\n"` unchanged |

The language-class case matters: it confirms the fix does not disturb the
`extract_code_lang` path, which writes the opening fence through a different
branch.

## Goals

- A code block's closing fence is always followed by a proper block separator.
- Regression coverage across the trailing-newline range that determines severity.

## Non-goals

- Refactoring other `renderer.rs` arms. RFC 016's implementer checked every arm
  in `enter_element` and `leave_element` individually; this is the only remaining
  instance. Nothing further to sweep.
- Changing how `<pre>` content itself is emitted, or the opening fence.
- Normalising trailing newlines inside code content. The content is preserved
  verbatim by design; this RFC changes only the separator after the fence.

## Compatibility

Output changes for `<pre>` whose content ends in one or more newlines followed by
further content. That is the point — the previous output was malformed, and in
the two-newline case was not parseable as intended at all.

Patch release, alongside RFC 016.

## Testing

Regression tests spanning the severity range, since the defect is invisible at
zero trailing newlines:

| Case | Why |
|---|---|
| No trailing newline, followed by content | Guards the already-correct case |
| One trailing newline, followed by content | The cosmetic failure |
| Two trailing newlines, followed by content | **The corruption case** |
| `<pre>` followed by `<hr>` | Cross-check against RFC 016's fix |
| `<pre>` alone | Guards the terminal case |
| `language-` class with trailing newline | Guards the opening-fence path |

## Acceptance criteria

1. All six cases in §Verified produce the stated output.
2. Regression tests cover all six positions in §Testing.
3. `cargo test --workspace --locked`, `cargo fmt --check`, and
   `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
4. No `renderer.rs` change beyond the single line.
5. Test count reconciles against 107 plus additions.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| An existing code-block test encodes the buggy output | Unexpected failure | None found — `tests/block_elements.rs` uses the unaffected no-trailing-newline shape. If one does fail, stop and report; it may be a second characterisation case worth updating rather than a regression. |
| The fix disturbs the language-class path | Fenced language lost | Covered by a §Verified case and a required regression test |
| A third instance of this defect class exists | More corruption | RFC 016's implementer checked every arm; recorded as complete rather than assumed |

## Alternatives considered

| Option | Assessment |
|---|---|
| **Explicit `newlines_emitted = 0` after `push_str`** | Works, but duplicates what `push_raw` exists for. RFC 016 set the precedent; matching it keeps the two fixes legible as one class. |
| **Fold into RFC 016** | Rejected. RFC 016 was approved and implemented before this was confirmed; extending an approved RFC after the fact blurs what was reviewed. A separate RFC in the same release is cleaner. |
| **Defer to `2.1.9`** | Rejected. Same file, same root cause, same age as RFC 016 — cutting a patch for one of a matched pair and leaving the more severe one behind is not defensible once documented. |
