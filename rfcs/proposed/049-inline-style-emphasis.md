# RFC 049 — Inline `style` emphasis, and where reading presentation stops

**Status.** Proposed
**Author.** Architect
**Created.** 2026-09-30
**Milestone.** Unassigned. Additive and opt-in; a minor.
**Source.** bekoedit's item 4, their top gap since 2026-09-24 — *every bold word in a Google Docs paste arrives plain*. Owner, 2026-09-30: approved in principle, with *"our library is not a CSS engine but a Markdown converter which should respect the original HTML as input. We had better declare it clearly."*
**Touches.** `src/utils.rs`, `src/renderer.rs`, `src/options.rs`, the bindings, `docs/src/`.

---

## 1. The defect is an asymmetry, not a missing feature

`src/utils.rs` already parses `font-weight` out of a `style` attribute, and already knows the threshold:

```rust
pub(crate) fn emphasis_negated_by_style(tag: &str, style: Option<&str>) -> bool {
    let property = match tag { "b" | "strong" => "font-weight", "i" | "em" => "font-style", … };
    …
    value == "normal" || value.parse::<f64>().is_ok_and(|weight| weight <= 500.0)
}
```

It is used in one direction only — to **remove** emphasis from a `<b>` that CSS has un-bolded (the Google
Docs wrapper case, RFC 028).

So today:

| Input | Output | |
|---|---|---|
| `<b style="font-weight:400">x</b>` | `x` | correct — the style is honoured |
| `<span style="font-weight:700">x</span>` | `x` | **wrong — the identical declaration is ignored** |

**The same property, parsed by the same code, honoured when it subtracts and ignored when it adds.** That
is the defect. The feature is the symmetry.

## 2. The principle, declared

> **mdka reads the element, not the environment.**

An inline `style` attribute **is part of the element it appears on** — it arrived in the document we were
handed, and reading it is reading the input. A `class` is **a reference to a stylesheet we were never
given**; resolving it would mean inventing what we cannot see.

That is the line, and it is principled rather than a convenient stopping point:

- `<span style="font-weight:700">` — **read it.** The document says bold.
- `<span class="c7">` — **cannot be read, ever.** The document says "look elsewhere", and there is no
  elsewhere.
- `<style>` blocks and linked stylesheets — **out of scope permanently**, same reason.

**This must be stated in the documentation**, not just implemented. It tells a user with a Google Docs
paste why bold works, and a user with a CMS export why it does not, without either of them reading the
source.

## 3. Scope: `font-weight` and `font-style`, and nothing else

**Exactly the two properties the negation path already understands.** Symmetry is the principle; coverage
is not.

| Property | Bold/italic when | Already parsed? |
|---|---|---|
| `font-weight` | `bold`, `bolder`, or a number ≥ 600 | **yes**, threshold at ≤500 |
| `font-style` | `italic` or `oblique` | **yes**, negation on `normal` |

**Not in scope, and the RFC should say why:** `text-decoration: line-through` (→ `~~`),
`font-family: monospace` (→ code) and everything else. Each is a plausible next ask, and each would be a
new capability rather than the completion of an existing one. If demand appears they get their own RFC and
their own evidence.

## 4. One rule, not two opposite special cases 🛑

The implementation must **replace** `emphasis_negated_by_style`, not sit beside it. Two functions with
opposite polarity reading the same property is how the asymmetry happened in the first place.

One question, asked of any element: *does this element's inline style make it bold, not bold, or say
nothing?* Three answers, one code path, and the tag then decides what to do with it:

- a `<b>` whose style says *not bold* → no emphasis (today's behaviour, preserved);
- a `<span>` whose style says *bold* → emphasis (the new behaviour);
- anything saying nothing → the tag's own default (today's behaviour, preserved).

**Byte-identical output when the option is off** is the criterion that proves the refactor did not change
the negation path.

## 5. Opt-in first, and measure before considering a default

**Opt-in.** bekoedit has said opt-in is acceptable, and turning it on by default changes output for
everyone — which is the thing this project's only known consumer pins an exact version to avoid.

**But the principle in §2 argues for it eventually being the default**, because ignoring a declaration the
document makes is a fidelity gap, not a feature. **Do not decide that here.** RFC 043's method applies:
measure how often the shape occurs in real HTML, and how often turning it on would change output, before
proposing a default. That measurement belongs to a later slice with evidence, not to this RFC's opinion.

## 6. Acceptance criteria

1. `<span style="font-weight:700">x</span>` → `**x**`; `<span style="font-style:italic">x</span>` → `*x*`,
   with the option on.
2. **With the option off, output is byte-identical to `3.0.0`** over the `mode_identity` corpus — the
   P3-golden discipline, and the proof that §4's refactor preserved the negation path.
3. `<b style="font-weight:400">x</b>` → `x` still, with the option **on and off**. The negation path is
   not a casualty of adding its mirror.
4. The full value grammar of both properties is covered and tested: `bold`, `bolder`, `600`, `700`, `900`,
   `normal`, `400`, `500`, `italic`, `oblique`, absent, malformed.
5. **Nested and conflicting cases have stated answers**, chosen deliberately and tested — a bold `<span>`
   inside a `<b>`, a `<span style="font-weight:700">` inside a `<span style="font-weight:400">`.
6. The option exists in all four surfaces with one name, and **can affect output** — the test `3.0.0`
   applied when it removed six options that could not.
7. **The §2 principle is documented**, including that `class` and stylesheets are permanently out of reach.
8. Existing counts pass, each with its command; workflows green.

## 7. Open questions for the owner

1. **The option's name.** `read_inline_style` says what it does; `emphasis_from_style` says what it is for.
   My preference is the second — the surface should name the outcome, not the mechanism.
2. **Whether §5's later default question should be opened at all**, or whether opt-in is the permanent
   answer. My recommendation is to leave it open and decide on evidence.
