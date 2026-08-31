# Developer Handoff — RFC 023 · Getting-started documentation reconciliation

**Governing RFC.** [RFC 023](../../accepted/023-getting-started-doc-reconciliation.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P1
**Prepared.** 2026-08-31

---

## 1. Purpose

M2 corrected the API reference and left the getting-started layer behind. The two
now contradict each other, and several published examples do not run.

**This exists because of a boundary I drew.** RFC 006's scope named
`docs/src/api/` and never swept `getting-started/`. You implemented that scope
correctly. The scope was wrong.

## 2. Confirmed defects — verified by the architect

| ID | Defect |
|---|---|
| `D-06` | `usage-cli.md:49-51` documents `--preserve-classes`, `--preserve-data`, `--preserve-aria` as "Keep X attributes". All three are deprecated no-ops. The page never mentions `--unwrap-wrappers`, the one flag added in 2.2.0 that works. |
| `D-08` | `usage-python.md:116` claims a `py.typed` marker (PEP 561). There is no `py.typed` and no `.pyi` anywhere under `python/`. |

## 3. Reported by the audit — re-verify before correcting

Do not correct these from the audit's description. Check each against source
first, and **report any that turn out to be different from described** — that
list is more valuable than the corrections.

`D-04` (`core.md:103` describes bulk behaviour backwards), `D-07` (Python and
Node guides recommend a deprecated no-op that now warns), `D-09` (`elements.md`
lists `<span>` as a block separator), `D-10` (tables never mentioned), `D-12`
(Node examples: duplicate `const`, undeclared import), `D-13` (TypeScript example
imports an unexported type), `D-14` (placeholder clone URL), `D-15` (assorted).

Also fix the duplicated `dom_smoothie` benchmark cell (`D-11b`) — a transcription
error, not staleness. The stale figures themselves stay with RFC 012.

## 4. The standard

**Correct the claim, or delete it. Do not soften it.**

`api/options.md` and `api/modes.md` already meet the bar — the audit called them
models of honest documentation. Apply that existing standard to the pages left
behind. Do not invent a new one, and do not lower it.

## 5. ⚠ `py.typed` is a decision

Two honest outcomes. **Pick deliberately and record which:**

- **Ship the marker** — add `py.typed`, ensure packaging includes it, and confirm
  it is in the **built wheel and sdist**, not just the repo.
- **Delete the claim** — then typed Python users are correctly informed.

Prefer shipping it; the annotations largely exist. **But only if you can verify
it in a built artifact.** A `py.typed` present in the repo and absent from the
wheel is the same defect in a new place — that is exactly how `D-08` and `S-01`
both happened.

**Tell RFC 026's implementer which way you went.** Their wheel gate asserts
`py.typed` only if you ship it.

## 6. Every runnable example must be executed

`D-12` and `D-13` are examples that fail on their first line. Run every snippet
you present as runnable, and record its real output.

Snippets that are deliberately fragments must be **marked as such**, and the
marker is what excludes them. Coordinate the convention with RFC 026's
docs-example gate — you are correcting the content, they are building the thing
that keeps it correct.

## 7. Add what is missing

- `usage-cli.md`: add `--unwrap-wrappers`; mark the three deprecated flags as
  deprecated no-ops, matching `--help` exactly.
- `elements.md`: add a **"Not Yet Supported"** section naming tables (RFC 008),
  `<dl>`, `<del>`/`<s>`, `<sup>`/`<sub>` (RFC 009), `<video>`/`<audio>`
  (`A-13`). A reader currently cannot discover from the element reference that
  tables are unsupported.

**Scope boundary, per RFC 027 Rule 2.** This covers `docs/src/getting-started/`,
plus `api/core.md` and `api/elements.md` where the audit found specific errors.
**Not covered:** `TROUBLESHOOTING.md` and `MIGRATION.md` — both are new documents
rather than corrections, deferred to M3/M4. Benchmark staleness stays with
RFC 012.

## 8. Required verification

Per RFC 027 Rule 3, state tree-vs-artifact for each — this matters especially for
`py.typed`.

1. Each corrected claim, with the **source location** that verifies the new
   wording. This is the RFC 006 evidence standard.
2. Every runnable example executed, with output.
3. `usage-cli.md`'s table diffed against `--help` output.
4. If shipping `py.typed`: proof it is in a built wheel **and** sdist.
5. `mdbook build` clean.
6. `cargo test --workspace --locked`, fmt, clippy — unchanged.

## 9. Prohibited shortcuts

- Do not soften a no-op into "limited effect" or "reserved for future use".
- Do not correct a claim you have not verified against source.
- Do not present a snippet as runnable without running it.
- Do not add `py.typed` to the repo and call the claim true.
- Do not fix `TROUBLESHOOTING.md`/`MIGRATION.md` — out of scope, and the gap is
  recorded.

## 10. Known risks

| Risk | If it happens |
|---|---|
| An audit-reported defect is not as described | **Report it.** The audit is accurate so far — 12/12 verified — so a discrepancy is interesting. |
| `py.typed` packaging is harder than expected | Report it and take the delete-the-claim option rather than shipping an unverifiable claim. |
| More stale claims turn up than the audit listed | Very likely. Fix them and list them; that list is a deliverable. |

## 11. Acceptance checklist

- [ ] No getting-started page documents a no-op as working
- [ ] `usage-cli.md` matches `--help` exactly, `--unwrap-wrappers` included
- [ ] Every runnable example executed, output recorded
- [ ] `py.typed` true in a built artifact, or the claim removed — decision recorded
- [ ] RFC 026's implementer told which way
- [ ] `elements.md` has "Not Yet Supported" naming tables
- [ ] Duplicated benchmark cell fixed
- [ ] Each corrected claim paired with its verifying source location
- [ ] `mdbook build` clean

## 12. Escalate rather than decide

Stop and raise if: a claim cannot be made true without a code change; `py.typed`
cannot be verified in a built artifact; or you find a documented behaviour that
contradicts the code in a way no listed finding covers.
