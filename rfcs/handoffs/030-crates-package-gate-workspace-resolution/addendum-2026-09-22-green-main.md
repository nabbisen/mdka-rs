# Addendum — get `main` green: cache partition + one docs block, 2026-09-22

**Amends.** RFC 030's gate (§1) and touches RFC 032's docs gate (§2)
**Source.** `.git-exclude/review-request/main-red-after-039-037/README.md` — read §3 for why §1 is what it is
**Baseline.** `81f0ec5` — 5 of 7 workflows green; `crates package gate` and `docs example gate` red
**Scope.** Two small fixes, **one commit**. Neither is a defect in RFC 037 or RFC 039.

---

## 1. `crates package gate` — cache the downloads, not the extractions

In `.github/workflows/crates-package-gate.yaml`, change the cache step's `path` to:

```yaml
          path: |
            ~/.cargo/registry/index
            ~/.cargo/registry/cache
            target
```

replacing `~/.cargo/registry` and `~/.cargo/git`. **Key and restore-keys unchanged.**

**Why.** `cargo package --workspace` builds the dependents against an extraction in
`~/.cargo/registry/src/<hash>/mdka-<version>/`, and **cargo never re-extracts while the version is
unchanged**. The cache key is `hashFiles('**/Cargo.lock')`; RFC 039 added no dependencies, so the key did not
change and CI restored the pre-RFC-039 extraction — hence *"cannot find function
`html_to_markdown_many_with`"* against source that no longer exists.

All stale state is in `src/`, which is derived; `cache/` holds only the crates.io `.crate` downloads and has
no tmp-registry entry at all. Excluding `src/` forces a fresh extraction at **zero** download cost.
`~/.cargo/git` goes entirely — `Cargo.lock` has zero git-sourced entries.

**Add a line to the workflow comment**, beside the RFC 030 note: the gate verifies a from-scratch consumer
build, so `~/.cargo/registry/src` must never be cached — a cached extraction makes it verify stale source.

**Verify:** the gate green on the pushed tip, and confirm from the log that `mdka-node` compiles against the
freshly packaged crate.

## 2. `docs example gate` — one block needs to stand alone

`docs/src/getting-started/usage-python.md:104` fails with `NameError: name 'mdka' is not defined`. It is
written as a continuation of the block above it, which does `import mdka` and defines `pages`. **RFC 032's
gate runs every fenced block as a standalone program.**

Give it its own `import mdka` and its own `pages`, or mark it non-runnable.

**While you are there:** the rule is general — *every runnable fenced block must stand alone*. Worth a line
in whatever the docs-authoring guidance is, since continuation blocks are the natural way to write a
tutorial and this will recur.

## 3. Report back

No review package needed. Commit both, push, and report the seven workflow conclusions on the full SHA.

**One note on how §2 slipped, for the next time a gate is degraded:** the RFC 039 package reported its local
docs-gate run as *"9 failures, all no-`--python` interpreter environment misses, none in a file or block this
slice touched."* The interpreter was missing, so **every** Python block failed — the real failure was inside
the set classified as noise. When a gate is degraded, its failures cannot be triaged by category; only re-run
it somewhere it works.
