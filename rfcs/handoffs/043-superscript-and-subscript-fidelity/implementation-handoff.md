# Developer Handoff — RFC 043 · `<sup>` and `<sub>` that silently change the meaning

**RFC.** `rfcs/accepted/043-superscript-and-subscript-fidelity.md` — accepted by the owner, 2026-09-25
**Milestone.** Unassigned; an output change, so a minor. Release vehicle is the owner's call at prep time
**Priority.** P1 for correctness — a quarter of real occurrences convert to a different statement
**Prepared.** 2026-09-25
**Baseline.** `082a3eb` — 583 Rust, 42 Node, 25 loader, 90 Python; seven workflows green
**Order.** §2 first and **measure before adding §3.** The map is strictly-better and low risk; the fallback is the invasive part and should only carry what the map cannot.

---

## 1. What is wrong

```
2<sup>n − 1</sup>    ->  2n − 1      reads as "two times n minus one"
10<sup>−9</sup>      ->  10−9        reads as "ten minus nine"
x<sup><i>n</i></sup> ->  x*n*        an italic n, not an exponent
x<sub>i</sub>        ->  xi
```

Nothing in the output says a value was raised or lowered. This is not lossy — it is **false**, which is the
distinction that separates it from the `<dl>` fallback we deliberately kept.

**Measured over 415 occurrences on four real pages** (Wikipedia *Markdown*, *Exponentiation*, *Scientific
notation*; Rust Book ch03-02): **185 (44%)** are bracketed citation markers and already correct, **128 (30%)**
already map to Unicode, and **102 (24%) are silently wrong.**

Re-fetch the corpus yourself from those four URLs; do not trust a copy.

## 2. Widen the map — do this first 🛑

Every occurrence fixed here gets a **real** superscript, which beats any notation. Symmetric for `<sup>` and
`<sub>`; the current asymmetry is an accident (`<sub>` maps `m`, `a`, `x` but not `i`).

1. **U+2212 MINUS SIGN behaves as `-`.** One character, and it removes the most dangerous rendering in the
   corpus — `10<sup>−9</sup>` is currently `10−9`.
2. **Extend letter coverage** to the Unicode modifier-letter sets. **They are not symmetric and you cannot
   assume they are:** superscript has 25 of 26 (no `q`); subscript has **17** (no `b c d f g q w y z`). So
   `x<sub>y</sub>` can never map and must fall back — expect the fallback to fire more for `<sub>`.
3. **See through emphasis-only content.** `<sup><i>n</i></sup>` is the variable *n* raised, exactly as
   `<sup>n</sup>` is. If the only markup inside is `<i>`/`<em>`/`<b>`/`<strong>`/`<var>`, strip it and map the
   text. A superscript character cannot carry emphasis anyway.

**Keep the all-or-nothing rule.** Today an element maps only if *every* character maps, and that must stay:
a half-mapped `xₘₐₓ`-style run mixed with plain letters would be worse than either outcome. Uppercase mostly
has no forms, so `x<sup>N</sup>` will fall back — that is correct.

**Then measure and report** how many of the 102 the map alone fixes, before writing §3. That number decides
how much work the fallback is actually doing.

## 3. The fallback, for what remains

`^(…)` for `<sup>`, `_(…)` for `<sub>`, **always parenthesised.** One rule, no edge cases, and after §2 it
should be rare.

**Do not mark self-delimiting content.** If the element's **source text** (tags stripped) matches
`^\[.*\]$` or `^\(.*\)$`, leave it exactly as today. This is what keeps all 185 citation markers untouched,
and it is a property of the text, not a guess about meaning. **It is the single most important rule in this
handoff** — a regression here would make 44% of real occurrences worse in exchange for fixing 24%.

**On emphasis and the fallback:** §2.3 strips emphasis *when mapping succeeds*. When it falls back, the
content is ordinary inline content and its emphasis should survive — `x^(*n* + 1)`, not `x^(n + 1)`.

**`^` has no CommonMark meaning. `_` does.** The subscript marker must go through the escaping machinery, and
the result must be **parsed**, not read. `x_(i)` should be safe under the intraword-underscore rule — that is
a conclusion to verify with the harness, not to assume.

**Never inside `<pre>` or code**, where content is verbatim and no marker belongs.

## 4. Not in this slice

- Raw HTML passthrough. RFC 008 §4 and RFC 009 §4.3 both declined it; nothing here reopens that.
- MathML, `<math>`, or general mathematics support.
- Any change to how citation markers render.

## 5. Criteria

1. **The corpus table**, re-measured on the four pages: citations **185/185 unchanged**; the already-mapping
   set **128/128 unchanged**; all **102** accounted for, split between *newly mapped by §2* and *marked by
   §3*, with both counts stated.
2. `10<sup>−9</sup>` → `10⁻⁹`; `x<sup><i>n</i></sup>` → `xⁿ`; `x<sub>i</sub>` → `xᵢ`; `x<sub>max</sub>` →
   `xₘₐₓ` still.
3. `2<sup>n − 1</sup>` → `2^(n − 1)`, and the output **parses** with base and marker in one text run.
4. The subscript marker parses as text in a paragraph, **a table cell, a list item and a heading** — four
   contexts, because `_` is a CommonMark character and the escaping machinery differs by context.
5. No marker inside `<pre>`/code.
6. `<sup>` and `<sub>` symmetric wherever Unicode allows; where it does not, the asymmetry is **stated in the
   code** with the missing letters listed, so the next reader does not think it an oversight.
7. Harness cells for both elements; `docs/src/api/elements.md`'s `<sup>`/`<sub>` row states the mapping rule,
   the fallback and the self-delimiting exception.
8. 583 Rust / 42 Node / 25 loader / 90 Python still pass, plus the new cells; seven workflows green.

## 6. Report back

`.git-exclude/review-request/043-sup-sub-fidelity/README.md`. Lead with the §5.1 table — it is the
deliverable. Include the §2-only measurement taken before §3 existed, so the fallback's share is visible
rather than inferred.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.

**One thing to know rather than act on:** our only known consumer pins `mdka = "=2.5.1"` exactly, so this
will not reach them by upgrading. Telling them is the architect's job, not part of this slice — but it is
why the corpus table matters: it is what the letter will be written from.
