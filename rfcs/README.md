# mdka RFCs

Design records for the mdka project. Lifecycle, folder semantics, and naming are
governed by [RFC 000](./done/000-rfc-lifecycle-policy.md).

**The folder is the source of truth for an RFC's state.** Each file's `Status`
field mirrors its folder; if the two ever disagree, the folder wins.

Planning context for the whole portfolio lives in [`ROADMAP.md`](../ROADMAP.md).

## Proposed

None. Everything currently open has been accepted — see below.

## Accepted

| ID | Title | State |
|----|-------|-------|
| 040 | [npm platform coverage, and an error message that contradicts the documentation](./accepted/040-npm-platform-coverage.md) | **Accepted (owner, 2026-09-24).** npm ships 3 platform families where the CLI ships 5 and PyPI 6; on the rest `npm install` exits 0 and `require` blames a non-existent npm bug. Part B (the message) outranks Part A (the matrix) |
| 039 | [Public API surface coherence](./accepted/039-public-api-surface-coherence.md) — [Half A handoff](./handoffs/039-public-api-surface-coherence/half-a-handoff.md) | **Half A shipped in 2.4.0**; Half B specifies the `3.0` target and is **unscheduled**, needing its own acceptance |

M4's remaining work is RFC 011 (robustness) and RFC 013 (comment migration); both are reserved, see below.

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
| 022 | [CLI allocator; settle `jemalloc`](./done/022-cli-allocator-and-jemalloc.md) — [handoff](./handoffs/022-cli-allocator-and-jemalloc/implementation-handoff.md) · [deprecation](./handoffs/022-cli-allocator-and-jemalloc/alloc-counter-deprecation-handoff.md) · [second half](./handoffs/022-cli-allocator-and-jemalloc/second-half-handoff-2026-09-23.md) | 2.2.2; `alloc_counter` removed in 2.4.0 |
| 023 | [Getting-started documentation reconciliation](./done/023-getting-started-doc-reconciliation.md) — [handoff](./handoffs/023-getting-started-doc-reconciliation/implementation-handoff.md) | 2.2.2 |
| 026 | [Consumer-artifact verification gates](./done/026-consumer-artifact-gates.md) — [handoff](./handoffs/026-consumer-artifact-gates/implementation-handoff.md) | 2.2.2 |
| 027 | [Verification discipline: the consumer pass](./done/027-verification-discipline.md) — [handoff](./handoffs/027-verification-discipline/implementation-handoff.md) | 2.2.2 |
| 029 | [Published-surface documentation repair](./done/029-published-surface-documentation-repair.md) — [handoff](./handoffs/029-published-surface-documentation-repair/implementation-handoff.md) | 2.2.3 |
| 030 | [Crates package gate: verify the workspace, not the registry](./done/030-crates-package-gate-workspace-resolution.md) — [handoff](./handoffs/030-crates-package-gate-workspace-resolution/implementation-handoff.md) | 2.3.0 |
| 031 | [Docs example gate must compile what mdBook publishes](./done/031-docs-gate-must-model-mdbook.md) — [handoff](./handoffs/031-docs-gate-must-model-mdbook/implementation-handoff.md) · [031b](./handoffs/031-docs-gate-must-model-mdbook/followup-031b.md) | 2.3.0 |
| 032 | [Gates report every failure; execute Python and TypeScript examples](./done/032-gates-report-everything-and-execute-examples.md) — [handoff](./handoffs/032-gates-report-everything-and-execute-examples/implementation-handoff.md) | 2.3.0 |
| 033 | [Published docs: the source is what the reader gets](./done/033-published-docs-source-is-what-the-reader-gets.md) — [handoff](./handoffs/033-published-docs-source-is-what-the-reader-gets/implementation-handoff.md) | 2.3.0 |
| 034 | [PyPI: a declared wheel matrix](./done/034-pypi-declared-wheel-matrix.md) — [handoff](./handoffs/034-pypi-declared-wheel-matrix/implementation-handoff.md) · [034b](./handoffs/034-pypi-declared-wheel-matrix/followup-034b.md) | 2.3.0 |
| 025 | [Markdown output-validity harness](./done/025-output-validity-harness.md) — [handoff](./handoffs/025-output-validity-harness/implementation-handoff.md) · [025c](./handoffs/025-output-validity-harness/addendum-025c.md) · corpus slice `025b` unscheduled | 2.3.0 |
| 024 | [Inline composition: the output sink](./done/024-inline-composition-output-sink.md) — [handoff](./handoffs/024-inline-composition-output-sink/implementation-handoff.md) · [024b](./handoffs/024-inline-composition-output-sink/addendum-024b.md) · [024c](./handoffs/024-inline-composition-output-sink/addendum-024c.md) · [024d](./handoffs/024-inline-composition-output-sink/addendum-024d.md) · [024e](./handoffs/024-inline-composition-output-sink/addendum-024e.md) | 2.3.0 |
| 028 | [Inline elements around block content; emphasis negated by its own style](./done/028-emphasis-around-block-content.md) — [handoff](./handoffs/028-emphasis-around-block-content/implementation-handoff.md) · [028b](./handoffs/028-emphasis-around-block-content/addendum-028b.md) | 2.3.0 |
| 035 | [Block structure inside containers](./done/035-block-structure-inside-containers.md) — [handoff](./handoffs/035-block-structure-inside-containers/implementation-handoff.md) | 2.3.0 |
| 010 | [Escaping and text round-trip](./done/010-escaping-and-text-round-trip.md) — [handoff](./handoffs/010-escaping-and-text-round-trip/implementation-handoff.md) · closed M3 | 2.3.0 |
| 036 | [Whitespace and separators at block boundaries](./done/036-whitespace-at-block-boundaries.md) — [handoff](./handoffs/036-whitespace-at-block-boundaries/implementation-handoff.md) · [036b](./handoffs/036-whitespace-at-block-boundaries/slice-b-handoff.md) · [036d](./handoffs/036-whitespace-at-block-boundaries/slice-d-handoff.md) | 2.4.0 |
| 038 | [Marker lines colliding with other CommonMark constructs](./done/038-marker-line-collisions.md) — [handoff](./handoffs/038-marker-line-collisions/implementation-handoff.md) | 2.4.0 |
| 008 | [GFM table support](./done/008-gfm-table-support.md) — [008a](./handoffs/008-gfm-table-support/slice-a-handoff.md) · [008b](./handoffs/008-gfm-table-support/slice-b-handoff.md) | 2.4.0 |
| 037 | [Emphasis emission fidelity: empty and nested](./done/037-emphasis-emission-fidelity.md) — [handoff](./handoffs/037-emphasis-emission-fidelity/implementation-handoff.md) | 2.4.0 |
| 009 | [Element coverage extension](./done/009-element-coverage-extension.md) — [handoff](./handoffs/009-element-coverage-extension/implementation-handoff.md) | 2.4.0 |
| 012 | [Benchmark hardening and regenerating the published claims](./done/012-benchmark-hardening.md) — [handoff](./handoffs/012-benchmark-hardening/implementation-handoff.md) | 2.4.0 |

## Archive

None.

## Reserved numbers

These numbers are allocated in the roadmap but not yet drafted. Each is written
at the start of its milestone, so its design reflects what the preceding
milestone actually shipped. Numbers are permanent and are never reused.

| ID | Title | Milestone |
|----|-------|-----------|
| 011 | Robustness: fuzzing + I/O error paths | M4 |
| 013 | Internal comment migration to English | M4 |

## Maintaining this index

Update this file in the same commit that moves an RFC between folders. Before
moving one, run `grep -rl 'NNN-slug.md' rfcs/` and fix inbound references in the
same commit.
