# Developer Handoff — RFC 021 · Bulk conversion output-collision safety

**Governing RFC.** [RFC 021](../../accepted/021-bulk-output-collision-safety.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P0
**Prepared.** 2026-08-31

---

## 1. Purpose

Converting files whose output names collide writes one file, discards the rest,
and **reports success for all of them**. Make the losers fail instead.

## 2. Reproduce it first

```
mkdir -p /tmp/c/a /tmp/c/b /tmp/c/out
echo '<h1>FROM A</h1>' > /tmp/c/a/index.html
echo '<h1>FROM B</h1>' > /tmp/c/b/index.html
mdka -o /tmp/c/out /tmp/c/a/index.html /tmp/c/b/index.html
# both lines report "-> out/index.md", exit 0
cat /tmp/c/out/index.md   # FROM A. FROM B no longer exists anywhere.
```

Which input survives is a race between rayon workers, so it may not be `A` on
your machine. That non-determinism is part of the defect.

## 3. ⚠ The constraint that shapes this entire slice

**`MdkaError` must not change.** Not a new variant, not `#[non_exhaustive]`.

It has exactly one variant, `Io(#[from] std::io::Error)`, and is not
`#[non_exhaustive]`. Adding a variant is a breaking change — and so is adding the
marker, because an exhaustive downstream `match` loses its exhaustiveness
(`cargo-semver-checks: enum_marked_non_exhaustive`). `ROADMAP.md` §Major version
position says no major version is planned.

The audit recommends restructuring the error type here. **We are deliberately not
following that recommendation**, because doing so would park a live data-loss
defect behind a major version we have no plan to cut. The richer error type is
recorded in `ROADMAP.md` as 3.0 work.

**If you conclude the fix cannot be expressed without touching `MdkaError`, stop
and raise it.** Do not decide that on your own — it changes the release shape.

## 4. Design

The bulk API already returns:

```rust
Vec<(&'a P, Result<PathBuf, MdkaError>)>
```

and `MdkaError::Io` wraps an arbitrary `std::io::Error`. So a collision is
expressible today as `io::ErrorKind::AlreadyExists` carrying both source paths
and the contested destination in its message. No type change, and the CLI's
existing per-file error reporting surfaces it unmodified.

### Rules

- **First occurrence in input order wins.** Input order is caller-controlled and
  deterministic; the current winner is whichever thread finished last.
- **Detect before converting.** No conversion work and no partial output for a
  file that will be rejected. Build the destination set up front.
- **Exit code non-zero** when any input fails.

### Not in scope

- Auto-renaming (`index-1.md`). Plausible future feature, bad default — it
  invents filenames the caller did not ask for.
- Non-atomic writes generally (`S-10`) — M4.
- Case-insensitive filesystem collisions (`Index.html` vs `index.html`). Real,
  and **out of scope**. Note it in the review request rather than half-solving
  it.

**Scope boundary, per RFC 027 Rule 2.** This handoff covers the bulk path in
`src/lib.rs` and its CLI reporting only. Single-file conversion has no collision
concept and is untouched. The error-type improvement that would give better
messages here is deferred to 3.0 with a named home in `ROADMAP.md`.

## 5. Required verification

State for each whether it ran against the tree or an installed artifact
(RFC 027 Rule 3).

1. The §2 reproduction, before the change.
2. The same command after: first input succeeds, second errors naming both
   sources and the destination, exit code non-zero.
3. `cat` the surviving file — it must contain the **first** input's content.
4. A non-colliding bulk conversion produces byte-identical output to 2.2.0.
5. Run the collision case **repeatedly** — at least 20 times — and confirm the
   winner is always the first input. This is currently a race; prove it no longer
   is.
6. `cargo test --workspace --locked`, `cargo fmt --check`, clippy `-D warnings`.
7. `cargo public-api` or equivalent, or a manual check, confirming `MdkaError` is
   unchanged.

## 6. Prohibited shortcuts

- Do not add a variant to `MdkaError`.
- Do not add `#[non_exhaustive]`.
- Do not auto-rename.
- Do not detect the collision *during* the parallel map and race to report it —
  detect up front, deterministically.
- Do not report a single run as proof the race is gone.

## 7. Known risks

| Risk | If it happens |
|---|---|
| Pre-flight detection is O(n²) | Use a hash set keyed by resolved destination. |
| A test depends on current collision behaviour | Unlikely; if one does, it was asserting data loss — update it with a comment naming RFC 021. |
| The destination is computed in more than one place | Likely. Find them all; the detection and the write must agree, or you have moved the race rather than removed it. |
| Relative vs absolute path forms compare unequal | Resolve before comparing. Two spellings of one path must collide. |

## 8. Acceptance checklist

- [ ] First input wins, deterministically, proven over repeated runs
- [ ] Later colliders return an error naming both sources and the destination
- [ ] Non-zero exit when any input fails
- [ ] No output written for a rejected input
- [ ] `MdkaError` unchanged — no variant, no marker
- [ ] Test covers the collision end to end, asserting the error **and** the
      surviving content
- [ ] Non-colliding conversion byte-identical to 2.2.0
- [ ] Case-insensitive collision limitation noted in the review request
- [ ] Count reconciles; fmt and clippy clean

## 9. Escalate rather than decide

Stop and raise if: the fix appears to require an `MdkaError` change; destination
computation cannot be made single-source; or the first-wins rule turns out to
conflict with something in the CLI's ordering.
