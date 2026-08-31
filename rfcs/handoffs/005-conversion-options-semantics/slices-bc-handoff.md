# Developer Handoff — RFC 005 Slices B and C

**Governing RFC.** [RFC 005](../../done/005-conversion-options-semantics.md), Option 3 recorded 2026-08-08
**Supersedes.** Nothing. [`implementation-handoff.md`](./implementation-handoff.md) covered Slice A, which is complete and approved.
**Milestone.** M2 · Truth in the API surface
**Prepared.** 2026-08-12

This Handoff directs Slices B and C. It does not redefine RFC 005. If
implementation uncovers a conflict, stop and raise it.

---

## 1. Purpose

Make the eight-field option surface honest. One field gains a real effect; five
are deprecated as documented no-ops; two already work.

## 2. The decision, and what it is not

**Option 3.** `preserve_ids` emits anchors. `preserve_classes`,
`preserve_data_attrs`, `preserve_aria_attrs`, `preserve_unknown_attrs` and
`drop_presentation_attrs` are deprecated as no-ops.

Your Slice A matrix is what settled it: Balanced, Strict and Preserve are
already byte-identical, so deprecating five fields collapses nothing that was
not already collapsed.

**This is not "give up on attribute preservation."** Markdown has no attribute
syntax; some flavours (Pandoc, kramdown) do, and if the project ever wants it
that is a feature RFC designed deliberately. What is being removed is a promise
the format could never keep.

**Nobody's output changes** except where an `id` is present.

## 3. Change scope

| Path | Change |
|---|---|
| `src/options.rs` | Deprecate five fields + the `preserve_aria_attrs` builder; `#[allow(deprecated)]` on `for_mode` |
| `src/renderer.rs` | Emit anchors for `preserve_ids` |
| `cli/src/main.rs` | `#[allow(deprecated)]` at the flag call sites — **attributes only** |
| `node/src/lib.rs` | Same |
| `python/src/lib.rs` | Same |
| `tests/` | Slice C |

### Why bindings are in scope when RFC 005 said they were not

`#[deprecated]` fires where a field is **set**, not only where it is read. Those
five are set in four crates — the `for_mode` presets, and all three bindings.
Under `clippy -D warnings` that fails the workspace build.

So Slice B must add `#[allow(deprecated)]` at those sites or it cannot land
green. **Attributes only.** No signature change, no behaviour change, no
restructuring. RFC 006 still owns binding parity and documentation.

Recorded as a scope amendment in RFC 005.

## 4. Non-change scope

- `drop_interactive_shell`, `unwrap_unknown_wrappers` — they work.
- **Do not remove any field.** Deprecation only; removal is a breaking change and
  no major version is planned.
- `docs/` in its entirety — RFC 006. The option docs will be wrong until then;
  leave them.
- `src/traversal.rs` unless anchor emission genuinely requires it — prefer
  `renderer.rs`.
- Comment handling (`Preserve` mode's unimplemented intent) — separate question.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml`

Still untracked, still not gitignored. Stage by explicit path.

## 5. Slice B1 — `preserve_ids` emits anchors

When `preserve_ids` is true and an element carries a non-empty `id`, emit an
anchor before that element's own output:

```html
<a id="…"></a>
```

### The output must remain valid Markdown

The anchor must not corrupt the element it precedes. `<a id="x"></a>## Install`
is broken — an ATX heading needs `#` at line start.

Required cases and their expected output:

| Input | Expected |
|---|---|
| `<h2 id="install">Install</h2>` | `"<a id=\"install\"></a>\n\n## Install\n"` |
| `<p id="intro">Text</p>` | `"<a id=\"intro\"></a>\n\nText\n"` |
| `<p>a <span id="s">b</span> c</p>` | anchor inline before `b`, paragraph intact |
| `<h2>No id</h2>` | `"## No id\n"` — unchanged |
| `<h2 id="">Empty</h2>` | `"## Empty\n"` — empty `id` emits nothing |
| `preserve_ids = false` | no anchor, whatever the input |

Determine placement mechanics yourself; those outputs are the contract.

### ⚠ Escaping — this is a new HTML injection surface

**Read this before writing the emission.**

This is the engine's first emission of an input-derived value into **HTML
attribute context**. Today `href`, `src`, `alt` and `title` go into Markdown
link and image syntax — not HTML attributes.

An `id` of `x" onload="alert(1)` would, unescaped, produce:

```html
<a id="x" onload="alert(1)"></a>
```

That is an injected attribute in output a downstream renderer may render. mdka
documents that it does not sanitise HTML, but that non-goal covers *passing
through* existing markup — it does not license **constructing** new HTML from
untrusted values.

**Escape the value for attribute context: `&` → `&amp;`, `"` → `&quot;`.**
Order matters — `&` first, or you double-escape.

Required cases:

| Input `id` | Emitted |
|---|---|
| `x" onload="alert(1)` | `<a id="x&quot; onload=&quot;alert(1)"></a>` |
| `a&b` | `<a id="a&amp;b"></a>` |
| `plain-id` | `<a id="plain-id"></a>` |

If you conclude a different escaping set is correct, **stop and report** rather
than choosing. Getting this wrong is a vulnerability, not a formatting nit.

## 6. Slice B2 — deprecate five fields

```rust
#[deprecated(
    since = "2.2.0",
    note = "no effect: Markdown has no attribute syntax. See RFC 005."
)]
```

on `preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`,
`preserve_unknown_attrs`, `drop_presentation_attrs`, and on the
`preserve_aria_attrs` **builder method**.

Update each field's doc comment to say plainly that it has no effect and why.
The current comments describe behaviour that has never existed.

Then `#[allow(deprecated)]` at every internal set site (§3) — narrowest scope
that compiles, not blanket crate-level.

**`preserve_ids` is not deprecated.** It gains a real effect in B1.

## 7. Slice C — proof, restated

RFC 005's original criterion was "one test per field demonstrating it changes
output." Under Option 3 five fields deliberately do **not** change output, so:

| Field | Test proves |
|---|---|
| `preserve_ids` | Toggling it **changes** output — anchor present vs absent |
| `drop_interactive_shell` | Still changes output |
| `unwrap_unknown_wrappers` | Still changes output — use a **bare-sibling-text** fixture, per your own Slice A finding that block-element fixtures cannot discriminate it |
| The five deprecated | Toggling **does not** change output, in all five modes |

The last row matters as much as the first. Locking the no-op in means a future
accidental change is visible as a test failure rather than a surprise.

### Slice A's characterisation matrix will need updating

`preserve_ids` cases currently assert no anchor. Those assertions become wrong.

**Update them; do not delete them.** Keep a comment recording that they
previously asserted the pre-anchor behaviour and naming RFC 005 — the same
treatment `void_element_hr` got under RFC 016, and for the same reason: a bare
corrected assertion teaches a future reader nothing.

Report which characterisation tests changed and why.

## 8. Required verification

```
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Baseline 113.** Expect 113 plus your additions, with some Slice A cases
changed per §7.

Clippy is the one to watch: if any `#[allow(deprecated)]` is missing, the
workspace fails. That failure is the mechanism working — add the attribute at
that site, do not widen an existing allow to cover it.

Assert only on `html_to_markdown` / `html_to_markdown_with` output. No internal
functions, no intermediate representations — the mistake the deleted
preprocessor's suite made.

## 9. Prohibited shortcuts

- Do not remove any field.
- Do not emit an unescaped `id`.
- Do not blanket-allow deprecated at crate level.
- Do not touch `docs/`.
- Do not delete Slice A tests that change — update them with a comment.
- Do not "fix" the binding surfaces beyond adding attributes.

## 10. Known risks

| Risk | If it happens |
|---|---|
| Anchor placement corrupts an element's Markdown | The §5 table is the contract. If an expected output looks wrong, report before adjusting it. |
| A deprecation site was missed | Clippy fails. Add the attribute there. |
| Escaping set turns out to need more than `&` and `"` | Stop and report — security-relevant. |
| Slice A changes look larger than expected | Report the list before proceeding; the anchor change may reach more cases than anticipated. |

## 11. Required evidence

1. All §5 output cases, run by you.
2. All §5 escaping cases, run by you.
3. `git diff src/options.rs` — deprecations and allows only.
4. `git diff cli/ node/ python/` — attributes only, no logic.
5. Test count reconciled against 113 + additions.
6. The list of Slice A tests that changed, with reasons.
7. fmt and clippy clean.

## 12. Acceptance checklist

- [ ] `preserve_ids` emits an escaped anchor; all §5 cases pass
- [ ] Empty `id` emits nothing
- [ ] Escaping cases pass, `&` before `"`
- [ ] Five fields and the `preserve_aria_attrs` builder deprecated with a note
- [ ] Field doc comments corrected
- [ ] `#[allow(deprecated)]` narrowly scoped at each set site
- [ ] No field removed
- [ ] Slice C: one proof per field, including no-op proofs for the five
- [ ] Slice A tests updated with explanatory comments, not deleted
- [ ] Count reconciles; fmt and clippy clean
- [ ] `docs/` untouched; `create-release.yaml` untracked

## 13. Required review-request format

Standard eleven parts. The substance:

4. **The §5 output and escaping cases, run by you**
5. **Which Slice A tests changed, and why**
6. Anything about the escaping decision you were unsure of

## 14. Evidence standard

If a count does not reconcile, say so explicitly.

## 15. Escalate rather than decide

Stop and raise it if: an expected output in §5 seems wrong; escaping needs more
than `&` and `"`; a deprecation cannot be silenced narrowly; or anchor emission
needs `traversal.rs` changes beyond what §4 allows.

## 16. After this lands

RFC 006 — option documentation and binding parity — becomes unblocked, and can
finally describe what the options actually do. It also carries the
`figure`/`figcaption` finding recorded in `ROADMAP.md`.
