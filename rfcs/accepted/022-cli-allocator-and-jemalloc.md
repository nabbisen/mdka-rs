# RFC 022 — Remove the counting allocator from the shipped CLI; settle `jemalloc`

**Status.** Accepted 2026-08-31 — implementer may start
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P1
**Touches.** `cli/src/main.rs`, `Cargo.toml`, `src/alloc_counter.rs` visibility, CI test step.
**Source.** External audit 2026-08-31 — `C-01` (High), `C-03` (Medium), `C-07` (Low).
**Prepared.** 2026-08-31

## Summary

The shipped `mdka` CLI installs a **measurement instrument** as its global
allocator. Remove it. While there, resolve the `jemalloc` feature, which gates
dependencies that no code uses.

## `C-01` — the allocator

`cli/src/main.rs:24`:

```rust
#[global_allocator]
static ALLOCATOR: mdka::alloc_counter::CountingAllocator = mdka::alloc_counter::CountingAllocator;
```

Confirmed present in the released binary's source. Its comment says it does not
interfere with library users — true, and beside the point: it interferes with
**CLI users**, who are the ones running bulk conversion.

The audit measured up to **2.5×** cost from 1 to 32 threads, capping parallel
speedup at 5.4× where 13.7× was available. **I have not reproduced that number**
— I confirmed the structural defect, not the magnitude. The implementer must
measure before and after rather than repeating the audit's figure.

The magnitude does not change the decision. A counting allocator exists to be
measured, not shipped, and every allocation in the hottest path pays for a
counter nobody reads at runtime. In a project whose documentation leads on
performance characteristics, this is the flagship `parallel` feature paying a
toll for a benchmark harness.

**Fix:** delete the two lines. Benchmarks that need the counter install it in the
benchmark target, which is where an instrument belongs.

### `C-07` — `alloc_counter` is public API

Because `cli` reaches it as `mdka::alloc_counter::CountingAllocator`, it is public
surface for semver purposes — a benchmark-only concern any downstream crate can
depend on. Once the CLI no longer installs it, reduce its visibility. If a
benchmark target still needs it, `#[doc(hidden)]` is **not** privacy; prefer
`pub(crate)` plus a benchmark-local shim, or gate it behind a feature that is off
by default.

## `C-03` — the dead `jemalloc` feature

`Cargo.toml:47` declares:

```toml
jemalloc = ["dep:tikv-jemallocator", "dep:tikv-jemalloc-ctl"]
```

`grep -rn jemalloc src/ cli/src/` returns **nothing**. The feature pulls two
dependencies that no code references. Enabling it changes nothing except build
time and the dependency graph.

**Decide, do not preserve.** Two acceptable outcomes:

- **Delete it** — the default, absent a reason to keep it. Nothing uses it and
  nothing has since it was introduced.
- **Implement it** — a real `#[global_allocator]` swap for the CLI, benchmarked
  against the system allocator to show it earns its place.

Deleting is a change to the feature set. It is not a breaking API change in the
sense the release policy governs, but a consumer building with
`--features jemalloc` would see the flag disappear. Given the feature has never
done anything, that build was already getting the system allocator; nothing they
observe changes.

**Also add `--all-features` to the CI test step.** The feature was dead partly
because nothing ever built it. A feature no CI job exercises is a claim, not
code — the same defect class as the inert options M2 spent four RFCs removing,
in the build system rather than the API.

## Compatibility

Patch. Removing the allocator changes performance, not behaviour. Output is
byte-identical; only the allocation counter is gone, and nothing reads it at
runtime.

## Risks

| Risk | Mitigation |
|---|---|
| A benchmark depends on the CLI installing the counter | Check the benchmark targets before deleting; move installation there. |
| `--all-features` breaks CI immediately | Likely, and useful. If `jemalloc` is deleted the problem disappears; if kept, the breakage is the reason to fix it. |
| Reducing `alloc_counter` visibility breaks a downstream user | Improbable but real, since it is public today. Note it in the CHANGELOG. |

## Acceptance criteria

1. No `#[global_allocator]` in `cli/src/main.rs`.
2. A before/after measurement of bulk conversion, **run by the implementer**, in
   the review request — with the actual numbers, whatever they show.
3. `jemalloc` is deleted or implemented, and which was chosen is recorded with a
   reason.
4. `--all-features` is in the CI test step and passes.
5. `alloc_counter`'s visibility is reduced, or a written reason it must stay
   public.
6. Conversion output is byte-identical; the test suite is unchanged in count
   except for anything added here.
