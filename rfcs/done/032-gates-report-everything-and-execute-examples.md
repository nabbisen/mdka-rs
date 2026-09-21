# RFC 032 — Gates report every failure, and execute Python and TypeScript examples

**Status.** Implemented (2.3.0)
**Author.** Architect
**Created.** 2026-09-16
**Milestone.** M3 — recommended **before RFC 025** (§5)
**Source.** RFC 031 review — `.git-exclude/reviewed/031-docs-gate-must-model-mdbook/README.md` §7
**Related.** RFC 026 (gates), RFC 031 (§6 rule: a gate models the artifact the consumer meets)

---

## 1. Summary

Two small control repairs, both surfaced while implementing RFC 031:

1. **Under-reporting.** A multi-target `cargo` invocation stops at its first
   failure, so the number of failures reported depends on scheduling, not on
   the code. RFC 031 saw five failures locally and one on CI for the same
   commit. `ci.yaml`'s `cargo test` has the same shape.
2. **Python and TypeScript examples are never run.** RFC 031 made the JS path
   execute examples after `node --check` passed one that failed at load. The
   Python and TS paths still stop short of execution — the same substitution.

## 2. Under-reporting

### 2.1 Evidence

RFC 031, `evidence/step1c-failure-count-depended-on-cores.txt`, same documents:

```
local, 32 cores                     5 failing reported
CI runner                           1 failing reported
local, forced -j1                   1 failing reported
local, -j1, --keep-going            5 failing reported
```

The gate was red in every case — so this is not a false pass. The cost is
**round trips**: fix the one reported, push, go red on the next, repeat.

### 2.2 Where else it applies

Re-derived from `.github/workflows/`:

| Location | Invocation | Flag |
|---|---|---|
| `ci.yaml:53` | `cargo test --workspace --all-features --locked` | **`--no-fail-fast`** |
| `crates-package-gate.yaml` | `cargo package --workspace --locked` | `--keep-going` (supported by `cargo package`) |
| `ci.yaml:50` | `cargo clippy --workspace --all-targets …` | `--keep-going` if clippy passes it through — **check, do not assume** |
| `ci.yaml:56` | `cargo build --workspace --locked` | `--keep-going` |

`docs-example-gate` already has `--keep-going` (RFC 031).

### 2.3 Why now

**`ci.yaml:53` is the one that matters for M3.** RFC 025 is an output-validity
harness: its purpose is to produce many failures at once, across test binaries,
so the engine RFCs after it can see the whole defect surface. Without
`--no-fail-fast`, the first failing test binary hides the rest.

## 3. Execute Python and TypeScript examples

### 3.1 What each path does today

Re-derived from `check-docs-examples.py`:

| Path | Checks | Does not |
|---|---|---|
| Python | `py_compile`; AST symbol resolution against the installed wheel; kwargs bound against installed signatures (RFC 029) | **run the example** |
| TypeScript | `tsc --noEmit` against the shipped `index.d.ts` | **run the emitted JS** |
| JavaScript | `node --check`, then **executes** against the built binding (RFC 031) | — |

A Python example that is valid syntax, names real symbols and binds its kwargs,
but fails at runtime, passes. A TS example with a D2-class load or runtime
failure passes.

### 3.2 Change

Mirror RFC 031's JS approach:

- **Python** — run each example with the venv interpreter already built in the
  job, in its own temporary directory, with a fixture set covering every file any
  Python example reads, and a timeout.
- **TypeScript** — after `tsc --noEmit` passes, emit and **execute** the JS
  against the built binding, same sandboxing as `check_js`.

Same rule as RFC 031's JS fixtures: an example reading a file not in the fixture
set must **fail loudly**, never pass.

### 3.3 ~~Not urgent~~ — corrected at review

> **Corrected 2026-09-16.** This section originally said the 2.2.3 consumer pass
> "executed every Python and TS example and all passed", and on that basis called
> §3 a fix for nothing currently broken. **The report does not say that** — it
> says *"Python — all claims passed"*, meaning the behavioural claims. The
> architect paraphrased it into something stronger.
>
> Implementing §3 found **two live defects** the old gate passed:
> `usage-python.md:28` raised `NameError`, and `usage-nodejs.md:166` failed TS1484
> under `tsc --init` settings.

## 4. §6 statement required

Per RFC 031 §6, each path must state what it still substitutes. At minimum:

- all three execute against the **locally built** binding or wheel, not the
  published package — covered by the `npm install gate` and `pypi wheel gate`;
- fixtures are one-line files, so behaviour on real-sized input is not exercised;
- exit code 0 is the pass condition — output that differs from what the page
  claims still passes.

## 5. Sequencing

**Before RFC 025.** §2 is minutes of work and directly serves RFC 025. §3 is
small because the venv and binding already exist in the job.

If the owner prefers not to delay RFC 025, **split**: land §2 now as a
one-commit slice, and move §3 after RFC 028.

## 6. Acceptance criteria

- [ ] `ci.yaml` `cargo test` uses `--no-fail-fast`
- [ ] `--keep-going` on `cargo package --workspace` and `cargo build`; clippy checked and either added or the reason it cannot be stated
- [ ] **Each flag observed doing its job** — two deliberately failing targets in a throwaway branch or worktree, both reported. A flag nobody has seen change the output is decoration.
- [ ] Python examples executed; fixture set; timeout; unknown file read fails loudly
- [ ] TS examples emitted and executed against the built binding
- [ ] **Each new execution path observed red** on a deliberately broken example that its previous check passed — e.g. a Python call that raises at runtime; a TS example that type-checks and throws at load
- [ ] §4 substitutions stated in each path's docstring
