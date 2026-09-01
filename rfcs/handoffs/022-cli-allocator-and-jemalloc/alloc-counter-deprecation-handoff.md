# Developer Handoff — RFC 022 · deprecate `alloc_counter` instead of removing it

**Governing RFC.** [RFC 022](../../accepted/022-cli-allocator-and-jemalloc.md), § Correction
**Corrects.** Part of `d5d0551`, already on `main`
**Milestone.** M2b → `2.2.2`
**Follows.** [`.git-exclude/reviewed/022-alloc-counter-removal-decision/README.md`](../../../.git-exclude/reviewed/022-alloc-counter-removal-decision/README.md) — owner ruling
**Prepared.** 2026-09-01

---

## 1. What changed and why

**Your implementation was correct for the goal it was given.** The goal changed.

`d5d0551` removed `pub mod alloc_counter;` from the library. That is a
compatibility break, and `ROADMAP.md` reserves those to the project owner. RFC
022's risk table — mine — called it a CHANGELOG note, which under-weighted it.

**Owner ruling: deprecate now, remove on a schedule inside v2.** Deprecate in
`2.2.2`, remove in `2.4.0`.

**This is less work than what you built**, and undoes most of it.

## 2. The change

Restore the module to the library, deprecated:

```rust
// src/lib.rs
#[deprecated(
    since = "2.2.2",
    note = "benchmark-only utility, never part of the conversion API; \
            scheduled for removal in 2.4.0. See RFC 022."
)]
pub mod alloc_counter;
```

Then:

- Move `benches/alloc_counter.rs` back to `src/alloc_counter.rs`.
- **Delete the three `#[path]` shims** in `benches/memory.rs`,
  `examples/quick_mem.rs`, `examples/measure_mem.rs`.
- **Delete the `#![allow(dead_code)]`** you added — it was only needed because
  the module was compiled privately into three binaries. As public API, every
  item is reachable and none is dead.
- Each of the three consumers goes back to `use mdka::alloc_counter::…` with
  `#[allow(deprecated)]` at the use site — the RFC 005 pattern, narrowest scope
  that compiles, not crate-level.

**Keep everything else from `d5d0551`.** The allocator removal from
`cli/src/main.rs`, the `jemalloc` deletion, `--all-features` in CI, and the
`architecture.md` / `README.md` corrections are all approved and unaffected.

`architecture.md` needs one more touch: the file is back under `src/`, and the
line should now say deprecated and scheduled for removal in `2.4.0`.

## 3. CHANGELOG

`[Unreleased]` currently says `alloc_counter` "moved out of the library
entirely" and that a downstream user must vendor it. **That is now wrong** —
rewrite it as a `Deprecated` entry: the module remains, is deprecated since
`2.2.2`, and is scheduled for removal in `2.4.0`.

Say plainly that it was never part of the conversion API and exists only for
this project's benchmarks, so nobody has a reason to depend on it.

## 4. Scope boundary, per RFC 027 Rule 2

This handoff covers **only** the `alloc_counter` surface decision and its
documentation. It does not revisit the allocator removal, `jemalloc`, or the CI
feature flag — all approved.

It also does **not** perform the `2.4.0` removal. That is scheduled, gated, and
belongs to M4 (§6).

## 5. Required verification

Per RFC 027 Rule 3, state tree-vs-artifact for each.

1. `cargo test --workspace --all-features --locked` — expect **139**, unchanged.
2. `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. **Public API diff against `2.2.1`**: `git diff 2.2.1 HEAD -- src/lib.rs`, `pub` lines only. Expect the `pub mod alloc_counter;` line **restored** — i.e. no net public-surface change from `2.2.1`. That is the whole point; show it.
4. `#[allow(deprecated)]` is at the three use sites, not crate-level.
5. `benches/memory.rs`, `examples/quick_mem.rs`, `examples/measure_mem.rs` all still build and run.
6. No `#[path]` shim and no `#![allow(dead_code)]` remain from `d5d0551`.

## 6. What happens at `2.4.0` — not now, recorded so it is not lost

The removal is scheduled for `2.4.0` (M4) with a **required gate**:

> Immediately before removing, re-run the reverse-dependency check on
> crates.io and confirm no dependent references `alloc_counter`,
> `CountingAllocator` or `AllocSnapshot`.

Checked at this ruling, all six current dependents (`htm_md`,
`bigquery-functions`, `elvish-core`, `threadcat`, `htmlmd-core`,
`zapmyco-tools`) reference none of them — every use of `mdka` across all six is
`from_html`, `html_to_markdown`, `html_to_markdown_with` or `options`.

**That check must be re-run, not cited.** New dependents appear between
releases, and a two-release-old check is exactly the stale claim this project
keeps finding. If a dependent then uses it, the removal waits.

## 7. Prohibited shortcuts

- Do not remove the module now.
- Do not use crate-level `#![allow(deprecated)]`.
- Do not leave the CHANGELOG saying it was removed.
- Do not revert anything else from `d5d0551`.

## 8. Acceptance checklist

- [ ] `#[deprecated]` `pub mod alloc_counter;` restored in `src/lib.rs`, `since = "2.2.2"`, note naming `2.4.0`
- [ ] Module file back at `src/alloc_counter.rs`
- [ ] Three `#[path]` shims deleted; `#![allow(dead_code)]` deleted
- [ ] `#[allow(deprecated)]` at each of the three use sites only
- [ ] Public API diff vs `2.2.1` shows **no net change**
- [ ] CHANGELOG entry is `Deprecated`, not "removed", and names `2.4.0`
- [ ] `architecture.md` line updated
- [ ] 139 tests; fmt and clippy clean
- [ ] Nothing else from `d5d0551` reverted

## 9. Escalate rather than decide

Stop and raise if: the deprecated module cannot be consumed by benches/examples
without a wider `allow`; the public API diff shows anything other than a clean
restore; or you find another consumer beyond the three known ones.

## 10. On the round trip

You built the right thing for the RFC as written, and the RFC was wrong on a
point that was mine to get right. The `pub(crate)` option I offered does not
compile, and the compatibility question should never have been a CHANGELOG note.

The part of your work that survives unchanged is the part that mattered: the
allocator is out of the shipped binary, measured, with byte-identical output.
