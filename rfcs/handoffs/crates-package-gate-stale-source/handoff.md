# Handoff — the `crates package gate` verified stale source

**Priority:** after the allocation gate fix, before the `3.1.0` cut.
**Severity:** this gate can pass while verifying the wrong source. Today it failed, which was luck.

---

## What happened

`crates package gate` failed on `be278e0`:

```
Verifying mdka-cli v3.0.0
error[E0609]: no field `emphasis_from_style` on type `ConversionOptions`
```

`emphasis_from_style` **is** on `ConversionOptions` at that commit — `cargo test --workspace` passes
648 tests, and `cargo clippy --all-targets` is clean. The gate compiled `mdka-cli` against a
**different, older `mdka 3.0.0`** than the one in the tree.

The run log shows the sequence:

```
Packaging mdka v3.0.0 (…/mdka-rs)
Compiling mdka v3.0.0 (…/target/package/mdka-3.0.0)      ← fresh, correct
Unpacking  mdka v3.0.0 (registry …/target/package/tmp-registry)
Compiling mdka-cli v3.0.0 (…/target/package/mdka-cli-3.0.0)   ← failed here
```

`mdka` itself packaged and compiled fine from fresh source. What `mdka-cli` then resolved did not
have the new field. `Cache restored from key: Linux-cargo-package-gate-v2-…` — an exact hit.

## Why this is not the known "structurally red at release" condition

It is not. **The previous eleven runs of this workflow were all green**, including every push earlier
the same day. This is the first push that changed the library's public API **while the version stayed
`3.0.0`**, and that is exactly the shape this project already recorded:

> `reference_cargo_package_stale_extraction` — cargo never re-extracts a same-version crate.

## What to find out — and do not stop at one hypothesis

The obvious candidate is a cached, already-unpacked `mdka 3.0.0` under `target/package/` being reused
because its `.cargo-ok` marker is present. **That is a hypothesis, not a diagnosis.** This project has
already prescribed the wrong cache fix here once, on a confident-sounding single mechanism, and it
was measured and failed. Enumerate before fixing. At least:

1. A stale unpacked source tree under `target/package/` (or `tmp-registry`) restored by the cache.
2. A stale compiled `mdka` rlib that `!target/debug/**/*mdka*` does not cover, because this build
   happens under `target/package/*/target/`, not `target/debug/`.
3. Something in `~/.cargo/registry/cache` — the `.crate` tarballs are cached; the extracted `src` is
   not, but check rather than assume.
4. Anything the run log shows that none of these predict.

**Settle it with evidence from a run, not from reading the workflow file.** Listing the restored
cache's relevant paths in a debug step is cheap and decisive.

## The criterion that matters

**A version bump would make this go away, and that is not a fix.** `3.1.0` gives every path a new
name, so the gate will very likely be green at the release whatever we do. Do not let that close it:

> **Prove the fix at an unchanged version.** Change something in `src/` that `mdka-cli` uses, leave
> the version at its current number, push, and show the gate green — having actually compiled the new
> source, not skipped it. A green run that did not compile the change proves nothing.

The failure mode this gate has is not "goes red at releases". It is **"can go green against source
that is not in the tree"**, and that is worth an hour.

## Report

The mechanisms you enumerated, which one the evidence picked, what you changed, and the unchanged-
version proof above. If the cache key namespace needs bumping for the fix to take effect at all,
say so explicitly — this project has shipped an inert cache patch before, for exactly that reason.

## Scope

`.github/workflows/crates-package-gate.yaml`, and nothing in `src/` beyond whatever throwaway change
the proof needs (reverted before the final push). No release.
