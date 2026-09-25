# Addendum — RFC 048 slice 2, 2026-09-26 · the `crates package gate` cache

**Amends.** `slice-2-result-model-handoff.md`, frozen and unedited. From the review at
`.git-exclude/reviewed/048-slice-2-result-model/README.md` §2.
**Baseline.** `1371c81`.
**Scope.** `.github/workflows/crates-package-gate.yaml`. **No library, binding, test or documentation change.**

---

## 1. Your diagnosis was wrong, and the real cause is cheaper to fix

You reported the gate as *structurally* red until the workspace version moves off `2.9.0`, because
`cargo package --workspace` resolves `mdka` from crates.io when that version is published. **It does not**,
and the run's own log says so.

| Check | Result |
|---|---|
| A clean `git worktree` at `1371c81`, `cargo package --workspace` | **Passes.** All four crates verified; `target/package/tmp-registry/mdka-2.9.0.crate` is the local one, and `pub struct FileOutcome` is in the extracted `mdka-2.9.0/src/lib.rs` |
| The failing run's resolution | `Unpacking mdka v2.9.0 (registry '…/target/package/tmp-registry')` — the **local** registry, not crates.io |
| The failing run's first step | `Cache restored from key: Linux-cargo-package-gate-0658c3…`, 253 MB |

**The key is `hashFiles('**/Cargo.lock')`. Slice 2 changed no dependency**, so the key did not move, so
`target/` came back from **before** the API change — carrying `target/package/`'s extraction of `mdka 2.9.0`
with the old source in it.

**This is the RFC 030 addendum's own defect, in a directory that addendum did not cover.** Its comment, in
this file, already states the mechanism: *"Cargo never re-extracts while a version is unchanged, so a cached
extraction from before a source-only change … verifies stale source under the current version number."* It
excluded `~/.cargo/registry/src` and left `target/package/`, which holds exactly such an extraction.

## 2. What this costs, and why it is worth fixing rather than working around

**The gate can verify stale source at any time** — any source-only change to `mdka`, unchanged version, warm
cache. It has passed until now only because no such change altered the API in a way the dependents notice.
**The red is the gate telling the truth about itself.**

So: **not** a version bump, which masks it until the next inter-release source change, and **not** a standing
exception — the checklist says this gate has none, and you were right not to claim one.

## 3. The fix

**Keep `target` in the cache; exclude the packaging directory.** `actions/cache` supports `!`-prefixed
exclusions, so the dependency build stays cached and the extraction does not.

**Price it before and after** (`ROADMAP.md`'s rule: judge CI cost cumulatively, and prefer keeping the
benefit while removing the defect). If dropping `target/package` costs meaningful minutes, say the number —
a correct gate is worth some seconds, but the trade should be visible, not assumed.

**Extend the comment block** that already explains the `~/.cargo/registry/src` exclusion, so the next reader
sees both halves of the same lesson in one place, with the date and this slice named.

## 4. Show it works, both ways 🛑

A cache exclusion that is never exercised proves nothing.

1. **Green on `main` as it stands** — the gate passes at the current commit, with a cache restored.
2. **Stale-source control:** demonstrate that the old behaviour would have failed and the new one does not.
   The cheapest honest form is two runs on a scratch branch: one with the cache warm from a commit **before**
   the API change, and one after the exclusion. Paste both conclusions.
3. If you cannot construct (2) on a runner without a lot of machinery, **say so and show the local
   equivalent** — a clean worktree passing, and the same worktree failing when `target/package` is copied in
   from a pre-slice-2 build. Do not skip it silently.

## 5. Criteria

1. `crates package gate` green on `main`, with a cache hit in the log.
2. §4's demonstration, in whichever of its two forms, with output.
3. The cost difference stated as a number.
4. The comment block covers both exclusions and says why, naming the date and the slice.
5. No change outside that workflow file.

## 6. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. **Name who approved
anything you push.** `git commit -F <file> -- <explicit paths>`.

**Slice 3 waits for this.** Writing the migration guide against a tree whose packaging nobody has verified
would mean documenting a `3.0` that may not package.

Append to `.git-exclude/review-request/048-slice-2-result-model/README.md` as a further part — same slice.
