# RFC 021 — Bulk conversion output-collision safety

**Status.** Accepted 2026-08-31 — implementer may start
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P0
**Touches.** `src/lib.rs` (bulk path), `cli/src/main.rs` reporting, tests.
**Source.** External audit 2026-08-31, `S-02` (High). Reproduced independently.
**Prepared.** 2026-08-31

## Summary

Converting multiple files whose names share a stem writes one output and
**reports success for all of them**. The losing file's content is discarded
silently. Report the collision instead.

## The defect, reproduced

```
$ mdka -o out/ a/index.html b/index.html
a/index.html -> out/index.md
b/index.html -> out/index.md
$ ls out/          # index.md
$ cat out/index.md # FROM A          — FROM B is gone
$ echo $?          # 0
```

Both inputs report success. One file's content no longer exists anywhere.

`index.html` is the most common filename on the web, so converting any directory
tree of scraped or exported HTML hits this immediately. Which input survives is
decided by whichever rayon worker writes last — it is a race, so the outcome is
not even consistent between runs.

**This is the worst failure class the project can ship: silent data loss that
reports success.** It ranks above the npm breakage on severity of *kind*, and
below it only because the npm package fails loudly and immediately.

## Design

Detect stem collisions across the input set and fail the losers, rather than
letting the last writer win.

### Constraint: this must not touch `MdkaError`

`MdkaError` has exactly one variant, `Io(#[from] std::io::Error)`, and is **not**
`#[non_exhaustive]`. Adding a variant is a breaking change, and so is adding the
marker — Rust classifies `enum_marked_non_exhaustive` as major. `ROADMAP.md`
§Major version position states no major version is planned.

**The audit's remediation plan bundles this fix with an error-type redesign and
schedules the pair as medium-term. That sequencing would park a live data-loss
defect behind a major version this project has no plan to cut.** It is the one
place the audit's recommendations should not be followed as written.

Fix it inside the existing type. The bulk API already returns

```rust
Vec<(&'a P, Result<PathBuf, MdkaError>)>
```

and `MdkaError::Io` wraps an arbitrary `std::io::Error`, so a collision is
expressible today as `io::ErrorKind::AlreadyExists` carrying both colliding
source paths and the contested destination in its message. No new variant, no
marker, no major bump, and the CLI's existing per-file error reporting surfaces
it without change.

The richer error type stays desirable and stays recorded for 3.0 (`C-06`,
`C-10a`). It is not a prerequisite for stopping data loss.

### Which input wins

**The first occurrence in input order wins**; every later collider errors. Input
order is deterministic and caller-controlled, so the result is reproducible —
unlike today, where the winner is whichever thread finished last.

Detection happens **before** any conversion work, so no partial output is written
for a file that is going to be rejected.

### Not in scope

Auto-renaming (`index.md`, `index-1.md`). It is a plausible future feature and a
bad default: it invents filenames the caller did not ask for, and it would make
the CLI's output set depend on input ordering in a way users would then rely on.
Report the conflict and let the caller decide.

Also out of scope: the unsynchronised concurrent `fs::write` the audit notes
alongside this. Once colliding destinations are rejected, two workers no longer
target one path. Non-atomic writes in general (`S-10`) remain an M4 item.

## Compatibility

Additive in behaviour, and a patch. Inputs that previously "succeeded" while
destroying data now return an error for the discarded file. That is a change in
observable behaviour, and it is the point — the previous behaviour was a defect,
and no caller can be relying on losing their data.

The `Ok` path is unchanged for any input set without collisions.

## Risks

| Risk | Mitigation |
|---|---|
| Pre-flight detection is O(n²) on large sets | Use a hash set keyed by resolved destination path. |
| Paths differing only by case on case-insensitive filesystems | Real, and out of scope. Note it in the review request rather than half-solving it. |
| A caller was relying on last-write-wins | Not credible; the winner is currently a race. |

## Acceptance criteria

1. Two inputs with colliding output stems: the first succeeds, later ones return
   an error naming both source paths and the contested destination.
2. The process exit code is non-zero when any input fails.
3. No output file is written for a rejected input.
4. `MdkaError` is unchanged — no new variant, no `#[non_exhaustive]`.
5. A test covers the collision case end to end, asserting both the error **and**
   that the surviving file has the first input's content.
6. A non-colliding bulk conversion is byte-identical to today.
