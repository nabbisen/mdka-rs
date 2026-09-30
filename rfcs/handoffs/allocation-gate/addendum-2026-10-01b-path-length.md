# Addendum 2 — allocation gate, 2026-10-01 (b)

Amends `handoff.md` and `addendum-2026-10-01.md`, both of which stay as issued.

**`be278e0` is red on CI.** `main` is red now. This is the first push where the gate ran anywhere but
this machine, and it caught a third environment dependence — after thread count, after toolchain.

```
bulk file conversion allocation moved: measured 34125 B, baseline 34205 B, diff 80 B (tolerance 0 B)
```

## Cause — proven, not guessed

**The measurement depends on the length of the fixture path**, which the pid suffix introduced last
round to make the temp directories unique. I patched an override in and swept the suffix length:

| suffix chars | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---:|---:|---:|---:|---:|---:|---:|
| bulk measured | 34005 | 34045 | 34085 | 34125 | 34165 | **34205** | 34245 |

**Exactly +40 B per character.** The baseline `34205` was taken with a 6-digit pid; the CI runner's pid
was `5153`, four digits — **34125, the 80 B the gate reported.** Not noise, not the runner, not the
compiler: arithmetic.

It follows that `std::env::temp_dir()`'s own length matters too, so a machine with a longer `TMPDIR`
shifts the baseline as well. **The fixture path must not vary in length at all.**

## Fix — proven

Create one fixture root, `set_current_dir` into it, and use **relative** paths of fixed length
(`in`, `out`, `f0.html`) for the conversion itself. Measured:

| suffix chars | 1 | 4 | 6 | 9 |
|---|---:|---:|---:|---:|
| bulk measured | **32905** | **32905** | **32905** | **32905** |

and across thread counts 1 / 2 / 4 / 8 / 32: **32905 every time.** The absolute path still varies —
but nothing inside the measured window sees it.

This test binary holds exactly one `#[test]`, so a process-global `set_current_dir` is contained.
Restore the directory (or just `remove_dir_all` the root by an absolute path captured beforehand)
after the measurement.

## What to do

1. Apply the relative-path fix. **Re-derive the baseline from your own run** — `32905` is my number,
   on my machine; the harness offset may differ, as it did last round.
2. **Re-derive the `2.9.0` comparison** the same way and confirm the ~152 B/file signal survives.
3. **Add the path-length sweep to the module doc**, beside the thread-count and toolchain evidence.
4. **Correct the header.** It currently presents `std::process::id()` as a fix; it was also the third
   environment dependence. Say so.
5. **Push as soon as it is green locally** — `main` is red until this lands.

## The general lesson, worth one line in the file

Three rounds, three environment dependences: **pool size, toolchain, path length.** Each was found by
running somewhere new, never by reasoning. The module doc should say plainly that a zero-tolerance
allocation baseline is a claim about an environment, and that the only proof is running it in a
different one.

## Scope

`tests/allocation_gate.rs` only. The architect's local commits (`0b81a2c`, `9995c6a`, and this one)
are carried along as ancestors when you push; that is expected and is not you pushing their work.
Nothing else, and no release.
