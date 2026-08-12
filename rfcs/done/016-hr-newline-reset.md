# RFC 016 — `<hr>` swallows the newline before following content

**Status.** Implemented (2.1.8)
**Tracks.** Patch `2.1.8`. Correctness defect in shipped output.
**Touches.** `src/renderer.rs` (one line), `tests/characterisation_elements.rs`,
new regression tests.
**Depends on.** Nothing. RFC 005 Slice A found it; the fix is independent.

## Summary

`<hr>` produces no separating newline before whatever follows it, whenever it is
not the first element. The result is not merely ugly — it destroys the
horizontal rule and corrupts the following text. One line, shipped in every 2.x
release.

## Motivation

### The defect

```
"<p>A</p><hr><p>B</p>"       → "A\n\n---B\n"
"<p>A</p><hr><hr><p>B</p>"   → "A\n\n------B\n"
"<h1>T</h1><hr><p>B</p>"     → "# T\n\n---B\n"
```

Correct when `<hr>` is first or last:

```
"<hr><p>B</p>"               → "---\n\nB\n"     ✓
"<p>A</p><hr>"               → "A\n\n---\n"     ✓
```

### Why it is worse than a formatting nit

In CommonMark a thematic break must be a line containing only `-`, `_` or `*`
and whitespace. **`---B` is not a thematic break** — it is a paragraph
containing the literal text `---B`.

So the output loses the horizontal rule *and* mangles the following content into
it. Two elements silently merge into one malformed line. That is data
corruption, not a cosmetic gap.

### Age

The `"hr"` match arm is byte-identical from tag `2.0.0` to `HEAD`. It has shipped
in **all twelve 2.x releases**, 2.0.0 through 2.1.7.

### Why nothing caught it

No test exercised `<hr>` with a following sibling until RFC 005 Slice A built
its characterisation matrix. `tests/block_elements.rs` covers `<hr>` in
isolation, where the bug does not reproduce.

## Root cause

`src/renderer.rs`:

```rust
"hr" => {
    self.begin_block();
    self.emit_pending_prefix();
    self.output.push_str("---");   // ← writes content without updating state
    self.end_block();
}
```

`push_str` writes to the buffer but leaves `newlines_emitted` untouched. Every
other content-emitting arm either resets it explicitly or goes through
`push_raw`, which does it for them.

So `end_block()` → `ensure_newlines(2)` finds `newlines_emitted` still at `2` —
stale, set by `begin_block()` *before* `"---"` was written — concludes the
newlines are already present, and emits nothing.

When `<hr>` is the document's first element, `ensure_newlines`'s
`output.is_empty()` early return never sets `newlines_emitted` at all, so the
stale value does not exist and the bug does not reproduce. That is exactly why
only the "not first" cases are wrong.

## Proposed fix

Use the existing helper written for precisely this:

```rust
-                self.output.push_str("---");
+                self.push_raw("---");
```

`push_raw` writes the string and then, for content with no trailing newlines,
sets `newlines_emitted = 0` and `at_line_start = false` — the state update that
is missing. No new logic; the idiom already exists and is used elsewhere in the
same file.

### Verified, not proposed

Applied and measured at RFC drafting, then reverted:

```
"<p>A</p><hr><p>B</p>"                          → "A\n\n---\n\nB\n"        ✓
"<hr><p>B</p>"                                  → "---\n\nB\n"             ✓ unchanged
"<p>A</p><hr>"                                  → "A\n\n---\n"             ✓ unchanged
"<p>A</p><hr><hr><p>B</p>"                      → "A\n\n---\n\n---\n\nB\n" ✓
"<h1>T</h1><hr><p>B</p>"                        → "# T\n\n---\n\nB\n"      ✓
"<blockquote><p>Q</p><hr><p>R</p></blockquote>" → "> Q\n\n> ---\n\n> R\n"  ✓
"<hr>"                                          → "---\n"                  ✓ unchanged
"<ul><li>a</li></ul><hr><p>B</p>"               → "- a\n\n---\n\nB\n"      ✓
```

Blockquote prefixing and list interaction both behave correctly, which was the
main thing worth checking beyond the simple cases.

**Exactly one existing test fails** under the fix:
`tests/characterisation_elements.rs::void_element_hr`, which deliberately
records the buggy output with a comment naming this root cause. That is RFC 005
Slice A's marking mechanism working as designed — the one test expected to
change, changing.

## Goals

- `<hr>` emits a thematic break separated from surrounding content in all
  positions.
- The characterisation test is updated to the corrected output, not deleted.
- Regression coverage so this cannot silently return.

## Non-goals

- Refactoring other `renderer.rs` arms. Several also write via `push_str` with
  explicit state updates; those are correct as written and are not in scope.
- Auditing for similar state-desync bugs elsewhere. Worth doing, but as its own
  work — see Risks.
- Anything touching `ConversionOptions`. RFC 005 owns that.

## Compatibility

Output changes for input containing `<hr>` followed by other content. That is
the point: the previous output was malformed.

A consumer who somehow depended on `---B` was depending on corruption. Patch
release is appropriate — this restores documented, intended behaviour rather
than changing a designed one.

## Testing

1. Update `void_element_hr` to the corrected output, keeping a comment recording
   that it previously encoded the bug and pointing at this RFC.
2. Add regression tests covering: `<hr>` first, last, mid-document, consecutive
   `<hr><hr>`, inside a blockquote, and after a list. The blockquote case
   matters most — it exercises the deferred-prefix path.
3. Full suite green.

## Acceptance criteria

1. All eight cases in §Verified produce the stated output.
2. `void_element_hr` updated, not removed, with a comment pointing here.
3. Regression tests cover all six positions in §Testing.
4. `cargo test --workspace --locked`, `cargo fmt --check`, and
   `cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean.
5. No `renderer.rs` change beyond the single line.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Other arms have the same state-desync defect | More corruption undiscovered | Out of scope here, but **worth a follow-up audit**: any arm writing via `output.push_str` without a subsequent state update is suspect. Recorded rather than fixed, so this RFC stays reviewable. |
| The fix changes output in a case not tested | Silent regression | Eight cases verified pre-emptively; six more required as regression tests |
| Consumers depended on the old output | Complaints | The old output is malformed CommonMark. Documented in `CHANGELOG.md`. |

## Alternatives considered

| Option | Assessment |
|---|---|
| **Set `newlines_emitted = 0` explicitly after `push_str`** | Works, but duplicates what `push_raw` exists to do. The helper is the idiom; use it. |
| **Fix in `end_block()`** | Wrong layer. The defect is that `"hr"` fails to record what it wrote, not that `end_block` misreads correct state. |
| **Wait for `2.2.0`** | Rejected by the project owner. Twelve releases of corrupted output on a common element justifies a patch. |
