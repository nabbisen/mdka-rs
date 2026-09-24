# RFC 043 — `<sup>` and `<sub>` that silently change the meaning

**Status.** **Accepted (owner, 2026-09-25)** — handoff issued
**Author.** Architect
**Created.** 2026-09-24
**Milestone.** Unassigned. An output change, so a minor.
**Source.** Owner accepted option B of the `2.4.0` deferred decisions on 2026-09-24 — a visible fallback for unmappable content. This RFC is written after measuring what that would actually do, and the measurement changes the case.
**Touches.** `src/traversal.rs` or wherever `<sup>`/`<sub>` are mapped, `src/renderer/escape.rs` if the marker needs escaping, `docs/src/api/elements.md`, the output-validity harness.
**Supersedes nothing.** RFC 009 §4.3 set the current rule; this narrows it.

---

## 1. Summary

`2<sup>n − 1</sup>` converts to `2n − 1`, which reads as *two times n minus one*. `10<sup>−9</sup>` converts
to `10−9`, which reads as *ten minus nine*. The markup vanishes flush against the base and **nothing in the
output tells the reader a value was raised.**

RFC 009 §4.3 leaves unmappable content *"exactly as it was"*. For citation markers that is right — the
brackets carry the signal. For an exponent there is no signal, and the result is not lossy but **false**.

The owner's decision is a visible fallback. This RFC specifies it, and adds two things the measurement
turned up: **the same defect in `<sub>`**, and **three gaps in the Unicode map** that account for much of the
harm without needing a fallback at all.

## 2. The measurement — and a correction to what I told the owner

**In the decision request I wrote that 99% of 201 real `<sup>` occurrences are citation markers, and that
"that measurement was right".** It was right about its corpus, and its corpus was unrepresentative: it did
not include the technical and mathematical prose mdka exists to convert.

Re-measured on 2026-09-24 over **four real pages** — Wikipedia's *Markdown*, *Exponentiation* and *Scientific
notation*, and Rust Book ch03-02 — **415 occurrences**:

| Bucket | Count | |
|---|---|---|
| Bracketed citation markers | **185 (44%)** | already correct, and must stay untouched |
| Map to Unicode today | **128 (30%)** | already correct |
| **Unmapped and not self-delimiting** | **102 (24%)** | **silently wrong** |

**So the harm is roughly a quarter of real occurrences, not one percent.** The correction matters because the
earlier figure was the main argument for leaving it alone.

What the 102 actually are — 36 distinct renderings, dominated by:

```
20×  *x*     19×  *n*     11×  *y*      5×  −1     3×  1/2
 3×  +∞       3×  −9       3×  *m*      2×  n − 1   2×  −27
```

Two shapes dominate: **italic maths variables** (`<sup><i>n</i></sup>`) and **U+2212 MINUS SIGN**.

## 3. Three gaps that need no fallback

Measured against a `2.5.1` build:

| Input | Output | |
|---|---|---|
| `10<sup>-9</sup>` — ASCII hyphen | `10⁻⁹` | correct |
| `10<sup>−9</sup>` — **U+2212** | `10−9` | **reads as subtraction** |
| `x<sup>n</sup>` | `xⁿ` | correct |
| `x<sup>y</sup>` | `xy` | `ʸ` exists in Unicode; not in our map |
| `x<sup><i>n</i></sup>` | `x*n*` | `n` maps bare, but the emphasis defeats it |

**Closing these is strictly better than marking them** — a real superscript beats a notation. In particular
U+2212 is a one-character addition that removes the most dangerous reading in the corpus.

**`<sub>` has the same defect, and inconsistently.** `H<sub>2</sub>O` → `H₂O` and `x<sub>max</sub>` → `xₘₐₓ`,
but `x<sub>i</sub>` → `xi` and `a<sub>n−1</sub>` → `an−1`. **Any fix that covers only `<sup>` repeats RFC
040's mistake** — fixing one channel and leaving the identical defect one over.

## 4. Proposal

### 4.1 Widen the map first

- Treat **U+2212 MINUS SIGN** as `-` for both `<sup>` and `<sub>`.
- Extend the letter coverage to the Unicode modifier-letter set where a form exists, symmetrically for
  `<sup>` and `<sub>`. `<sub>` already maps `m`, `a`, `x` but not `i`; that asymmetry is an accident.
- **See through emphasis-only content.** `<sup><i>n</i></sup>` is a variable *n* raised, exactly as
  `<sup>n</sup>` is. If the only markup inside is `<i>`/`<em>`/`<b>`/`<strong>`/`<var>`, map the text and drop
  the emphasis — a superscript character cannot carry it anyway.

### 4.2 Then the fallback, for what remains

Emit `^(…)` for `<sup>` and `_(…)` for `<sub>`, **always parenthesised** — one rule, no edge cases, and after
§4.1 it fires rarely.

**Do not mark self-delimiting content.** If the source text of the element is bracketed — `^\[.*\]$` or
`^\(.*\)$` — leave it exactly as today. That is the rule that keeps all 185 citation markers untouched, and
it is a property of the *text*, not a guess about semantics.

`^` has no CommonMark meaning. **`_` does**, so the subscript marker must go through the escaping machinery
and the output must be **parsed**, not eyeballed — `x_(i)` is safe by the intraword rule, but that is a
conclusion to verify, not assume.

### 4.3 Where it must not fire

Inside `<pre>`/code, where content is verbatim text and no marker belongs.

## 5. Not in scope

- Raw HTML passthrough — RFC 008 §4 and RFC 009 §4.3 both declined it and nothing here revisits that.
- MathML, `<math>`, or any general mathematics support.
- Changing citation-marker rendering in any way.

## 6. Acceptance criteria

1. **Measured against the same four pages**, reported as a table: citations unchanged (**185/185**), the
   already-mapping set unchanged (**128/128**), and every one of the **102** either newly mapped by §4.1 or
   marked by §4.2 — with the split between those two stated.
2. `<sup>` and `<sub>` are symmetric: every mapping added to one is added to the other where a Unicode form
   exists, and the fallback fires on both.
3. `10<sup>−9</sup>` → `10⁻⁹`; `x<sup><i>n</i></sup>` → `xⁿ`; `x<sub>i</sub>` → `xᵢ`.
4. `2<sup>n − 1</sup>` → `2^(n − 1)` and the output **parses** with the base and the marker in one text run —
   verified with the harness, not by reading.
5. The subscript marker is checked for emphasis interaction: `x_(i)` parses as text, and in a table cell, a
   list item and a heading.
6. No marker inside `<pre>`/code.
7. The harness gains cells for both elements; `docs/src/api/elements.md`'s `<sup>`/`<sub>` row states the new
   rule, the fallback and the self-delimiting exception.

## 7. Consequence outside the code

**bekoedit pins `mdka = "=2.5.1"` exactly.** They will not receive this by upgrading, and they rank
conversion correctness highly. When it ships they must be **told**, not left to find it — their `Minimal`
mode is affected like any other, though citation markers, their common case, are untouched by §4.2's rule.
