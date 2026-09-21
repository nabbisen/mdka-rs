# Implementation handoff — RFC 032 · Gates report every failure, and execute Python and TypeScript examples

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/done/032-gates-report-everything-and-execute-examples.md`](../../done/032-gates-report-everything-and-execute-examples.md) — **Accepted 2026-09-16 by the owner**, unsplit
**Milestone.** M3. **Order: 030 ✅ → 031 ✅ → 032 → 025 → 024 → 028.**
**Size.** Small–medium. CI-only; no `src/` change.

---

## 0. Preconditions

None. RFC 031 and 031b are approved. **This handoff is an instruction to start.**

## 1. Why before RFC 025

RFC 025 is a harness whose purpose is to show many failures at once. Today
`ci.yaml`'s `cargo test` stops at the first failing test binary, so the harness's
first CI run would show one binary's failures and hide the rest. Part A is what
makes RFC 025's output readable.

## Part A — report every failure

### A.1 Changes

Re-derived from `.github/workflows/`:

| Location | Now | Add |
|---|---|---|
| `ci.yaml:53` | `cargo test --workspace --all-features --locked` | `--no-fail-fast` |
| `ci.yaml:56` | `cargo build --workspace --locked` | `--keep-going` |
| `ci.yaml:50` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `--keep-going` — **see A.2** |
| `crates-package-gate.yaml` | `cargo package --workspace --locked` | `--keep-going` (listed in `cargo package --help`) |

`docs-example-gate` already has it (RFC 031).

Each flag goes **before** any `--`. Add a one-line comment at each saying why,
referencing RFC 032 — as with 030, the comment is what stops a tidy-up removing
it.

### A.2 Clippy — run it, don't read the help

`cargo clippy --help` prints Clippy's own usage and **does not list
`--keep-going`**. Clippy forwards unrecognised options to `cargo check`, which
does support it — but that is reasoning, not evidence. Establish it by A.3's
method. If it does not work, leave clippy unchanged and say so; do not remove
`-D warnings` or restructure the step to force it.

### A.3 Prove each flag changes the output

A flag nobody has seen change the output is decoration. In a throwaway worktree
or branch, **never on `main`**:

| Flag | Break | Expect without | Expect with |
|---|---|---|---|
| `cargo test --no-fail-fast` | one failing `#[test]` in **two different test binaries** (e.g. `tests/compat.rs` and `src/utils/tests.rs`) | one binary's failure reported, then stop | **both** reported |
| `cargo build --keep-going` | compile errors in two independent targets | first only | both |
| `cargo clippy --keep-going` | a lint error in two independent crates | first only | both — or "flag has no effect", stated |
| `cargo package --keep-going` | two crates broken as in RFC 030 §3.2 | first only | both |

**Force `-j1`** for the build-type rows — RFC 031 showed the "without" case
reports more failures on a many-core machine, which can make the flag look
ineffective when it is not. Captures with `EXIT:` inside the file.

## Part B — execute Python and TypeScript examples

### B.1 Current state — re-derived

`python3 .github/workflows/scripts/check-docs-examples.py --list`: 33 runnable of
72. Of those, **9 Python** (8 on `usage-python.md`, 1 in `README.md`) and **1
TypeScript** (`usage-nodejs.md:166`).

| Path | Today |
|---|---|
| Python | `py_compile`, AST symbol resolution, kwargs bound to installed signatures — **never runs** |
| TS | `npx … tsc --noEmit` against `index.d.ts` — **never runs** |

### B.2 Python

Execute each example with the venv interpreter the job already builds
(`--python`), in its own temp directory, with a timeout — mirror `check_js`.

**Fixtures — re-derived, and one trap.** Files the Python examples name:

| Block | Reads |
|---|---|
| `usage-python.md:101` | `page.html` |
| `usage-python.md:122` | `a.html`, `b.html`, `c.html` |
| `usage-python.md:137` | **`missing.html`** |

⚠ **`missing.html` must not be created.** That block is the *Error Handling*
example: it reads a file that does not exist and catches `mdka.MdkaError`. Build
the fixture set by "every filename any example mentions" and you create it — the
example then converts successfully, never enters `except`, exits 0, and passes
**without exercising the thing it documents.**

So, for the fixture set: list the files explicitly, as `JS_FIXTURES` does, and
say in a comment why `missing.html` is absent.

And because exit 0 is also what a *broken* error path could produce, **assert
that this example printed `Conversion failed:`**. That is one targeted output
check, not a general output-matching mechanism — do not build the latter; it is
out of scope (RFC 032 §4 lists it as a limit).

**The JS "unknown file fails loudly" guarantee does not fully transfer.** In
Python an unknown read raises `MdkaError`; uncaught, that is a loud exit 1, but
an example that catches it will exit 0. State that difference in the
`check_python` docstring rather than claiming parity with `check_js`.

### B.3 TypeScript

After `tsc --noEmit` passes, emit JS and **execute it** against the built binding,
sandboxed as in `check_js`.

⚠ **This may surface a real consumer defect, not a gate problem.** The example
uses ESM syntax — `import { htmlToMarkdown, … } from 'mdka'` — against a
**CommonJS** binding (`index.js`). Node's ESM loader resolves named imports from
CommonJS by static analysis of the file, and that can fail with *"does not
provide an export named …"*.

If execution fails that way:

1. **Do not adjust the gate's emit settings until it passes.** That is the RFC 031
   §6 failure — teaching the gate to forgive.
2. Reproduce it the way a consumer would: a fresh directory, `npm install mdka`
   **from the registry**, the example saved as `.mts` or with `"type": "module"`,
   compiled with `tsc`, run with `node`.
3. If the consumer reproduction fails too, **stop and raise it** — that is a
   finding about the published package, and a bigger one than this RFC.
4. If only the gate fails, the gate is not modelling the consumer; fix the model
   and say what was wrong.

Note also: `JsConversionOptions` and `ConvertResult` are types imported in a value
`import`. `tsc` elides them on emit under default settings; under
`verbatimModuleSyntax` it would not, and the runtime import would fail. Use the
settings a consumer following the page would plausibly have, **state them**, and
say what a different, also-plausible setting would do.

### B.4 Prove each new path red

For each, a deliberately broken example that the **current** check passes and the
new execution catches:

- Python: syntactically valid, real symbols, correct kwargs — raises at runtime
- Python error path: `missing.html` example with the `except` branch changed so it
  no longer prints `Conversion failed:` → caught by the output assertion
- TS: type-checks, throws at load or run

Captures: current check EXIT 0, new check non-zero.

## Part C — §4 statements

Each execution path's docstring states what it still substitutes:

- runs the **locally built** binding / wheel, not the published package — covered
  by `npm install gate` / `pypi wheel gate`, named;
- fixtures are one-line files;
- exit 0 is the pass condition (plus the one targeted Python output check).

## Out of scope

- Output matching against documented results, generally.
- `src/`.
- Changing any example's content, **unless** execution shows it is wrong — then
  fix it, and record it as a finding in the review request, since it is a defect
  the consumer pass missed.

## Acceptance checklist

- [ ] A.1 flags added with comments; clippy per A.2
- [ ] A.3 each flag shown changing the output, `-j1` for build-type rows, captures with `EXIT:`
- [ ] B.2 Python executed; explicit fixture list without `missing.html`; `Conversion failed:` asserted; docstring states the non-parity with JS
- [ ] B.3 TS emitted and executed; settings stated; any ESM-named-import failure handled per B.3 steps 1–4
- [ ] B.4 each new path observed red where the old check was green
- [ ] Part C statements in each docstring
- [ ] Docs gate runnable count unchanged (33/72) unless an example was fixed — then explain
- [ ] All six of our workflows green; tests/fmt/clippy clean

## Report back

`.git-exclude/review-request/032-gates-report-everything/README.md`, evidence
under `evidence/`, exit codes inside the files.
