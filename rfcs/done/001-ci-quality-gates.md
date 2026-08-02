# RFC 001 — CI quality gates

**Status.** Implemented (2.1.7)
**Tracks.** M1 · Trustworthy baseline. Establishes the automated gate that all
subsequent RFCs land through.
**Touches.** A new `.github/workflows/ci.yaml`; pre-existing lint and format
failures in `examples/quick_bench.rs`, `benches/parallel.rs`, and
`python/src/lib.rs`; a CI badge in `README.md`.

## Summary

The repository has no workflow that runs tests, formatting, or lints. The only
existing workflows build documentation and publish release artifacts. This RFC
adds a blocking CI workflow and fixes the pre-existing failures that currently
prevent one from passing.

## Motivation

`.github/workflows/` contains `docs.yaml`, `release-executable.yaml`,
`release-npm.yaml`, and `release-pypi.yaml`. Nothing runs on push or pull
request. Consequences observed at 2.1.6:

- `cargo fmt --check` fails on `examples/quick_bench.rs:62` and has evidently
  failed for some time.
- `cargo clippy --workspace --all-targets --all-features` emits 10 warnings.
- The Node.js (`node/test.js`) and Python (`python/test_mdka.py`) suites have
  never been executed by automation.
- Six `ConversionOptions` fields became inert without any gate noticing —
  precisely the class of regression a test gate exists to catch.

The incoming handoff bundle flagged this as RISK-001 and left it open.

## Goals

- Every push and pull request to `main` runs format, lint, build, and test.
- Lints are blocking: `-D warnings`.
- The declared MSRV is **corrected to 1.88** and then verified rather than
  asserted. See §MSRV correction.
- The Node.js and Python binding suites run automatically.

## Non-goals

- Numeric code-coverage thresholds. Coverage stays qualitative for now.
- Changes to the three release workflows or the docs workflow.
- A full cross-platform test matrix. Binding *builds* already fan out across
  platforms in the release workflows; this RFC tests on Linux only and records
  broader coverage as a follow-up.
- Fuzzing — owned by RFC 011.

## Proposed design

### Workflow structure

A single `.github/workflows/ci.yaml`, triggered on `push` and `pull_request`
against `main`, plus `workflow_dispatch`. Four independent jobs so a binding
failure does not mask a library failure:

| Job | Steps | Blocking |
|---|---|---|
| `rust` | `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace`; `cargo build --workspace --locked` | yes |
| `msrv` | Pin toolchain 1.88, `cargo check --workspace --locked` | yes |
| `node` | `npm ci`; `npm run build`; `node test.js` | yes |
| `python` | `uv` + `maturin develop`; `python -m pytest test_mdka.py` | yes |

Cargo registry and build artefacts are cached by job. The `python` job follows
the `uv` setup already used by `release-pypi.yaml` rather than introducing a
second Python toolchain convention.

### MSRV correction

**Added by amendment, 2026-08-02, on project-owner decision.**

The published MSRV of 1.85 has never been true for the 2.x line. Verified:

| Check | rustc 1.85 | rustc 1.88 |
|---|---|---|
| `cargo check -p mdka@2.1.6` | fails, `E0658` | passes |
| `cargo check --workspace` | fails | passes, incl. `mdka-node` |
| `cargo check -p mdka@2.1.5` (tag `2.1.5`, `scraper 0.26.0`) | fails, `E0658` | — |

Cause: `scraper` uses let-chains (`if let … && let …`) in its own source
(`src/element_ref/mod.rs`, `src/html/mod.rs`), stabilized in Rust **1.88**. This
holds for both `scraper 0.26.0` (used 2.0.0–2.1.5) and `0.27.0` (2.1.6), so it
is not a regression introduced by the `0.26 → 0.27` bump in commit `025891d`.

`scraper` declares no `rust-version`, so Cargo's MSRV-aware resolver cannot
reject it — the failure surfaces only at compile time. `Cargo.toml` likewise has
no `rust-version` field, which is why nothing ever caught this: the 1.85 figure
existed solely as prose ("edition 2024 implies ≥1.85") and was never built.

**Owner decision: correct the published MSRV to 1.88.** This changes no
capability — anyone on 1.85–1.87 has never been able to build mdka 2.x. The
alternative (downgrading `scraper` past 0.26 to genuinely support 1.85) was
rejected as materially more expensive than it first appeared.

Required changes, landing with the gate so the two cannot disagree:

| Location | Change |
|---|---|
| `Cargo.toml` | Add `rust-version = "1.88"` (new field; no dependency change) |
| `docs/src/getting-started/installation.md:14` | `1.85` → `1.88` |
| `.github/workflows/ci.yaml` | Pin `1.85` → `1.88` |

`installation.md` is the only in-repo MSRV statement — `README.md`, the rest of
`docs/src/`, and every other manifest were audited and state none.

Do **not** edit the handoff bundle under `.git-exclude/specs/` (`decision-log.md`
DEC-004, `requirements.md` CC-02). It is a received inbound artifact and a
historical record of what was handed over; correcting it would falsify the
record. The correction belongs here and in `CHANGELOG.md` (RFC 002).

### Pre-existing failures to fix

These must be resolved in the same change, or the workflow cannot be made
blocking on first landing:

Five distinct sites, verified 2026-08-02 (clippy reports 10 warnings in total;
the `python/src/lib.rs` pair is counted once per target, lib and lib-test):

| Location | Issue | Required resolution |
|---|---|---|
| `examples/quick_bench.rs:62` | rustfmt diff (indentation) | Run `cargo fmt`; do not hand-edit |
| `benches/parallel.rs:34:29` | `clippy::manual_repeat_n` | Use `std::iter::repeat_n(html, REPEAT)` |
| `examples/quick_bench.rs:38:18` | `clippy::useless_format` | `"-".repeat(122)` — the `format!` wrapper is redundant, and `.to_string()` on an owned `String` is equally redundant |
| `examples/quick_compare.rs:52:18` | `clippy::useless_format` | As above |
| `python/src/lib.rs:188:1`, `:225:1` | `clippy::too_many_arguments` (9/7) | `#[allow(clippy::too_many_arguments)]` with a rationale comment — the argument list *is* the PyO3 keyword-argument API and must not be restructured to satisfy a lint |

Both `useless_format` sites are in `examples/`, not `benches/`. Clippy's
suggested fix (`"-".repeat(122).to_string()`) is itself redundant — `repeat`
already returns `String`. Apply `"-".repeat(122)` and do not paste the
suggestion verbatim.

The `too_many_arguments` suppression is a deliberate, documented exception, not
a convenience. Restructuring those signatures would change the published Python
API and belongs to RFC 006 if it is wanted at all.

### Formatting order

Per project rules: complete all implementation, then run `cargo fmt` once, then
run tests and checks. Do not review the formatted output.

## Compatibility

None. No library, CLI, or binding code changes behaviour. The `python/src/lib.rs`
change is an attribute only.

## Security

The workflow requires no secrets. Permissions are set to `contents: read` at the
workflow level. Publishing credentials remain confined to the release workflows.

## Testing and verification

The gate is its own verification. Evidence required for review:

- A link to a green CI run on the branch.
- Local `cargo fmt --check` exit 0.
- Local `cargo clippy --workspace --all-targets --all-features -- -D warnings` exit 0.
- Local `cargo test --workspace` output, with the pass count stated.

Baseline for comparison, measured 2026-08-01 on this repository at 2.1.6:
**74 tests passed** (12 lib, 18 block_elements, 7 compat, 6 file_conversion,
22 inline_elements, 5 robustness, 4 doc), 0 failed.

Note that the incoming handoff bundle claims 84 tests. That figure is stale and
must not be used as the baseline.

## Acceptance criteria

1. `.github/workflows/ci.yaml` exists and runs on push and pull request to `main`.
2. All four jobs pass on `main`.
3. `-D warnings` is enforced in the `rust` job and the workflow fails if a lint regresses.
4. The MSRV job pins 1.88, runs `--locked`, and passes.
4a. `Cargo.toml` carries `rust-version = "1.88"`.
4b. `docs/src/getting-started/installation.md` states 1.88.
5. Zero `cargo fmt --check` diffs and zero clippy warnings across the workspace.
6. `cargo test --workspace` reports 74 passed, 0 failed — or a higher pass count with the difference explained.
7. A CI status badge is added to `README.md` alongside the existing badges.

## Prohibited shortcuts

- Do not silence clippy with blanket `#![allow]` at crate level.
- Do not mark the workflow `continue-on-error` to make it green.
- Do not disable, skip, or `#[ignore]` any test to achieve a pass.
- Do not restructure the PyO3 signatures under this RFC.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Binding jobs are slow or flaky (native builds) | CI latency, false failures | Cache aggressively; if flakiness appears, report it rather than making the job non-blocking |
| `-D warnings` breaks on a future toolchain's new lints | `main` goes red without a code change | Accepted. Pin the clippy toolchain if it becomes disruptive; treat as a follow-up |
| Bench dev-dependencies require a recent toolchain | `--all-targets` may fail on the MSRV job | The MSRV job runs `cargo check --workspace --locked` without `--all-targets`; benches are advisory and excluded |
| A future `scraper` release raises the floor again, silently | MSRV drifts out of date once more | `rust-version = "1.88"` plus the pinned `msrv` job now make it fail loudly instead of silently. This is the gap that let 1.85 survive untested. |

## Alternatives considered

- **Advisory (non-blocking) CI first, blocking later.** Rejected: the failures
  are known and small, and an advisory gate would not have prevented any of the
  defects found in the 2.1.6 review.
- **Reuse the existing `install-rust.sh` helper for all jobs.** Deferred: that
  script serves the release workflows' versioned-toolchain needs; CI can use
  the standard toolchain action more simply.
