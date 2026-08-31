# RFC 023 — Getting-started documentation reconciliation

**Status.** Proposed
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P1
**Touches.** `docs/src/getting-started/*`, `docs/src/api/core.md`, `docs/src/api/elements.md`, `python/` packaging.
**Source.** External audit 2026-08-31 — `D-04`, `D-06`, `D-07`, `D-08`, `D-09`, `D-10`, `D-12`, `D-13`, `D-14`, `D-15`.
**Prepared.** 2026-08-31

## Summary

M2 corrected the API reference and left the getting-started layer behind. The
audit found the two now contradict each other, and that several published
examples do not run.

## Why this exists — and it is my error

RFC 006's scope, which I wrote, named `docs/src/api/`. It never swept
`docs/src/getting-started/`. So `docs/src/api/options.md` correctly documents
five options as deprecated no-ops while `docs/src/getting-started/usage-cli.md`
still tells CLI users those flags keep attributes.

**The defect class M2 existed to eliminate survived one directory away, because
a scope I wrote drew a boundary that had no reason to be there.** The audit found
it in one pass by reading the docs as a new user would, front to back, rather
than by directory.

## Confirmed defects

I verified these directly.

| ID | Defect |
|---|---|
| `D-06` | `usage-cli.md:49-51` documents `--preserve-classes`, `--preserve-data`, `--preserve-aria` as "Keep X attributes". They are deprecated no-ops. The page also never mentions `--unwrap-wrappers`, the one flag added in 2.2.0 that works — the same inversion the bindings had before RFC 006. |
| `D-08` | `usage-python.md:116` states *"mdka ships with a `py.typed` marker (PEP 561). All public symbols are annotated."* There is no `py.typed` and no `.pyi` anywhere under `python/`. Verified in the repo; the audit verified it in the published sdist too. |

Taken from the audit and to be re-verified by the implementer before correcting:
`D-04` (`core.md:103` describes the bulk API's behaviour backwards), `D-07`
(Python and Node guides recommend a deprecated no-op that now emits a warning),
`D-09`/`D-10` (`elements.md`: `<span>` listed as a block separator; tables never
mentioned), `D-12`/`D-13` (Node and TypeScript examples do not run — duplicate
`const`, undeclared import, unexported type), `D-14` (placeholder clone URL),
`D-15` (assorted smaller mismatches).

## Design

**Correct the claim, or delete it. Do not soften it.** The standard is the one
`api/options.md` and `api/modes.md` already meet — the audit called them models
of honest documentation, and the fix is to apply that existing standard to the
pages left behind, not to invent a new one.

### `py.typed` is a decision, not a correction

Two honest outcomes, and the implementer must not pick silently:

- **Ship the marker.** Add `py.typed`, ensure packaging includes it, and confirm
  the *published wheel and sdist* contain it. Then the claim becomes true.
- **Delete the claim.** Then typed Python users are correctly informed.

Prefer shipping it — the annotations largely exist and the promise is
reasonable — but **only if it can be verified in a built artifact**. A `py.typed`
that exists in the repo and not in the wheel is the same defect in a new place.
This is the RFC 020 lesson: check what the user receives.

### Every code example must be executed

`D-12` and `D-13` are examples that fail on their first line. Any snippet
presented as runnable must be run by the implementer and its real output
recorded. Snippets that are deliberately illustrative fragments must be marked as
such so the distinction is visible.

### Add what is missing, not only fix what is wrong

- `usage-cli.md` gains `--unwrap-wrappers`, and marks the three deprecated flags
  as deprecated no-ops, matching `--help`.
- `elements.md` gains a **"Not Yet Supported"** section naming tables (RFC 008),
  `<dl>`, `<del>`/`<s>`, `<sup>`/`<sub>` (RFC 009), and `<video>`/`<audio>`
  (`A-13`). A reader currently cannot discover that tables are unsupported from
  the element reference at all.

## Not in scope

`TROUBLESHOOTING.md` and `MIGRATION.md` (audit §3c/3d) — both are real gaps and
both are new documents rather than corrections. M3 or M4.

The stale benchmark figures (`D-11`) stay with RFC 012, except the duplicated
`dom_smoothie` cell, which is a transcription error rather than staleness and
should be fixed here.

## Compatibility

Documentation only. No code change except the `py.typed` packaging decision.

## Acceptance criteria

1. No getting-started page documents a no-op as working.
2. `usage-cli.md`'s option table matches `--help` exactly, `--unwrap-wrappers`
   included.
3. Every runnable example has been executed by the implementer, with output
   recorded in the review request.
4. The `py.typed` claim is true in a **built artifact**, or removed.
5. `elements.md` has a "Not Yet Supported" section naming tables.
6. `mdbook build` is clean.
7. Each corrected claim is listed with the source location that verifies the new
   wording — the RFC 006 evidence standard.
