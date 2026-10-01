# RFC 051 — A superscript that only looks raised is not notation

**Status.** Proposed — architect, 2026-10-01
**Author.** Architect
**Created.** 2026-10-01
**Milestone.** Unassigned. Changes output for every user; a minor, not a patch.
**Source.** bekoedit, 2026-10-01, as an explicit suggestion with no date. Owner, 2026-10-01, asked
the question that decides it: *"which is near sense of ordinary? APIs and UI/UX should be easy to
understand with instinct instead of confusion brought."*
**Touches.** The `<sup>`/`<sub>` mapping (RFC 043), `docs/src/`, tests.

---

## 1. What the rule does today

Measured on the published `3.1.0` CLI:

| Input | Output | |
|---|---|---|
| `x<sup>2</sup>`, `10<sup>−9</sup>` | `x²`, `10⁻⁹` | notation — correct |
| `H<sub>2</sub>O` | `H₂O` | notation — correct |
| `1<sup>st</sup> 2<sup>nd</sup>` | `1ˢᵗ 2ⁿᵈ` | **typography** |
| `1<sup>er</sup> 2<sup>e</sup>` (French) | `1ᵉʳ 2ᵉ` | **typography** |
| `1<sup>º</sup> 2<sup>ª</sup>` (Spanish) | **`1^(º) 2^(ª)`** | **typography, rendered differently** |
| `Acme<sup>TM</sup>`, `<sup>®</sup>` | `Acme^(TM)`, `^(®)` | typography |
| `text<sup>[1]</sup>` | `text\[1]` | already flattened — RFC 043 |

## 2. The answer to the owner's question

**Plain `1st` is the ordinary one, and the current rule's real problem is not that `1ˢᵗ` is ugly — it
is that the rule is unpredictable.**

Look at rows 3 and 5. **The same linguistic construct — an ordinal — converts two different ways,
and the reason is neither grammar nor meaning: it is whether Unicode happens to contain a superscript
glyph for those letters.** `1ˢᵗ` but `1^(º)`. No user can predict that, and no explanation of it will
ever feel like anything other than an implementation detail leaking into the output. That is exactly
the confusion the owner's test rejects.

Three further reasons plain wins on instinct:

1. **Nobody writes `1ˢᵗ` by hand in Markdown.** A `.md` file is text a person edits. `1st` is what
   they typed and what they expect to read back.
2. **`1ˢᵗ` is not findable.** A search for `1st` does not match it — bekoedit's point, and it bites
   hardest in an editor, which is their product and ours' main consumer.
3. **Those are modifier letters** (`ˢ` U+02E2, `ᵗ` U+1D57), not text. They sort oddly, they are
   missing from many monospace fonts, and copying them out of Markdown into anywhere else carries
   the oddity along.

## 3. The principle, which is already ours

RFC 043 flattens citation markers — `text<sup>[1]</sup>` → `text[1]` — and that was right for a
reason it never wrote down:

> **A superscript is *notation* when flattening it changes what it means.** `10⁻⁹` → `10-9` says
> something different; `x²` → `x2` says something different. **It is *typography* when flattening
> changes nothing.** `1st` is `1st`. `Acme™` is `Acme TM`.
>
> **Render notation. Flatten typography.**

So this is not a special case bolted onto RFC 043. It is RFC 043's own existing behaviour, stated as
a rule and then applied consistently. That is what makes it instinctive rather than a list of
exceptions to memorise.

## 4. Proposal

Flatten, where the content is unambiguously an ordinal suffix:

- **`st`, `nd`, `rd`, `th` immediately after a digit.** No notation uses these.
- **`º` (U+00BA) and `ª` (U+00AA)**, the ordinal indicators. They have no other use, and today they
  are the ones producing `^(º)` — the worst of the three outputs.

**Not proposed, deliberately:** French `e` and `er`. `10<sup>e</sup>` is a legitimate exponent (ten
to the power *e*), so flattening it would destroy notation to tidy typography — the exact error this
RFC exists to avoid. It stays as it is, and the documentation says so rather than leaving it to be
discovered.

**Not in scope:** `TM` and `®`. Mapping `<sup>TM</sup>` to `™` is a different change — a *mapping*,
not a flattening — and should be asked for by someone before it is built.

## 5. Open question for the owner

**Does this warrant its own release, or does it wait?** It changes output for every user and reverts
ordinals to what `2.5.1` produced. bekoedit has pinned the current output in a fixture specifically
so they notice, and has said they have no date in mind.

*Recommendation: not urgent, and not bundled with `3.1.1` — that release is a defect fix for an
opt-in option and should stay that way. This is an output change and deserves its own line in a
CHANGELOG where nobody is reading past it.*

## 6. Criteria

1. §1's rows 3 and 5 both produce plain text; rows 1, 2 and 7 are unchanged.
2. `10<sup>n</sup>` and `2<sup>n − 1</sup>` are unchanged — **the rule must not reach an exponent
   that happens to follow a digit.** This is the one way the fix can go wrong.
3. French `1<sup>er</sup>` unchanged, and documented as a deliberate limit with the `10ᵉ` reason.
4. The 417-occurrence corpus behind RFC 043 is re-run and the delta reported per category.
5. `mode_identity`'s goldens move only where §1 says they should, and every moved golden is named.
