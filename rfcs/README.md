# mdka RFCs

Design records for the mdka project. Lifecycle, folder semantics, and naming are
governed by [RFC 000](./done/000-rfc-lifecycle-policy.md).

**The folder is the source of truth for an RFC's state.** Each file's `Status`
field mirrors its folder; if the two ever disagree, the folder wins.

Planning context for the whole portfolio lives in [`ROADMAP.md`](../ROADMAP.md).

## Proposed

None. Everything currently open has been accepted — see below.

## Accepted

Review complete; the implementer may start. Folder is the source of truth for
state, per [RFC 000](./done/000-rfc-lifecycle-policy.md).

| ID | Title | Milestone | Priority |
|----|-------|-----------|----------|
| 025 | [Markdown output-validity harness](./accepted/025-output-validity-harness.md) — [handoff](./handoffs/025-output-validity-harness/implementation-handoff.md), [addendum 025c](./handoffs/025-output-validity-harness/addendum-025c.md) | M3 → `2.3.0` | **P0** — ✅ harness (`7338b17`) and `025c` (`d5d64cd`) approved; corpus `025b` unscheduled |
| 024 | [Inline composition: the output sink](./accepted/024-inline-composition-output-sink.md) — [handoff](./handoffs/024-inline-composition-output-sink/implementation-handoff.md) | M3 → `2.3.0` | **P0** — **next, handed over 2026-09-16** (19 cells) |
| 028 | [Inline elements around block content, and emphasis negated by its own style](./accepted/028-emphasis-around-block-content.md) — [handoff](./handoffs/028-emphasis-around-block-content/implementation-handoff.md) | M3 → `2.3.0` | **P0** — queued behind 024; scope amended 2026-09-16 (22 cells) |
| 010 | [Escaping and text round-trip](./accepted/010-escaping-and-text-round-trip.md) — [handoff](./handoffs/010-escaping-and-text-round-trip/implementation-handoff.md) | M3 → `2.3.0` | **P0** — queued behind 028; accepted 2026-09-16 (24+ cells) |
| 030 | [Crates package gate: verify the workspace, not the registry](./accepted/030-crates-package-gate-workspace-resolution.md) — [handoff](./handoffs/030-crates-package-gate-workspace-resolution/implementation-handoff.md) | M3 → `2.3.0` | **P1** — ✅ implemented & approved; moves to `done/` at `2.3.0` prep |
| 031 | [Docs example gate must compile what mdBook publishes](./accepted/031-docs-gate-must-model-mdbook.md) — [handoff](./handoffs/031-docs-gate-must-model-mdbook/implementation-handoff.md), [follow-up 031b](./handoffs/031-docs-gate-must-model-mdbook/followup-031b.md) | M3 → `2.3.0` (docs publish on merge) | **P1** — ✅ implemented & approved (031, 031b); D6 open (owner); moves to `done/` at `2.3.0` prep |
| 032 | [Gates report every failure, and execute Python and TypeScript examples](./accepted/032-gates-report-everything-and-execute-examples.md) — [handoff](./handoffs/032-gates-report-everything-and-execute-examples/implementation-handoff.md) | M3 → `2.3.0` | **P2** — ✅ implemented & approved; moves to `done/` at `2.3.0` prep |
| 033 | [Published docs: the source is what the reader gets](./accepted/033-published-docs-source-is-what-the-reader-gets.md) — [handoff](./handoffs/033-published-docs-source-is-what-the-reader-gets/implementation-handoff.md) | M3 → `2.3.0` (docs publish on merge) | **P1** — ✅ implemented & approved; moves to `done/` at `2.3.0` prep |
| 034 | [PyPI: a declared wheel matrix, built on purpose and checked where it is published](./accepted/034-pypi-declared-wheel-matrix.md) — [handoff](./handoffs/034-pypi-declared-wheel-matrix/implementation-handoff.md) | M3 → `2.3.0` | **P1** — ✅ implemented & approved (034, 034b); moves to `done/` at `2.3.0` prep |

All nine are M3 · Output validity → `2.3.0`. **Sequencing: 030–034 ✅, 025 ✅, 025c ✅ → 024 → 028 → 010.**
Tables (RFC 008) and element coverage (RFC 009) moved to M4 / `2.4.0` by owner decision on 2026-09-16; neither has a file yet.

- **030 and 031 go first** and are independent of the engine work. Both are
  control repairs: each fixes a gate that was passing something the consumer's
  artifact fails. Fixing the instruments before taking the measurements.
- **032 precedes 025** because `cargo test` without `--no-fail-fast` would show
  only the first failing binary of a harness built to show many failures.
- **033 follows 032, not in parallel with it.** Both edit the docs gate script;
  its handoff opens with a stop block until 032 is approved.
- **025 precedes 024**, and **028 follows both** — the harness must be able to
  observe the defects before the fixes claim to have removed them, and RFC 028's
  mechanism choice depends on RFC 024's shape.

- **028 and 010 follow 024, one at a time.** All three change `src/renderer.rs`;
  each handoff opens with a stop block naming the approval it waits for.

**Handoffs for 024, 028 and 010 are queued, not dispatched** — each §0 states its
precondition. **Handoffs are frozen once named ready**; later changes arrive as dated
addenda.

## Implemented

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | 2.1.6 |
| 001 | [CI quality gates](./done/001-ci-quality-gates.md) — [handoff](./handoffs/001-ci-quality-gates/implementation-handoff.md) | 2.1.7 |
| 002 | [Governance artifacts](./done/002-governance-artifacts.md) — [handoff](./handoffs/002-governance-artifacts/implementation-handoff.md) | 2.1.7 |
| 003 | [Architecture documentation reconciliation](./done/003-architecture-doc-reconciliation.md) — [handoff](./handoffs/003-architecture-doc-reconciliation/implementation-handoff.md) | 2.1.7 |
| 004 | [Orphaned preprocessor disposition](./done/004-preprocessor-disposition.md) — [handoff](./handoffs/004-preprocessor-disposition/implementation-handoff.md) | 2.1.7 |
| 014 | [Release-time CI verification](./done/014-release-time-ci-verification.md) — [handoff](./handoffs/014-release-time-ci-verification/implementation-handoff.md) | 2.1.7 |
| 015 | [Release tooling completion](./done/015-release-tooling-completion.md) — [handoff](./handoffs/015-release-tooling-completion/implementation-handoff.md) | 2.1.8 |
| 016 | [`<hr>` newline reset](./done/016-hr-newline-reset.md) — [handoff](./handoffs/016-hr-newline-reset/implementation-handoff.md) | 2.1.8 |
| 017 | [`<pre>` fence newline reset](./done/017-pre-fence-newline-reset.md) — [handoff](./handoffs/017-pre-fence-newline-reset/implementation-handoff.md) | 2.1.8 |
| 005 | [`ConversionOptions` semantics](./done/005-conversion-options-semantics.md) — [Slice A](./handoffs/005-conversion-options-semantics/implementation-handoff.md) · [Slices B/C](./handoffs/005-conversion-options-semantics/slices-bc-handoff.md) · [B1 placement](./handoffs/005-conversion-options-semantics/slice-b1-placement-correction-handoff.md) | 2.2.0 |
| 006 | [Option docs and binding parity](./done/006-option-docs-and-binding-parity.md) — [handoff](./handoffs/006-option-docs-and-binding-parity/implementation-handoff.md) · closed M2 | 2.2.0 |
| 018 | [README Quick Start: prebuilt binaries](./done/018-readme-prebuilt-binaries.md) — [handoff](./handoffs/018-readme-prebuilt-binaries/implementation-handoff.md) | 2.2.0 |
| 019 | [Release creation via dispatch](./done/019-release-creation-via-dispatch.md) — [handoff](./handoffs/019-release-creation-via-dispatch/implementation-handoff.md) · superseded RFC 015 Slice 2 | 2.2.0 |
| 020 | [npm distribution repair + install gate](./done/020-npm-distribution-repair.md) — [handoff](./handoffs/020-npm-distribution-repair/implementation-handoff.md) | 2.2.1 |
| 007 | [English-only public surface](./done/007-english-only-public-surface.md) — [handoff](./handoffs/007-english-only-public-surface/implementation-handoff.md) | 2.2.2 |
| 021 | [Bulk conversion output-collision safety](./done/021-bulk-output-collision-safety.md) — [handoff](./handoffs/021-bulk-output-collision-safety/implementation-handoff.md) | 2.2.2 |
| 022 | [CLI allocator; settle `jemalloc`](./done/022-cli-allocator-and-jemalloc.md) — [handoff](./handoffs/022-cli-allocator-and-jemalloc/implementation-handoff.md) · [deprecation](./handoffs/022-cli-allocator-and-jemalloc/alloc-counter-deprecation-handoff.md) | 2.2.2 |
| 023 | [Getting-started documentation reconciliation](./done/023-getting-started-doc-reconciliation.md) — [handoff](./handoffs/023-getting-started-doc-reconciliation/implementation-handoff.md) | 2.2.2 |
| 026 | [Consumer-artifact verification gates](./done/026-consumer-artifact-gates.md) — [handoff](./handoffs/026-consumer-artifact-gates/implementation-handoff.md) | 2.2.2 |
| 027 | [Verification discipline: the consumer pass](./done/027-verification-discipline.md) — [handoff](./handoffs/027-verification-discipline/implementation-handoff.md) | 2.2.2 |
| 029 | [Published-surface documentation repair](./done/029-published-surface-documentation-repair.md) — [handoff](./handoffs/029-published-surface-documentation-repair/implementation-handoff.md) | 2.2.3 |

## Archive

None.

## Reserved numbers

These numbers are allocated in the roadmap but not yet drafted. Each is written
at the start of its milestone, so its design reflects what the preceding
milestone actually shipped. Numbers are permanent and are never reused.

| ID | Title | Milestone |
|----|-------|-----------|
| 008 | GFM table support | M3 |
| 009 | Element coverage extension | M3 |
| 010 | Escaping & text-processing audit | M3 |
| 011 | Robustness: fuzzing + I/O error paths | M4 |
| 012 | Benchmark hardening | M4 |
| 013 | Internal comment migration to English | M4 |

## Maintaining this index

Update this file in the same commit that moves an RFC between folders. Before
moving one, run `grep -rl 'NNN-slug.md' rfcs/` and fix inbound references in the
same commit.
