# mdka — Roadmap

**Status.** Active — planning baseline approved by the project owner on 2026-08-02.
**Current version.** 2.1.6
**Governance.** RFC lifecycle follows [RFC 000](./rfcs/done/000-rfc-lifecycle-policy.md).

This document is the planning baseline from which the RFC portfolio is derived.
It records milestones, sequencing, and release policy. It does not record design
decisions — those live in individual RFCs.

---

## Release policy

| Release type | Trigger | Contents | Deliverable |
|---|---|---|---|
| **Patch** `X.Y.Z+1` | Ad hoc | Docs, CI, dependency bumps, bug fixes with no API change | Tarball + CHANGELOG entry |
| **Minor** `X.Y+1.0` | One milestone completed | Additive API, new element support | Tarball + CHANGELOG + RFC dispositions |
| **Major** `X+1.0.0` | Project owner decision only | Compatibility break | Migration guide required |

### Merge policy

**Decided by the project owner, 2026-08-02: commits go directly to `main`.** No
pull-request requirement, no branch protection.

Rationale: with a single committer, pull requests buy enforcement rather than
coordination, and the friction is recurring. Detection is unaffected — CI runs
on push and reports within minutes; recovery is a `git revert` that blocks
nobody.

The accepted consequence is that **CI on `main` is advisory**. Release-time
enforcement is provided separately by RFC 014, which prevents publication from
any commit whose CI did not pass. That is where the exposure that reaches users
actually lives.

### Release mechanics

- Tag format is `X.Y.Z` — no `v` prefix (Rust crate convention).
- Release archives place files at the archive root, with no intermediate parent
  directory, and carry the version in the filename (e.g. `mdka-2.1.7.tar`).
- Scheduling is **sequence-based**, not date-bound. Releases are cut at logical
  breaking points — normally when a milestone's RFCs are all resolved.
- One milestone maps to one release unless the owner directs otherwise.

### Major version position

No major version transition is planned on this roadmap. RFC 005 resolves the
`ConversionOptions` defect additively, within the 2.x compatibility line. Any
future major-version decision is reserved to the project owner and is not
implied by completion of any milestone below.

---

## Milestones

### M1 · Trustworthy baseline → `2.1.7` (patch)

No behaviour change. Establishes the quality gate that every later milestone
lands through, and closes the documentation claims that are wrong regardless of
any pending design decision.

Sequencing within M1: **001 → 002 → 004 → 003**. RFC 003 rewrites the workspace
layout section of the architecture docs, so it must follow RFC 004's disposition
of the orphaned preprocessor.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 001 | CI quality gates | P0 | S |
| 002 | Governance artifacts | P0 | S |
| 004 | Orphaned preprocessor disposition | P0 | S |
| 003 | Architecture documentation reconciliation | P0 | S |
| 014 | Release-time CI verification | P1 | S |

RFC 014 was added mid-milestone, after the merge-policy decision above made
explicit that CI on `main` is advisory. It depends only on RFC 001 and runs
independently of 002/003/004.

**Exit criteria.** CI green on `main` with `-D warnings` enforced; no release
can publish from a commit whose CI did not pass; CHANGELOG covers every
published version; no documentation statement contradicts observed engine
behaviour except those explicitly owned by RFC 005/006.

### M2 · Truth in the API surface → `2.2.0` (minor)

Closes the gap between what `ConversionOptions`, the CLI, the bindings, and the
documentation promise, and what the engine actually does. Six of the eight
option fields are currently inert.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 005 | `ConversionOptions` semantics — implement attribute handling | P0 | L |
| 006 | Option documentation + binding parity realignment | P1 | M |
| 007 | English-only public surface | P1 | M |

**Exit criteria.** Every public option field demonstrably changes output, with a
test per field per surface; Rust, CLI, Node, and Python expose the same option
set; no Japanese text in any artifact published to crates.io, npm, or PyPI.

### M3 · Conversion fidelity → `2.3.0` (minor)

Purely additive element coverage. Tables are the largest known gap against the
project's GFM positioning; today `<table>` content is emitted as an unstructured
text run.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 008 | GFM table support | P0 | L |
| 010 | Escaping & text-processing correctness audit | P1 | M |
| 009 | Element coverage extension (`dl`/`dt`/`dd`, `del`/`s`, `sup`/`sub`) | P2 | M |

**Exit criteria.** Tables round-trip to GFM pipe syntax including alignment and
header rows; the escaping audit has either confirmed or corrected each rule in
`docs/src/api/text-processing.md` against a test.

### M4 · Durability → `2.4.0` (minor)

| RFC | Title | Priority | Size |
|---|---|---|---|
| 011 | Robustness: fuzzing + `MdkaError::Io` error-path tests | P2 | M |
| 012 | Benchmark hardening + regenerate published performance claims | P2 | M |
| 013 | Internal comment migration to English | P2 | L |

RFC 013 is a large, purely mechanical diff. It is scheduled into a quiet release
deliberately, so it does not bury substantive changes in `git blame`.

#### Carried-forward review findings

Deferred here by owner decision on 2026-08-02 rather than reopening completed
RFCs. None is a correctness issue; all are hygiene. Recorded so they are not
lost — a deferred item tracked nowhere is an abandoned item.

| ID | Finding | Source | Suggested home |
|---|---|---|---|
| R-01 | `mdka-cli`, `mdka-node`, `mdka-python` declare no `rust-version`. Only the root `mdka` package does. Already covered transitively — `mdka-cli` depends on `mdka`, whose declaration propagates through the resolver — so this is tooling and crates.io display hygiene. Cleanest fix is `[workspace.package] rust-version` plus `rust-version.workspace = true` in each member. | RFC 001 review, round 2 | New slice, or fold into RFC 012 |
| R-02 | The `node` CI job checks `node/index.d.ts` for drift but not `node/index.js`, though `package.json` publishes both and `npm run build` regenerates both. | RFC 001 review, round 2 | Same |
| R-03 | `node/test.js` terminates the whole process on first failure — three concurrently-started async IIFEs share counters, and whichever finishes first with `failed > 0` calls `process.exit(1)`. Demonstrated: a single broken assertion left 5 of 35 tests unrun. The gate's exit code is correct, so CI still fails; diagnosis is what degrades. | RFC 001 review, round 2 | RFC 011 |

R-01 and R-02 originate in the architect's specification, not in implementation
work.

**Exit criteria.** Fuzz target runs clean for a defined budget; every performance
figure in `docs/src/design/` regenerated from a current benchmark run; no
Japanese text remains in `src/`, `cli/`, `node/`, or `python/`.

---

## Portfolio at a glance

| RFC | Title | Milestone | Priority | Size | Depends on |
|---|---|---|---|---|---|
| 001 | CI quality gates | M1 | P0 | S | — |
| 002 | Governance artifacts | M1 | P0 | S | — |
| 003 | Architecture documentation reconciliation | M1 | P0 | S | 004 |
| 004 | Orphaned preprocessor disposition | M1 | P0 | S | 001 |
| 005 | `ConversionOptions` semantics | M2 | P0 | L | 001, 004 |
| 006 | Option docs + binding parity | M2 | P1 | M | 005 |
| 007 | English-only public surface | M2 | P1 | M | — |
| 008 | GFM table support | M3 | P0 | L | 001 |
| 009 | Element coverage extension | M3 | P2 | M | 008 |
| 010 | Escaping & text-processing audit | M3 | P1 | M | 001 |
| 011 | Robustness: fuzzing + I/O error paths | M4 | P2 | M | 001 |
| 012 | Benchmark hardening | M4 | P2 | M | 008 |
| 013 | Internal comment migration to English | M4 | P2 | L | 007 |
| 014 | Release-time CI verification | M1 | P1 | S | 001 |

Numbers are permanent and never reused, per RFC 000. Numbers 005–013 are
reserved; those RFCs are drafted at the start of their milestone rather than up
front, so their design reflects what the preceding milestone actually shipped.

---

## Roadmap maintenance

At every RFC disposition point (implemented, withdrawn, superseded, deferred)
the following is reviewed and any material change reported to the project owner:

- RFC portfolio and milestone progress
- Whether roadmap assumptions still hold
- Dependency and risk changes
- Whether new RFCs are required, or existing ones should be split or merged

When all milestones are resolved, or no substantial development theme remains,
that state is reported to the project owner and joint replanning resumes. The
roadmap is not extended unilaterally.
