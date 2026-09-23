# Developer Handoff — `2.4.2` · An option documented as inert corrupts tables

**Source.** `.git-exclude/reviewed/2.4.1-documentation-audit/README.md` (D-1, D-2) and the decision request
`.git-exclude/review-request/unwrap-wrappers-defect/README.md`
**Authorised.** Owner, 2026-09-24 — cut a `2.4.2`, on its own, not bundled with RFC 040
**Milestone.** M4 · `2.4.2` (patch)
**Priority.** P1 — corrupt output in two modes that are on by default
**Prepared.** 2026-09-24
**Baseline.** `89c3d1d` — **570 Rust**, 42 Node, 90 Python; seven workflows green
**Scope.** §1 is a three-line code fix. §2 is the larger and more important half. Nothing else.

---

## 0. What this release is

`unwrap_unknown_wrappers` is published as having **no effect**, in `mdka --help`, in
`docs/src/getting-started/usage-cli.md` and in `docs/src/api/options.md`. It has an effect: inside a table
cell it breaks the row. It is **`true` by default in `Semantic` and `Minimal`**, so users who set nothing get
it, and `Minimal` is the mode we document for LLM preprocessing — bulk work nobody reads first.

**Do not change the documentation to match the behaviour.** The documentation is right and the code is
wrong, exactly as in `2.4.1` §1. Repairing the code makes three published statements true at once and needs
no doc edit.

## 1. 🛑 The fix

Against released `2.4.1`:

```
input     <table><tr><th>H</th></tr><tr><td><div>a</div><div>b</div></td></tr></table>

expected  | H |            actual (Semantic, Minimal, or --unwrap-wrappers)   | H |
          | --- |                                                             | --- |
          | a<br>b |                                                          | a
                                                                              
                                                                              b |
```

Parsed with `ENABLE_TABLES` the second is `Table(TableHead(TableCell("H")) TableRow(TableCell("a")))`
followed by `para("b |")`: the cell loses `b`, and `b |` renders as body text with a literal pipe.

**Where.** `src/renderer.rs`, `begin_unwrapped_separator` and `end_unwrapped_separator`.

Both guard `in_pre` and nothing else. Every other block boundary guards the cell case **first**:

```rust
fn enter_block(...) {
    if self.in_table_cell() { self.enter_cell_block(...); return; }   // <- this
    if self.in_pre { ... }
    ...
}
fn leave_block(...) {
    if self.in_table_cell() { self.leave_cell_block(block); return; } // <- and this
    ...
}
```

`begin_unwrapped_separator`'s own doc comment says it *"Reuses `begin_block`/`end_block` exactly — the same
path any other block already takes"*. **That sentence is the bug.** The path any other block takes begins
with the cell check; `begin_block` is what is left *after* it. The comment describes the intent correctly and
the code implements something narrower.

**What to do.** Give both functions the same cell branch the others have, ordered the same way — cell check
before the `in_pre` check:

- `begin_unwrapped_separator`: when `in_table_cell()`, emit the cell's separator via `cell_block_separator()`
  and return. Mirror `enter_cell_block`'s `cell_pre.is_some()` guard — inside a cell's own flattened `<pre>`
  a wrapper contributes nothing, for the same reason the outer `in_pre` guard exists.
- `end_unwrapped_separator`: when `in_table_cell()`, return without writing. `leave_cell_block` writes no
  separator on the way out either; the `<br>` belongs to the *next* block's entry. Confirm this against
  `leave_cell_block` rather than taking my word for it.

Update the doc comment so it stops asserting the thing that was false.

**Do not** fix this by setting `unwrap_unknown_wrappers: false` in `Semantic`. It looks smaller and it is
wrong: `Minimal` keeps the flag and stays broken, and anyone passing `--unwrap-wrappers` explicitly still
gets corruption.

## 2. 🛑 The invariants — this is why it shipped, and the larger half of the slice

`2.4.1` added 22 fixtures and its release notes claim they cover *"every wrapper tag, nesting, containers,
table cells and shell elements"*. Table cells are covered by exactly two:

```
<td><div id="a">c</div></td>
<td><span id="a">c</span></td>
```

**Every fixture puts one child in the cell. The defect needs two siblings.** Measured:

| Shape | Balanced vs Semantic |
|---|---|
| `<td><div id="a">c</div></td>` — the shipped fixture | agree |
| `<td><div><p>a</p><p>b</p></div></td>` | agree |
| `<td><div>a</div><div>b</div></td>` | **diverge** |
| `<td><div>a</div><p>b</p></td>` | **diverge** |

Those fixtures were written to prove an `id` property and they prove it. They are blind to everything else
about the same cells, and mode identity is not an `id` property. Adding a 23rd fixture would repeat the
mistake. **Add the two properties nothing currently asserts**, and run them over a real corpus, not a
hand-written list:

**P1 — an option documented as inert is inert.** For each of `preserve_classes`, `preserve_data_attrs`,
`preserve_aria_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs` and `unwrap_unknown_wrappers`:
flipping that field alone, with everything else held at each mode's defaults, is **byte-identical**, for
**every one of the five modes**, over the whole corpus.

**P2 — the four modes agree.** `Balanced` ≡ `Strict` ≡ `Semantic` ≡ `Preserve`, byte-identical, over the
whole corpus. `Minimal` is excluded: it is documented as genuinely distinct and is.

**Corpus.** Use `tests/output_validity/corpus/` **plus** cases that exercise a wrapper inside a table cell,
since the existing nine files contain none — verified: they produce zero Balanced/Semantic divergence today,
which is why they caught nothing. At minimum: two sibling wrappers in `<td>` and in `<th>`; a wrapper beside
a `<p>`; wrappers in `<li>` and `<blockquote>` (which agree today and must keep agreeing); a wrapper inside a
cell's `<pre>`.

**Both properties must fail before the §1 fix and pass after.** Demonstrate that — a property that passes
before the fix is testing the wrong thing.

## 3. Documentation and the changelog

No behavioural documentation changes. Two things do:

1. **`CHANGELOG.md`, the `2.4.2` entry** must state the correction plainly: `2.4.1` said *"An unwrapped
   wrapper now keeps the anchor, so the four modes are identical as documented"*, and the four modes were not
   identical. `create-release.yaml` publishes the release body as a link to `blob/<tag>/CHANGELOG.md`, frozen
   at the tag, so the 2.4.1 release page will keep showing that sentence forever. A forward correction is the
   only mechanism there is.
2. **Annotate the `2.4.1` bullet in `CHANGELOG.md` on `main`** with a pointer forward to `2.4.2`. Annotate —
   do not rewrite the original claim. `main`'s CHANGELOG is the copy most people read.

Two trivia found by the same audit, fold them in here rather than leaving them:

- `node/README.md`: link text says `/.github/workflows/release-npm.yaml`, the target is
  `release-executable.yaml`. Wrong file.
- `docs/src/design/architecture.md`: calls `examples/` the "Allocation measurement tool"; it holds four
  files, two of them benchmark helpers (`quick_bench.rs`, `quick_compare.rs`).

## 4. Explicitly not in this slice

- **The surface question.** That 6 of 8 options cannot affect output, and that `Strict`/`Preserve` differ
  from `Balanced` only in fields inert forever, is real and is going to `3.0` as its own RFC. **Do not
  deprecate `unwrap_unknown_wrappers`, do not remove any option, do not touch any mode's defaults, do not
  change what `--help` lists.** A patch changes no API.
- RFC 040 (npm platforms) ships separately.

## 5. Criteria for the slice

1. The §1 input produces `| a<br>b |` in **all five modes** and with `--unwrap-wrappers` in any mode, and its
   parse tree matches `Balanced`'s. Check the **tree**, not the presence of a table — a truncated table still
   parses as a table, which is how I nearly mis-scored this.
2. P1 and P2 both exist, both fail on `89c3d1d`, both pass after.
3. Existing tests still pass: **570 Rust**, 42 Node, 90 Python. The 22 `ID_WRAPPER_SHAPES` fixtures stay —
   they are not wrong, only narrow.
4. No change to any public signature, option default, or `--help` output.
5. All seven workflows green on the commit that will be tagged.
6. `CHANGELOG.md` has a dated `## [2.4.2]` section carrying §3's correction.

## 6. Report back

Say which of P1 and P2 failed before the fix and what they printed — that is the evidence the properties are
real. If either passes before the fix, stop and tell me: it means the property is not expressing what §2
describes, and I would rather rewrite it than have it merged green.
