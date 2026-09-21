# Follow-up handoff — RFC 034 · slice `034b`: declare the GIL requirement

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/done/034-pypi-declared-wheel-matrix.md`](../../done/034-pypi-declared-wheel-matrix.md) — §3 correction and addendum
**Review this follows.** `.git-exclude/reviewed/034-pypi-declared-wheel-matrix/README.md` §4
**Size.** Small. One attribute, one comment, two doc sentences, one CHANGELOG line, one toml comment.
**Closes.** RFC 034.

---

## 0. Preconditions

None. **This handoff is an instruction to start.**

## 1. Why

Your §8.1 finding: in PyO3 0.28 an unannotated `#[pymodule]` declares itself
**GIL-free** (`pyo3-macros-backend-0.28.3/src/module.rs:392`). The owner decided
free-threaded CPython is **not supported**, and the installation page now sends
free-threaded users to the sdist. A source build there produces a module that
claims to be safe without the GIL — never reviewed, and resting on a macro default.

Declare the requirement explicitly instead.

## 2. Change

`python/src/lib.rs:329`:

```rust
#[pymodule(gil_used = true)]
fn mdka_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
```

With a comment above it saying: free-threaded CPython is not supported (RFC 034,
owner 2026-09-16); the module has not been reviewed for running without the GIL;
PyO3 0.28's default for an unannotated module is GIL-free, so this is stated
explicitly rather than left to a default that can change between PyO3 versions;
changing it to `false` requires a thread-safety review in its own RFC.

## 3. Prove it — both directions, on a real free-threaded interpreter

You already have CPython 3.14.7t. Build from source on it (abi3 does not apply to
free-threaded builds; PyO3 builds version-specific automatically — confirm what
the build actually produced).

| Build | Expect | Record |
|---|---|---|
| **before** — current `main` source, on 3.14t | `sys._is_gil_enabled()` **False** after import | value, and any warning under `-W default` |
| **after** — with `gil_used = true`, on 3.14t | **True** after import | value, and the warning CPython emits — **quote it; do not assume its wording** |
| after, on regular CPython 3.10 and newest | abi3 wheel still builds; **80 tests pass** against the installed wheel | counts |

If the "after" row does not re-enable the GIL, **stop and report** — the attribute
is not doing what this slice assumes.

## 4. Documentation and records

- `docs/src/getting-started/installation.md`: the free-threaded sentence gains one
  clause — building from source on free-threaded CPython works, but mdka requires
  the GIL, so Python re-enables it on import. **Use the warning you actually
  observed** if you quote it.
- `CHANGELOG.md` `[Unreleased]`: under the existing free-threaded entry — source
  builds on free-threaded CPython now declare that they need the GIL; the 2.2.3
  free-threaded wheels ran without it.
- `python/wheel-matrix.toml` header: *"the gates check the last three against this
  file"* understates it — `floor-manifests` also checks the `abi3-pyXY` feature.
  Correct the sentence.

## 5. Out of scope

- A thread-safety review, and any `gil_used = false` decision.
- Anything else in `python/src/lib.rs`.

## 6. Acceptance checklist

- [ ] `#[pymodule(gil_used = true)]` with the explanatory comment
- [ ] §3 before/after on 3.14t captured, warning quoted as observed; what the ft build produced stated
- [ ] abi3 wheel + 80 tests on 3.10 and newest, against the installed wheel
- [ ] installation.md clause; CHANGELOG entry; toml comment corrected
- [ ] All seven of our workflows green

## 7. Report back

`.git-exclude/review-request/034b-gil-declaration/README.md`, evidence under
`evidence/`, exit codes inside the files.
