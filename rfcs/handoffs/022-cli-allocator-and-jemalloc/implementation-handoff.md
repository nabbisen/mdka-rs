# Developer Handoff — RFC 022 · CLI allocator removal; settle `jemalloc`

**Governing RFC.** [RFC 022](../../proposed/022-cli-allocator-and-jemalloc.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P1
**Prepared.** 2026-08-31

---

## 1. Purpose

The shipped CLI installs a measurement instrument as its global allocator.
Remove it. Then resolve the `jemalloc` feature, which gates dependencies no code
uses.

## 2. The allocator

`cli/src/main.rs:24`:

```rust
#[global_allocator]
static ALLOCATOR: mdka::alloc_counter::CountingAllocator = mdka::alloc_counter::CountingAllocator;
```

Its comment says it does not interfere with library users. True, and beside the
point — it interferes with **CLI users**, who are the ones running bulk
conversion. Every allocation on the hottest path pays for a counter nothing reads
at runtime.

**Measure before and after yourself.** The audit reports up to 2.5× on bulk
conversion from 1 to 32 threads. The architect confirmed the allocator is
installed but **did not reproduce that number**. Report what you actually
measure, whatever it is — including if it is far smaller than claimed. A finding
that shrinks under measurement is still a finding, and reporting it accurately
matters more than confirming the audit.

Use a bulk conversion over enough files for the parallel path to matter, on a
release build, and state the machine's core count.

## 3. `alloc_counter` visibility

Because `cli` reaches it as `mdka::alloc_counter::CountingAllocator`, it is
**public API for semver purposes** — a benchmark-only concern any downstream
crate can depend on.

Once the CLI stops installing it, reduce its visibility. `#[doc(hidden)]` is not
privacy — prefer `pub(crate)` plus a benchmark-local shim, or a feature that is
off by default. If a benchmark target genuinely needs it public, say so with the
reason instead of doing it silently.

Note the change in the CHANGELOG: it is a public-surface reduction, however
unlikely a consumer is.

## 4. The `jemalloc` feature

`Cargo.toml:47` declares it; `grep -rn jemalloc src/ cli/src/` returns nothing.

**Decide, do not preserve.**

- **Delete it** — the default. Nothing uses it and nothing has.
- **Implement it** — a real allocator swap for the CLI, benchmarked against the
  system allocator to show it earns its place.

Either is acceptable. Preserving it untouched is not — record which you chose and
why.

## 5. Add `--all-features` to CI

The feature was dead partly because nothing ever built it. Add `--all-features`
to the CI test step.

If `jemalloc` is deleted this is trivially satisfied. If kept, and
`--all-features` breaks, that breakage is the reason the feature needed
attention.

**Scope boundary, per RFC 027 Rule 2.** This handoff covers the CLI's allocator,
the `jemalloc` feature, `alloc_counter`'s visibility, and the CI feature flag.
Other performance findings — `C-02` (DOM re-serialisation for the capacity hint),
`C-04` (Python GIL), `C-05` (`create_dir_all` per file) — are **not** covered and
are M4. The boundary is at "things installed in the shipped binary that should
never have been there" versus "things that could be faster".

## 6. Required verification

Per RFC 027 Rule 3, state tree-vs-artifact for each.

1. Bulk conversion timing **before** the change — release build, real numbers,
   core count stated.
2. The same after.
3. Conversion output byte-identical before and after, over the full test corpus.
4. `--all-features` passes.
5. `cargo test --workspace --locked`, fmt, clippy `-D warnings`.
6. `cargo tree` before and after if `jemalloc` is deleted, showing the
   dependencies gone.

## 7. Prohibited shortcuts

- Do not repeat the audit's 2.5× figure as if you measured it.
- Do not leave `jemalloc` untouched.
- Do not use `#[doc(hidden)]` as a privacy mechanism.
- Do not change conversion behaviour. This slice is performance and surface only.

## 8. Known risks

| Risk | If it happens |
|---|---|
| A benchmark relies on the CLI installing the counter | Check the benchmark targets first; move installation there. |
| Measured gain is much smaller than 2.5× | Report it plainly. The allocator still should not ship. |
| `--all-features` breaks CI | Expected if `jemalloc` is kept. That is the finding — fix or delete. |
| Visibility reduction breaks a build | Report which target and why. |

## 9. Acceptance checklist

- [ ] No `#[global_allocator]` in `cli/src/main.rs`
- [ ] Before/after timings measured by you, with machine details
- [ ] Output byte-identical
- [ ] `jemalloc` deleted or implemented; choice recorded with a reason
- [ ] `--all-features` in CI and passing
- [ ] `alloc_counter` visibility reduced, or a written reason it cannot be
- [ ] CHANGELOG notes the public-surface change

## 10. Escalate rather than decide

Stop and raise if: a benchmark cannot work without the public `alloc_counter`;
implementing `jemalloc` looks worthwhile (that is a separate RFC, not this
slice); or output changes at all.
