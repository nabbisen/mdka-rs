# `corpus/` — captured real-world HTML

This directory is for **real clipboard or page captures only** (RFC 025 slice
`025b`), each with metadata saying where and how it was captured. It is empty
until those captures exist.

Hand-written inputs do not belong here, however realistic:

- runner fixtures written for this harness: `../runner_fixtures/`
- the runner's red-path proof: `../corpus_proof/`
- reproductions from field reports: `../field_reports.rs`

Nothing runs over this directory yet. Before captures land here, `025b` adds
a per-file record of expected violations with owners, with the same strict
semantics as `known_defect`: a listed violation that is absent fails, and an
unlisted one fails.
