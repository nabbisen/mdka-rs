# Developer Handoff — RFC 051, typographic superscripts

**Authorised.** Owner, 2026-10-01 — RFC 051 accepted, milestone **`3.2.0`**.
**RFC.** `rfcs/accepted/051-typographic-superscripts.md` — read §2 and §3 before any code.
**Scope.** The `<sup>`/`<sub>` mapping (RFC 043), `docs/src/`, tests. Implementation only; the bump,
CHANGELOG and RFC move are a separate prep slice.

## 🛑 Do not start this until `3.1.1` is tagged

RFC 050 ships as `3.1.1`, which promises **no output change for anyone**. This RFC changes output for
every user with an ordinal. **If this lands on `main` first, `3.1.1` is cut from a tip that already
contains it** and that promise is false.

**Order: RFC 050 → `3.1.1` prep → tag → this.** If RFC 050's slice is still in flight, this one waits.
Say in your report which state `main` was in when you started.

---

## 1. The rule, which is the point of the whole RFC

> **Render notation. Flatten typography.** A superscript is notation when flattening it changes what
> it means (`10⁻⁹` → `10-9`); typography when flattening changes nothing (`1st` is `1st`).

RFC 043 already does this for citation markers without having written it down. **Write the rule into
the code's own documentation, not just the output change** — the next person deciding a `<sup>` case
should be able to answer it from the rule rather than from a list.

## 2. What to flatten

- **`st`, `nd`, `rd`, `th` immediately after a digit.** No notation uses these.
- **`º` (U+00BA), `ª` (U+00AA)** — the ordinal indicators. These are the ones producing `^(º)` today,
  the worst of the three current outputs.

## 3. What to leave alone, deliberately 🛑

- **French `er` and `e`.** `10<sup>e</sup>` is a legitimate exponent — ten to the power *e*. Flattening
  it would destroy notation to tidy typography, which is the exact error this RFC exists to prevent.
  **Document it as a deliberate limit, with that reason**, rather than leaving it to be discovered.
- **`TM` and `®`.** Mapping `<sup>TM</sup>` → `™` is a *mapping*, not a flattening. Out of scope.
- **Every existing notation case.** `x²`, `10⁻⁹`, `H₂O`, `2^(n − 1)`.

## 4. The way this fix goes wrong

```
10<sup>n</sup>        must stay 10ⁿ      ← alphabetic, follows a digit, and IS an exponent
2<sup>n − 1</sup>     must stay 2^(n − 1)
```

A rule phrased as *"letters after a digit"* breaks both. The flatten set is the four English suffixes
and the two ordinal indicators, **as a closed set** — not a shape. Write it as one.

## 5. Criteria

1. RFC 051 §1's ordinal rows flatten; its notation rows are byte-identical to `3.1.0`.
2. §4's two cases explicitly tested, by those names.
3. French `1<sup>er</sup>` unchanged **and documented** with the `10ᵉ` reason.
4. **The 417-occurrence corpus behind RFC 043 is re-run and the delta reported per category** — how
   many occurrences move, in which category, and that none is a notation case. This is the criterion
   that justifies the release; a sample is not a substitute.
5. `mode_identity`'s goldens move **only** where §1 says they should, and **every moved golden is
   named in the report.** A golden that moves for an unexplained reason stops this slice.
6. The rule from §1 appears in `docs/src/` and in the code, in those words.
7. Counts, each with its command. `cargo fmt --check`, `clippy -D warnings`, docs-example gate clean.

## 6. Report, then stop

`.git-exclude/review-request/051-typographic-superscripts/README.md`, leading with criteria 4 and 5.
Commit and push **only this slice** — never a release, never unrelated commits.
