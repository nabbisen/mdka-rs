# Developer Handoff — the clippy timebomb went off; `main` is red

**Priority: first.** `main` is red and every push will be red until this lands.
**Scope.** `python/Cargo.toml` only. No `src/` change, no release.

---

## What happened, exactly as recorded

The ROADMAP has carried this since `3.3.0`'s prep:

> **`clippy::clone_on_copy` in `python/src/lib.rs` is a future-CI timebomb.** It fires on clippy
> `0.1.99` and not on the GitHub runner's current stable, which is why `main` is green. A runner
> image update will turn CI red on an unrelated push.

**That is what has now happened.** CI's clippy is **1.99.0**, and it fired on a documentation-only
commit (`e2c7be5`, `RELEASE-CHECKLIST.md`):

```
error: using `clone` on type `ConversionMode` which implements the `Copy` trait
  --> python/src/lib.rs:11:11
   |
11 | #[pyclass(from_py_object)]
   |           ^^^^^^^^^^^^^^ help: try removing the `clone` call: `from_py_object`
```

It is a **false positive in PyO3's own macro expansion** — the clone is in generated code, and the
suggestion ("try removing the `clone` call: `from_py_object`") is not actionable.

## The fix, proven under CI's exact command

Add to `python/Cargo.toml`:

```toml
[lints.clippy]
# PyO3's `from_py_object` expansion clones a Copy enum; the suggestion is not
# actionable because the clone is in generated code, not ours. Fires on clippy
# 1.99.0 and not on earlier stables.
clone_on_copy = "allow"
```

Verified with `cargo clippy --workspace --all-targets --all-features --keep-going -- -D warnings`,
the exact CI invocation: **0 errors.**

## What does not work, so you do not spend time on it 🛑

**An item-level `#[allow(clippy::clone_on_copy)]` does not suppress it.** I tested both placements —
above `#[pyclass]`, and between `#[pyclass]` and `#[derive]` — and under `-D warnings` the error
still fires. The lint's span resolves into macro-generated code, which an item attribute does not
cover.

**A warning about my own first measurement, because it is the trap here:** my initial runs used
`cargo clippy -p mdka-python --all-targets --all-features` **without `-D warnings`** and counted
`^error: using`. That reported 0 for placements that do not work, because without `-D warnings` the
lint is a *warning* and never matches `error:`. **Only measure with `-D warnings`.** Every number in
this handoff is from the full CI command.

## The cost, stated

`[lints.clippy]` is **crate-wide**: it also silences any genuine `clone_on_copy` in the `mdka-python`
binding. That is the price, and nothing narrower works. The alternative — dropping `Copy` from
`ConversionMode` to remove the lint's premise — would contort our own type to suit a false positive
in a dependency's macro, and would ripple through every by-value use. **Not worth it.**

A crate-level `#![allow(clippy::clone_on_copy)]` in `python/src/lib.rs` also works and is equivalent
in scope; the manifest form is preferred because it is declarative, visible where the crate is
configured, and survives file reorganisation. Use the manifest.

## Criteria

1. `cargo clippy --workspace --all-targets --all-features --keep-going -- -D warnings` clean — paste
   the command and its result.
2. `cargo test --workspace` unchanged at **675**; a lint setting changes no behaviour, so a different
   number means something else happened.
3. `python/Cargo.toml` only. No `src/`, no version bump, no release.
4. The comment in the manifest says **why** — a bare `allow` with no reason is how this becomes
   someone else's mystery.
5. **After pushing, confirm CI green on the full SHA.** This is the point of the slice; a local clean
   run is not it.

## Report

`.git-exclude/review-request/clippy-timebomb/README.md`. Push as soon as it is green locally —
`main` is red meanwhile.

## Then

`rfcs/handoffs/docs-audit/handoff.md` is the other slice waiting, and it is not urgent. This one
first.
