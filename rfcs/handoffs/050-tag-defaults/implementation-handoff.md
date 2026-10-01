# Developer Handoff — RFC 050, tag defaults and computed style

**Authorised.** Owner, 2026-10-01 — RFC 050 accepted, **`3.1.1` authorised** in the same decision.
**RFC.** `rfcs/accepted/050-tag-defaults-and-computed-style.md` — read §1 and §3 before any code.
**Baseline.** `origin/main`, eight workflows green, 648 Rust / 60 Node / 25 loader / 93 Python.
**Scope.** `src/renderer.rs` (`own_emphasis`), `docs/src/api/options.md`, tests. **Implementation
only — the version bump, CHANGELOG and RFC move are a separate prep slice.**

---

## 1. The whole defect in one function

`src/renderer.rs:227`:

```rust
let tag_default = matches!(
    (tag, class),
    ("b" | "strong", EmphasisClass::Bold) | ("i" | "em", EmphasisClass::Italic)
);
```

Four tags. Every **other** element whose default rendering is bold or italic falls through to
`Some(true) if emphasis_from_style => Some(true)` and **gains emphasis the source never meant.**

**Note the order of the match arms carefully.** Widening `tag_default` alone does **not** fix this —
the `Some(true) if emphasis_from_style` arm is tested *before* the `_ if tag_default` arm, so a
heading with `font-weight: 700` still returns `Some(true)`. The rule itself has to change: **a style
that restates the tag's own default must contribute nothing new.** Decide what `own_emphasis` should
return in that case and why; do not pattern-match your way to the table in §3 without being able to
state the rule in one sentence.

## 2. The sets

- **Bold by default:** `b`, `strong`, `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `th`
- **Italic by default:** `i`, `em`, `cite`, `address`, `var`, `dfn`

`th` is in the bold set **deliberately**, though it is currently unreachable (table cells never
receive emphasis). RFC 050 §5 decides it: today the defect is prevented by an unrelated boundary, and
that must not be what we are relying on. Say in your report that it is untested-by-construction.

## 3. The behaviour table

| the style says | the tag's default | result |
|---|---|---|
| bold | already bold | **nothing added** |
| bold | not bold | adds emphasis — **unchanged from `3.1.0`** |
| not bold | already bold | **removes** it — unchanged, shipped since `3.0.0` |
| nothing | already bold | the tag's own meaning — unchanged |

## 4. The case that must not over-correct 🛑

```html
<h1 style="font-weight:700"><span style="font-weight:700">H</span></h1>
```

**An authored bold inside a heading is real and must survive** — expected `# **H**`. Only the
heading's **own** restated default is ignored. This is the single way this fix can go wrong, so it
gets its own test with that name.

Likewise `<h1><b>H</b></h1>` must keep its `**`, option on or off.

## 5. Criteria

1. Every row of RFC 050 §1's table: **option on produces exactly what option off produces**, for the
   wrong rows and the already-correct rows alike.
2. §4's two cases pass. Named tests.
3. Negation unchanged: `<b style="font-weight:400">`, `<em style="font-style:normal">`,
   `<h2 style="font-weight:400">` (a heading cannot be un-bolded in Markdown — state what it does and
   why that is right).
4. **A realistic WebKitGTK-shaped fixture** — a document where *every* element carries a computed
   `style`, headings and all — converts with no emphasis the source did not mean. Build it from the
   shape bekoedit described (`<h1 style="caret-color: rgb(0, 0, 0); font-weight: 700; …">`), not from
   a minimal reduction.
5. **Option off is byte-identical to `3.1.0`.** `mode_identity`'s goldens untouched.
6. `docs/src/api/options.md` states the rule and lists both sets. The existing boundary text (table
   cells, bare list-item text, `Minimal`) stays accurate.
7. Counts, each with its command. `cargo fmt --check`, `clippy -D warnings`, docs-example gate clean.

## 6. Report, then stop

`.git-exclude/review-request/050-tag-defaults/README.md`, leading with criterion 5. Commit and push
**only this slice** — never a release, never unrelated commits. The architect's local commits ride
along as ancestors; that is expected.
