# Developer Handoff — RFC 001 · CI quality gates

**Governing RFC.** [RFC 001](../../proposed/001-ci-quality-gates.md) — Proposed
**Milestone.** M1 · Trustworthy baseline → `2.1.7`
**Position in M1.** First. Nothing else in M1 lands until this is green.
**Prepared.** 2026-08-02

This Handoff directs execution of RFC 001. It does not redefine it. If
implementation uncovers a conflict with the RFC, stop and raise it — patch the
RFC first, then this document.

---

## 1. Purpose

Add a blocking CI workflow that runs format, lint, build, and test on every push
and pull request, and fix the pre-existing failures that currently prevent such
a workflow from passing.

## 2. Background

The repository has four workflows: `docs.yaml` and three release workflows.
None runs on push or pull request. Nothing verifies the codebase between
releases.

Measured on this repository at 2.1.6 on 2026-08-02:

- `cargo test --workspace` — **74 passed, 0 failed**. Clean.
- `cargo fmt --check` — **fails**, one diff.
- `cargo clippy --workspace --all-targets --all-features` — **10 warnings**
  across five distinct sites. Would fail under `-D warnings`.
- Node and Python binding suites — never executed by automation.

Note the handoff bundle in `.git-exclude/specs/` claims 84 tests. That figure is
stale. **74 is the baseline.**

## 3. Applicable requirements

From RFC 001 §Goals: format, lint, build and test run on every push and PR;
lints blocking at `-D warnings`; MSRV corrected to 1.88 and then verified
rather than asserted (amended 2026-08-02, see §6 Slice 1b); both
binding suites run automatically.

## 4. Change scope

You may change:

| Path | Change |
|---|---|
| `.github/workflows/ci.yaml` | New file |
| `Cargo.toml` | Add `rust-version = "1.88"` — Slice 1b, amended |
| `docs/src/getting-started/installation.md` | MSRV line only — Slice 1b, amended |
| `examples/quick_bench.rs` | rustfmt fix + `useless_format` fix |
| `examples/quick_compare.rs` | `useless_format` fix |
| `benches/parallel.rs` | `manual_repeat_n` fix |
| `python/src/lib.rs` | Two `#[allow]` attributes + rationale comments only |
| `README.md` | Add one CI badge to the existing badge block |

## 5. Non-change scope — do not touch

- `src/`, `cli/`, `node/src/`, and all of `tests/`. No library, CLI, or binding
  logic changes under this RFC.
- The four existing workflows and `.github/workflows/scripts/install-rust.sh`.
- Any `Cargo.toml`, `package.json`, or `pyproject.toml`. No dependency or
  version changes.
- `tests/utils/` — RFC 004 owns its removal. Leave it exactly as it is, even
  though it is dead. Removing it here would confound this RFC's evidence.
- All documentation under `docs/` — RFC 003 owns it. **One exception:** the
  single MSRV line at `docs/src/getting-started/installation.md:14`, per
  Slice 1b. Nothing else in `docs/`.
- Japanese comments anywhere — RFC 007 and RFC 013 own them. You will encounter
  them in every file you open. Leave them.
- The PyO3 function signatures at `python/src/lib.rs:188` and `:225`. Add the
  suppression; do not restructure the arguments. Those argument lists *are* the
  published Python keyword API.

## 6. Required implementation

Land as three reviewable slices in this order. Slice 1 must be green locally
before slice 3 makes the workflow blocking.

### Slice 1 — Fix pre-existing lint and format failures

Five sites, all verified at 2026-08-02:

| Location | Lint | Fix |
|---|---|---|
| `examples/quick_bench.rs:62` | rustfmt indentation | `cargo fmt` |
| `examples/quick_bench.rs:38:18` | `useless_format` | `"-".repeat(122)` |
| `examples/quick_compare.rs:52:18` | `useless_format` | `"-".repeat(122)` |
| `benches/parallel.rs:34:29` | `manual_repeat_n` | `std::iter::repeat_n(html, REPEAT)` |
| `python/src/lib.rs:188:1`, `:225:1` | `too_many_arguments` (9/7) | `#[allow(clippy::too_many_arguments)]` + rationale comment |

Clippy's own suggestion for the `useless_format` sites is
`"-".repeat(122).to_string()`. **Do not paste that** — `repeat` already returns
`String`, so `.to_string()` is a redundant clone. Write `"-".repeat(122)`.

The rationale comment on each `#[allow]` must state *why* the lint is
suppressed — that the argument list is the PyO3 keyword-argument API and
restructuring it would break the published Python surface. A bare `#[allow]`
with no reason will be rejected at review.

Per project rules: finish all edits, then run `cargo fmt` **once**, then run
checks. Do not review the formatted output.

### Slice 2 — Add the workflow

Create `.github/workflows/ci.yaml`. Triggers: `push` and `pull_request` against
`main`, plus `workflow_dispatch`. Set `permissions: contents: read` at workflow
level. Set `defaults.run.shell: bash`, matching the existing workflows.

Four independent jobs, so a binding failure cannot mask a library failure:

**`rust`** — ubuntu-latest
```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --locked
```

**`msrv`** — ubuntu-latest, toolchain pinned to **1.88**
```
cargo check --workspace --locked
```

> **AMENDED 2026-08-02 — the pin is 1.88, not 1.85.**
>
> You escalated correctly that 1.85 could not hold. Confirmed and decided by the
> project owner: the published MSRV was never true and is corrected to 1.88.
> `scraper` uses let-chains in its own source — true of both `0.26.0`
> (2.0.0–2.1.5) and `0.27.0` (2.1.6), so this is not a regression from the
> recent dependency bump. `cargo check --workspace` passes at 1.88 including
> `mdka-node`; no narrowing of scope is needed.
>
> The MSRV correction now lands **inside this RFC**, as Slice 1b below. See
> RFC 001 §MSRV correction for the full rationale.

`--all-targets` is deliberately omitted. Bench dev-dependencies (`htmd`,
`html-to-markdown-rs`, `html2text`) require a newer toolchain and are advisory
only.

`--locked` is required: an MSRV job must verify the *locked* dependency set, or
it and the `rust` job can silently disagree about what was checked. Use
`--locked` in the `rust` job's `cargo test` too, for the same reason.

If 1.88 also fails to resolve, **stop and report.** Do not raise it further.

### Slice 1b — MSRV correction (new, amended in)

Land these together with the pin change, so the gate and the claim it verifies
cannot disagree:

| Location | Change |
|---|---|
| `Cargo.toml` | Add `rust-version = "1.88"` — new field only, no dependency change |
| `docs/src/getting-started/installation.md:14` | `**Minimum Supported Rust Version:** 1.85 (2024 Edition)` → `1.88` |

This is a **narrow, explicit exception** to the "do not touch `docs/`" rule in
§5. It covers that one line and nothing else. It is an MSRV value, not
documentation reconciliation, and RFC 003 does not cover `installation.md`.

`installation.md` is the only in-repo MSRV statement; `README.md`, the rest of
`docs/src/`, and every other manifest were audited and state none.

**Do not edit `.git-exclude/specs/`** (the handoff bundle's `decision-log.md`
DEC-004 or `requirements.md` CC-02). Your request proposed this; it is wrong.
Those are a received inbound artifact and a historical record of what was handed
over — correcting them would falsify the record. The correction is recorded in
RFC 001 and will appear in `CHANGELOG.md` under RFC 002.

**`node`** — ubuntu-latest
```
actions/setup-node@v6, node-version 24, cache: npm,
  cache-dependency-path: node/package-lock.json
npm ci                (working-directory: node)
npm run build         (working-directory: node)
node test.js          (working-directory: node)
git diff --exit-code node/index.d.ts
```

The final check is deliberate. `npm run build` regenerates `index.d.ts` from the
Rust doc comments in `node/src/lib.rs`. That file is both committed and
published to npm, so a drift check keeps the shipped type definitions honest.
It also directly serves RFC 007, which edits those doc comments.

**`python`** — ubuntu-latest
```
actions/setup-python@v6, python-version 3.x
cp -f README.md python/          # REQUIRED — see below
uv venv && uv pip install maturin pytest
maturin develop                  (working-directory: python)
python -m pytest test_mdka.py    (working-directory: python)
```

The `cp -f README.md python/` step is **not optional**. `python/pyproject.toml`
declares `readme = "README.md"`, but the README lives at the repository root.
All four maturin invocations in `release-pypi.yaml` stage it first, under the
step name "Stage README for maturin". Omitting it fails the build. This exact
omission was a shipped bug once already — see commit `3da9ce3`, "fix readme was
missing in pypi".

Use `uv` rather than bare `pip`; `release-pypi.yaml` already standardises on it,
and `python/.venv/` is already gitignored.

Cache cargo state per job with `actions/cache@v5`, keyed on
`hashFiles('**/Cargo.lock')`, matching the pattern in the existing workflows.
Use the action versions already in use in this repository: `checkout@v6`,
`setup-node@v6`, `setup-python@v6`, `cache@v5`.

### Slice 3 — Make it blocking and add the badge

Confirm all four jobs green on the branch, then add a CI status badge to the
existing badge block in `README.md`. Place it with the other workflow badges,
not as a new section — the project rules require the README stay concise.

## 7. Required tests

No new test code. This RFC delivers the gate, not new coverage.

`cargo test --workspace` must still report **74 passed, 0 failed**. If the count
changes, stop and explain why before proceeding — under this RFC's scope,
nothing should alter it.

## 8. Verification you must perform manually

**Confirm the Node runner actually fails the job.** `node/test.js` is a
hand-rolled runner, not a standard framework. It tracks its own `passed`/`failed`
counters and calls `process.exit(1)` at three points (lines 189, 280, 363),
across more than one async IIFE sharing those counters. Do not assume the exit
code is correct.

Verify empirically: temporarily break one assertion, run `node test.js`, confirm
`echo $?` is non-zero, then revert. Record the result in the review request. A
green CI job proves nothing if the runner cannot report failure.

`python/test_mdka.py` is standard pytest (71 test functions) and needs no such
check.

## 9. Required documentation updates

Only the README badge. Nothing else. `docs/` belongs to RFC 003.

## 10. Compatibility constraints

Zero behaviour change. No public API, CLI flag, binding signature, or output
byte may differ. The `python/src/lib.rs` edit is attributes and comments only.

Verify before submitting: `git diff` on `python/src/lib.rs` shows only
`#[allow(...)]` lines and comment lines.

## 11. Security constraints

- The workflow requires no secrets. Do not add any.
- Set `permissions: contents: read` at workflow level. Do not grant write.
- Do not add `pull_request_target`, or any trigger that runs untrusted code with
  repository credentials.
- Publishing tokens stay confined to the release workflows.

## 12. Prohibited shortcuts

- No crate-level `#![allow(...)]` to silence clippy.
- No `continue-on-error` to make a job green.
- No disabling, skipping, or `#[ignore]`-ing a test.
- No raising the MSRV pin beyond the owner-approved 1.88 to make the job pass.
- No restructuring the PyO3 signatures.
- No touching `tests/utils/`, `docs/`, or Japanese comments.

If any of these looks necessary, that is an escalation, not a decision you make.

## 13. Known risks

| Risk | If it happens |
|---|---|
| MSRV job fails to resolve on 1.88 | Stop and report. Do not bump the pin further. |
| `git diff --exit-code node/index.d.ts` fails on first run | Means the committed file already drifted from source. Report the diff; do not silently commit the regenerated file without flagging it. |
| Binding jobs slow or flaky | Report it. Do not make the job non-blocking. |
| `maturin develop` needs an active venv | Expected — that is why `uv venv` precedes it. |

## 13a. Review findings to address (amended in 2026-08-02)

From `.git-exclude/reviewed/001-ci-quality-gates/review-report.md`. None block
Slice 1, which is approved and landable as-is.

| ID | Severity | Required action |
|---|---|---|
| F-01 | Medium | Add `--locked` to the MSRV `cargo check`, and to the `rust` job's `cargo test`. An MSRV job must verify the locked dependency set; mixed locking within one job is a latent inconsistency. |
| F-02 | Medium | Cache keys collide across toolchains. All four jobs share `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`, but `msrv` compiles with a different rustc than `rust`, so their `target` artifacts are mutually useless. GitHub's cache is immutable per key — first writer wins and the rest restore a cache they discard. Scope the key per toolchain, or scope the cached `path` per job. |
| F-03 | Low | Add a `concurrency` group keyed on the ref with `cancel-in-progress: true`, matching `docs.yaml`. |
| F-04 | Low | Add `timeout-minutes` per job. A hung `npm run build` or `maturin develop` would otherwise hold a runner for the 6-hour default. |

Confirmed correct in the draft and not to be changed: the `cp -f README.md
python/` step with its commit reference; the `index.d.ts` drift check and its
placement after `npm run build`; `permissions: contents: read` with no secrets;
`--all-targets` omitted from the MSRV job; action versions matching repository
convention.

## 13b. Still outstanding from the original handoff

§8's `node/test.js` exit-code experiment was not reported in the submission. It
is independent of the MSRV question and remains **required before Slice 3 is
accepted**: break one assertion, run `node test.js`, confirm `echo $?` is
non-zero, revert, and report the result.

A green CI job proves nothing if the runner cannot signal failure.

## 13c. Evidence standard

The submitted `msrv-1.88-lib-only.log` reported `Finished in 0.04s` — a cache
hit, not a compile. The conclusion held when re-verified in a clean target
directory, but the log did not establish it.

For any future "does it build" claim, evidence must show actual compilation.
Use a fresh `CARGO_TARGET_DIR`, or include the `Compiling`/`Checking` lines.

## 14. Required evidence

Attach to the review request:

1. Link to a green CI run showing all four jobs.
2. `cargo fmt --check` — exit 0.
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit 0.
4. `cargo test --workspace` — full output, pass count stated.
5. `git diff python/src/lib.rs` — proving attributes and comments only.
6. The Node exit-code experiment: the broken assertion used, the observed
   non-zero exit code, and confirmation of revert.

## 15. Acceptance checklist

- [ ] `.github/workflows/ci.yaml` exists; triggers on push and PR to `main`
- [ ] All four jobs (`rust`, `msrv`, `node`, `python`) pass
- [ ] `-D warnings` enforced in the `rust` job
- [ ] MSRV job pins 1.88, uses `--locked`, and passes
- [ ] `Cargo.toml` carries `rust-version = "1.88"`
- [ ] `docs/src/getting-started/installation.md` states 1.88; nothing else in `docs/` touched
- [ ] `.git-exclude/specs/` untouched
- [ ] Cache keys are toolchain-scoped (F-02); `concurrency` group and `timeout-minutes` set (F-03/F-04)
- [ ] Zero fmt diffs, zero clippy warnings workspace-wide
- [ ] `cargo test --workspace` reports 74 passed, 0 failed
- [ ] `cp -f README.md python/` present in the python job
- [ ] `index.d.ts` drift check present in the node job
- [ ] Node runner proven to exit non-zero on failure
- [ ] CI badge added to the existing README badge block
- [ ] Workflow permissions are `contents: read`, no secrets
- [ ] No file outside §4 modified

## 16. Required review-request format

1. Implementation summary
2. Addressed requirements (RFC 001 goals, by number)
3. Changed files — complete list
4. Important implementation decisions
5. Differences from RFC 001, if any, and why
6. Executed tests and results
7. Build, format, and lint results
8. Evidence per §14
9. Unresolved issues
10. Known limitations
11. Requested review focus

## 17. Escalate rather than decide

Stop and raise it if you find: the MSRV pin cannot hold; `index.d.ts` has
already drifted; a binding suite fails for a reason predating this change; a
clippy fix would require a signature or behaviour change; or the 74-test
baseline moves.

None of these is yours to resolve by adjusting the gate.
