# mdka RFCs

Design records for the mdka project. Lifecycle, folder semantics, and naming are
governed by [RFC 000](./done/000-rfc-lifecycle-policy.md).

**The folder is the source of truth for an RFC's state.** Each file's `Status`
field mirrors its folder; if the two ever disagree, the folder wins.

Planning context for the whole portfolio lives in [`ROADMAP.md`](../ROADMAP.md).

## Proposed

| ID | Title | Milestone | Priority |
|----|-------|-----------|----------|
| 020 | [npm distribution repair + install gate](./proposed/020-npm-distribution-repair.md) | M2b → `2.2.1` | **P0** |
| 021 | [Bulk conversion output-collision safety](./proposed/021-bulk-output-collision-safety.md) | M2b → `2.2.1` | **P0** |
| 022 | [CLI allocator removal; settle `jemalloc`](./proposed/022-cli-allocator-and-jemalloc.md) | M2b → `2.2.1` | P1 |
| 023 | [Getting-started documentation reconciliation](./proposed/023-getting-started-doc-reconciliation.md) | M2b → `2.2.1` | P1 |
| 025 | [Markdown output-validity harness](./proposed/025-output-validity-harness.md) | M3 → `2.3.0` | **P0** |
| 024 | [Inline composition: the output sink](./proposed/024-inline-composition-output-sink.md) | M3 → `2.3.0` | **P0** |

All six arise from the independent audit of 2026-08-31. Handoffs follow once the
owner approves the set. **RFC 025 precedes 024**: the harness must be able to
observe the defects before the fixes claim to have removed them.

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

## Archive

None.

## Reserved numbers

These numbers are allocated in the roadmap but not yet drafted. Each is written
at the start of its milestone, so its design reflects what the preceding
milestone actually shipped. Numbers are permanent and are never reused.

| ID | Title | Milestone |
|----|-------|-----------|
| 007 | English-only public surface | M2 |
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
