# Developer Handoff — RFC 049 · inline `style` emphasis

**RFC.** `rfcs/accepted/049-inline-style-emphasis.md` — accepted by the owner, 2026-09-30
**Milestone.** Unassigned. Additive and opt-in; a minor.
**Priority.** P1 — bekoedit's top gap since 2026-09-24, and an asymmetry in our own code.
**Prepared.** 2026-09-30
**Baseline.** `8517526` — 629 Rust (`cargo test --workspace`), 59 Node, 25 loader, 91 Python; eight workflows green
**Scope.** `src/utils.rs`, `src/renderer.rs`, `src/options.rs`, the three bindings, the CLI, `docs/`. **No other conversion behaviour.**

---

## 1. The shape of the work

`emphasis_negated_by_style` already parses `font-weight` and already has the ≥600 threshold. It is used
**only to remove** emphasis from a `<b>`. The same declaration on a `<span>` is ignored.

**Replace it with one rule, do not add a second function.** Two functions of opposite polarity reading the
same property is how the asymmetry arose.

The rule, asked of any element: **what does this element's inline `style` say about emphasis?** Three
answers — *bold* / *not bold* / *says nothing* — and independently the same for italic. The tag then
combines with it:

| | element declares | result |
|---|---|---|
| `<b>`, `<strong>` | nothing | bold (today) |
| `<b>` | not bold | **not** bold (today — must not regress) |
| `<span>`, or any element | bold | **bold (new)** |
| any element | nothing | the tag's own meaning (today) |

**`font-weight`:** `bold`, `bolder`, or a number ≥ 600 → bold. `normal`, `lighter`, or ≤ 500 → not bold.
**`font-style`:** `italic` or `oblique` → italic. `normal` → not italic.
Anything unparseable → *says nothing*, never an error.

## 2. Cases I have decided, so you do not have to 🛑

These are architect decisions, not open questions. **Write a harness cell for each**, in all modes and both
readings.

| Input | Output | Why |
|---|---|---|
| `<b><span style="font-weight:400">x</span></b>` | `x` | The inner declaration wins for its own content, as CSS renders it |
| `<b style="font-weight:400"><span style="font-weight:700">x</span></b>` | `**x**` | Un-bolded then re-bolded |
| `<span style="font-weight:700">a<span style="font-weight:400">b</span></span>` | `**a**b` | Emphasis ends where the declaration is overridden |
| `<span style="font-weight:700"><b>x</b></span>` | `**x**` | **One level, not two.** Reuse RFC 037's existing collapse rule — do not write a second one |
| `<span style="font-weight:700;font-style:italic">x</span>` | whatever `<b><i>x</i></b>` produces today | **Reuse the existing bold+italic emission**, including RFC 037's addendum and RFC 044's fix. Invent nothing |
| `<em style="font-style:normal">x</em>` | `x` | Today's negation, preserved |
| `<div style="font-weight:700"><p>a</p><p>b</p></div>` | `**a**` / `**b**` | **Any element, not only inline ones.** A browser renders that bold, so faithfulness says we do too. It is opt-in, and this is the deliberate choice |

**On that last row:** it means a `style` on a container applies to its subtree, exactly as `<b>` does.
That is the price of "read the element", and it is the behaviour the HTML describes. If you find a case
where it produces something indefensible, **report it rather than special-casing** — a carve-out would be
the start of a CSS engine.

## 3. The option

**`emphasis_from_style`**, boolean, default **off**, on all four surfaces with that one name
(`emphasisFromStyle` in Node, `--emphasis-from-style` on the CLI). The owner chose the name: it names the
outcome, not the mechanism.

**It must be able to affect output** — that is the test `3.0.0` applied when it removed six options that
could not. This one passes it.

## 4. What must not move 🛑

**With the option off, output is byte-identical to `3.0.0`** over the `mode_identity` corpus, against the
existing P3 goldens, **untouched**. That is what proves §1's refactor did not damage the negation path.

**If a golden needs changing, stop and report.** Nothing in this slice may change default output.

And test the negation path **with the option both on and off** — it is today's behaviour and it is the
thing most likely to be broken by its own mirror.

## 5. Documentation — the part that is not code

RFC 049 §2 is a **principle to publish**, not just to implement:

> **mdka reads the element, not the environment.** An inline `style` attribute is part of the element it
> appears on. A `class` is a reference to a stylesheet we were never given.

Say plainly, on the options page and wherever emphasis is described:

- `<span style="font-weight:700">` works;
- **`<span class="c7">` never will**, whatever the stylesheet says — we do not see stylesheets, and this is
  permanent, not unimplemented;
- `<style>` blocks and linked stylesheets are out of scope for the same reason.

**A user with a CMS export needs to learn this from the page, not from an empty result.**

## 6. Criteria

1. One rule; `emphasis_negated_by_style` is gone, not joined by a sibling.
2. Every row of §2, as harness cells, all modes, both readings.
3. The value grammar tested: `bold`, `bolder`, `600`, `700`, `900`, `normal`, `lighter`, `400`, `500`,
   `italic`, `oblique`, absent, malformed, and a declaration among others (`color:red;font-weight:700`).
4. **Option off → byte-identical to `3.0.0`**, P3 goldens untouched.
5. The negation path passes with the option on **and** off.
6. `emphasis_from_style` on all four surfaces; the CLI's `--help` lists it.
7. §5's principle documented, including the permanent `class` limitation.
8. Counts with their commands; eight workflows green.

## 7. Not in this slice

- `text-decoration`, `font-family`, or any other property. RFC 049 §3 names them out and says why.
- Any change to the default. RFC 049 §5: the default question is decided later, on measured prevalence.
- Telling bekoedit. Correspondence is the owner's, and the letter comes after a release, not after a merge.

## 8. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging and releasing are
not yours. **Name who approved anything you push.** `git commit -F <file> -- <explicit paths>`.

Report to `.git-exclude/review-request/049-inline-style-emphasis/README.md`, leading with §6.4 — that
default output did not move — and then §2's table.
