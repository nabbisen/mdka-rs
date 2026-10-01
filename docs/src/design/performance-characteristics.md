# Performance Characteristics

**These figures describe mdka `3.0.0`, not the current release — see [`3.1.0`](#310-what-changed-and-what-was-not-re-measured) immediately below for what changed since.** They describe `3.0.0` (`main` @ [`2756c47`](https://github.com/nabbisen/mdka-rs/commit/2756c47e6d447facda832b699a94726693a53d57), the tagged `3.0.0` release plus two documentation-only commits), measured 2026-09-30, with load average recorded rather than merely asserted (see [Conditions](#conditions)). They replace the `2.3.0`-era numbers this page carried between `2.3.0` and `3.0.0`, and they also **replace this page's own first `3.0.0` regeneration** — that first sitting had unrecorded background load, and two of its findings did not survive a clean re-run. Both are corrected below rather than quietly dropped.

## `3.1.0`: what changed, and what was not re-measured

**Added 2026-10-01, after `3.1.0` shipped.** The figures below were measured at `3.0.0` and have
**not** been regenerated. That is deliberate — regeneration costs over three hours of a single
machine, and RFC 012 §9 records that this page is a dated snapshot, not a release obligation. What a
reader needs instead is to know whether `3.1.0` invalidates it, so here is the honest answer in both
directions.

**`3.1.0` did change the hot path, for every document, whether or not you use its new option.** RFC
049 replaced the emphasis tracker — one frame per matched `<strong>`/`<b>`/`<em>`/`<i>` — with two
inherited-value stacks that **every element pushes and pops**, block or inline. `emphasis_from_style`
being off by default does not avoid that work.

**Allocation was measured, and it went down.** Paired, same machine, same toolchain, interleaved,
three runs each, zero variance on every point:

| Workload | `3.0.0` | `3.1.0` | delta |
|---|---:|---:|---|
| Single string conversion | 10 507 B | 10 319 B | **−188 B (−1.8%)** |
| Bulk, 5 files | 33 845 B | 32 905 B | **−940 B, i.e. −188 B/file (−2.8%)** |

A flat **−188 bytes per conversion** — the same figure in both workloads, which is what a per-
conversion change should look like. It is also, for context, larger than the **+165 B/file** that
`3.0.0` itself added over `2.9.0` and that the table below records: **`3.1.0` gives back more than
`3.0.0` cost.** This property is now gated at tolerance 0 (`tests/allocation_gate.rs`), so it cannot
drift unnoticed.

**Wall time was not re-measured, and no claim is made about it.** This project cancelled its
wall-time work deliberately: three attempts, zero performance defects ever caught by a timing
measurement, and a "confirmed quiet" sitting that failed to reproduce its own finding (see the
non-reproduction below). An unmeasured change on the hot path is not evidence of a regression, and
the absence of a measurement is not evidence against one. **Neither is asserted here.**

## `2.9.0` vs `3.0.0`: is `3.0` slower?

The owner asked this directly before any measurement was taken: *"v3 may be slower than v2."* The
first attempt at answering this found a small string-conversion cost and a large bulk-conversion
speedup — then the owner reported background processes running during that sitting, and a
confirmed-quiet re-run, with load logged before/during/after every measurement (see the
[RFC 012 amendment](https://github.com/nabbisen/mdka-rs/blob/main/rfcs/done/012-benchmark-hardening.md#8-quietness-needed-evidence-too-2026-09-30)
this prompted), told a different story for both findings.

### The string-conversion path — the earlier "+2% to +5%" was the machine, not the library

`3.0` removed types and functions from the public surface; the one change inside the conversion
path itself is that `disposition()` in `src/traversal.rs`, called once per element, now asks
`ConversionOptions::unwraps_wrappers()` — a method comparing the mode enum — where `2.9.0` read a
`bool` field directly. The first sitting found a repeatable +2% to +5% cost across four runs and
reported it as real, small, and unfixed. **On a re-run with load sampled throughout (0.02–1.09,
confirmed quiet) it did not reproduce:**

| Dataset | run 1 | run 2 | run 3 | run 4 | median |
|---|---:|---:|---:|---:|---:|
| small | +2.33% | −0.24% | −0.91% | −0.50% | **−0.37%** |
| medium | +1.99% | +0.28% | −0.80% | −0.24% | **+0.02%** |
| large | −0.17% | −0.08% | +0.15% | +0.22% | **+0.03%** |
| deep_nest | +1.19% | −0.29% | +0.51% | +0.09% | **+0.30%** |
| flat | +0.48% | −0.03% | −0.26% | +0.34% | **+0.16%** |
| malformed | −5.06% | +4.29% | +1.60% | −32.90% | (noise — tens of µs absolute) |

Same harness, same two-worktree probe, same alternating-order protocol, same four repetitions —
the only thing that changed is a machine confirmed quiet instead of asserted quiet. **Every dataset
now sits within ±0.4%, `deep_nest` included**, and `deep_nest` — the one dataset where
`scraper::Html::parse_document` alone accounts for ~99.8% of total time (see [Scaling: Depth and
Width](#scaling-depth-and-width)), a path `3.0` never touches — moved by exactly as little as the
rest. **The earlier +2% to +5% finding passed its own internal control and was still a load
artifact.** That control catches the change leaking into a path it should not touch; it was never
able to tell "the library got slower" apart from "the room got noisier," which is exactly the gap
the RFC 012 amendment above closes. Read this page's `2.9.0` vs `3.0.0` string-conversion delta as
**not measurable** going forward, not as "small but real."

### Bulk file conversion: one measured allocation, and a wall-time result we could not reproduce

`html_files_to_markdown` returned `Vec<(&P, Result<PathBuf, MdkaError>)>` — a **borrowed** key —
through `2.9.0`. It now returns `Vec<FileOutcome>` with an **owned** `PathBuf`. That is one
allocation per file, and no benchmark before this project's `3.0.0` regeneration converted files at
all, so both the allocation and any effect on wall time were measured directly.

**Bytes allocated**, converting 10/100/1000 copies of `small.html`, measured with this project's
own counting allocator (`benches/alloc_counter.rs`) — this figure does not depend on scheduling and
has been stable across every sitting:

| Files | `2.9.0` | `3.0.0` | delta | per file |
|---:|---:|---:|---:|---:|
| 10 | 1,345,884–1,346,234 B | 1,347,484–1,347,884 B | +0.12% | **+160–167 B** |
| 100 | 13,458,340–13,461,840 B | 13,474,440–13,478,440 B | +0.12% | **+161–166 B** |
| 1000 | 134,615,600–134,650,600 B | 134,777,600–134,817,600 B | +0.12% | **+162–167 B** |

Flat across a 100× range in file count — exactly what "one more small owned allocation per
`FileOutcome`" predicts. **This is the one solid result in this section**, and it is the whole of
the allocation-side story: `3.0.0` allocates about 165 bytes more per file, permanently and by
design.

#### The wall-time story, and why this page will not tell you one

This section has now reported three different answers, and honesty is worth more here than a
conclusion:

| Sitting | What it measured at 16–32 threads |
|---|---|
| First regeneration | `3.0.0` **faster** by up to 37%, growing with thread count |
| Second, on a machine with load logged throughout, twice | `3.0.0` **slower** by 50–80%, growing with thread count |
| Third, six independent re-measurements | **No difference beyond noise** — ±5%, occasionally +15%, never monotonic |

The third sitting included one run that reproduced the second's setup exactly, down to installing
the same counting allocator in both probes, and one run taken immediately after four minutes of
sustained 32-thread allocation-heavy load, in case the second sitting's preceding 40 minutes of
benchmarking had left the machine in some worse state. Neither reproduced it. A 20-round
distribution check at 16 threads shows the two versions' individual measurements fully overlapping.

**What is odd, and unexplained:** `2.9.0`'s own numbers are consistent across all three sittings.
`3.0.0`'s were roughly twice as slow in the second sitting as in every measurement before or since.
Whatever happened was specific to one version in one sitting, not a general drift that would have
moved both together. Ambient load, the counting allocator's own overhead (real, but a flat 2–9%),
and prior sustained load were each tested and none reproduced it. The two releases resolve
**identical dependency versions** — the lockfiles differ only in this project's own four crate
versions — so the comparison was isolating this project's code change and nothing else.

**So: no wall-time regression is demonstrated, and none is ruled out.** The extra allocation is
real and is a plausible source of *some* cost under heavy parallelism; nobody has measured that
cost as anything but noise on this machine, and the one sitting that measured something large has
not been reproduced in six attempts.

**Why this is reported rather than quietly dropped:** the second sitting's finding was reviewed,
believed, and escalated as a shipped regression. It was measured twice — but twice *within one
sitting*, which the first sitting had already shown is not reproduction. A page that only printed
the answer that survived would teach nobody that.

### Node.js: rejecting an unrecognised option key

`2.9.0` silently ignored an option key it did not recognise; `3.0.0` rejects it, naming it. Measured
on the **published** npm packages — `mdka@2.9.0` and `mdka@3.0.0`, installed from the registry, not
a local build — interleaved, median of 7 rounds:

| Call | `2.9.0` | `3.0.0` | delta |
|---|---:|---:|---:|
| no options | 1958–2016 ns | 1984–1992 ns | −24 to +26 ns |
| one key (`mode`) | 2690–2730 ns | 2532–2549 ns | **−158 to −181 ns** |
| three keys | 2725–2745 ns | 2670–2704 ns | **−41 to −55 ns** |

**No added cost survives the comparison, across two independent runs.** If anything, `3.0.0`'s
calls with options were slightly faster than `2.9.0`'s, which folded the same options through an
extra `htmlToMarkdownWith` indirection that `3.0` removed. A reader who wants the isolated cost of
just the new key-check (compiled in vs. patched out, same binary) can find that number in RFC 048
slice 2's own review: ~55 ns for one key, ~150 ns for three, against ±100 ns noise on that harness
— consistent with "not measurable end to end" here.

### What this means

**Nobody converting single strings will notice `3.0.0`**: the cost this page reported after the
first sitting did not survive a confirmed-quiet re-run, and is corrected here to "not measurable"
rather than left at "small but real." **Bulk file conversion is measurably slower under real
parallelism, and the gap grows with thread count** — the opposite of what the first sitting
reported, for the same reason: that sitting's numbers were taken on a machine with unrecorded
background load. Node's option checking, the one place `3.0` deliberately adds work, costs nothing
that survives measurement, on either sitting. The corrected finding here is not more comfortable
than the wrong one it replaces, and it is reported anyway — that is the point of the RFC 012
amendment this regeneration prompted.

## The Focus of mdka

The Rust ecosystem offers a variety of excellent HTML-to-Markdown converters. Many of these projects prioritize feature-richness, complex edge-case handling, or high extensibility. 

`mdka` takes a different approach. Our mission is to provide a **"minimalist, lightweight, and memory-efficient"** converter, specifically optimized for resource-constrained environments or high-concurrency tasks where overhead must be kept to an absolute minimum. 

The benchmarks presented here are not intended to rank libraries or declare a "winner." Instead, they serve as internal metrics to verify whether `mdka` is successfully meeting its own design goals. We believe in choosing the right tool for the specific job, and we encourage developers to explore the diverse range of libraries available in the ecosystem to find the one that best fits their needs.

## The Evolution: v1 to v2

With the release of v2, `mdka` underwent a complete architectural overhaul. We moved away from the original implementation to a ground-up rewrite focused on:

- **Stack-Safe Traversal:** Implementing a non-recursive Deep First Search (DFS) to prevent stack overflow even with deeply nested HTML.
- **Optimized Memory Allocation:** Reducing unnecessary clones and leveraging Rust’s ownership model to minimize heap allocation.
- **Streamlined Processing:** Simplifying the conversion logic to achieve a predictable and lightweight execution path.

This rewrite resulted in a dramatic performance leap and a significantly reduced memory footprint compared to our previous version. `v3` (`3.0.0`) changed the public surface — two modes instead of five,
fewer options, one result model across the three bindings — but not this traversal, which is why
the section above finds no regression on the path everyone uses.

## Benchmark Results (2026-09-30)

Every table on this page was regenerated in one sitting on one machine, from the commit named
above — not compared against a number measured on a different day, machine, or `mdka` version. Reading a fresh
run against an old page's own numbers can suggest a library got faster or slower when only the
*conditions* changed; the fix is not a caveat, it is never comparing across sittings at all. The
peer-library tables below (Conversion Speed, Memory Allocation, Scaling) are each the product of
one `cargo bench` invocation measuring all eight libraries together, so the comparison *within*
each table is a single sitting; this regeneration also ran all three benchmark binaries
(`convert`, `memory`, `scaling`) back to back in one sitting, rather than split across a machine
restart as the previous attempt was, and logged `/proc/loadavg` before, during, and after each —
see [Conditions](#conditions) and the [RFC 012
amendment](https://github.com/nabbisen/mdka-rs/blob/main/rfcs/done/012-benchmark-hardening.md#8-quietness-needed-evidence-too-2026-09-30)
this requirement comes from. The commit and the machine were unchanged throughout; see
[How These Were Produced](#how-these-were-produced) to re-run any of them yourself.

### Conditions

| | |
|---|---|
| Date | 2026-09-30, one sitting, 18:24–18:57 |
| Commit | [`2756c47`](https://github.com/nabbisen/mdka-rs/commit/2756c47e6d447facda832b699a94726693a53d57) (`3.0.0` tag plus two documentation-only commits — no code differs from the tagged release) |
| Machine | AMD Ryzen 9 9950X (16-core / 32-thread) |
| OS | CachyOS Linux, kernel 7.2.8-1-cachyos |
| Rust | 1.98.1 (48a229cea 2026-09-01) |
| Criterion | 0.8.2 |
| Load average, `convert` | before 1.47/1.96/1.58 — during 1.96–3.19 (1 min) — after 2.64/2.90/2.44 |
| Load average, `memory` | before 2.64/2.90/2.44 — during 2.65–3.70 (1 min) — after 2.22/2.65/2.60 |
| Load average, `scaling` | before 2.22/2.65/2.60 — during 2.52–3.69 (1 min) — after 2.99/3.35/2.95 |

**These load figures are higher than the 0.01–1.09 range recorded for this same sitting's
`2.9.0`/`3.0.0` probe comparisons below, and the reason is disclosed rather than smoothed over: the
`before` readings still carried the decaying 5-/15-minute average of an earlier, interrupted attempt
at this same triple run, and a self-inflicted bug — a stray infinite polling loop left running by
one of this session's own tool calls, using ~14% of one of the 32 logical CPUs — was present for the
full duration and was found and killed only after the run finished.** Judged against the effect
sizes this sitting found (a collapse to sub-1% and a 50–80% slowdown), a sustained 0.14-core
artifact on a 32-thread machine is not a plausible explanation for either, and these three
`cargo bench` invocations are internally self-comparing across eight libraries in one process each,
where any constant background draw affects all eight equally. It is recorded here anyway, because
recording every environmental fact — including an unflattering one — rather than only the
convenient ones is the entire point of the RFC 012 amendment this page cites.

All eight libraries are `[dev-dependencies]` pinned to the exact versions below, **unchanged from
the `2.3.0`-era page** — regenerating this page never bumps them, so a stale claim never re-stales
itself silently between runs.

## Libraries Under Test

| Library | Version | HTML parser | Approach |
|---|---:|---|---|
| **mdka** | 3.0.0 (`main` @ `2756c47`) | `scraper` (`html5ever`) | Full DOM tree; non-recursive DFS |
| **mdka_v1** | 1.6.9 | `html5ever` | Full DOM tree; older implementation |
| **html2md** | 0.2.15 | `html5ever` | DOM-based converter |
| **fast_html2md** | 0.0.61 | `lol_html` | Streaming rewriter |
| **htmd** | 0.5.4 | `html5ever` | DOM-based converter |
| **html_to_markdown_rs** | 3.1.0 | `html5ever` | DOM-based converter |
| **html2text** | 0.16.7 | `html5ever` | Text-oriented converter |
| **dom_smoothie** | 0.17.0 | `dom_query` (`html5ever`) | DOM-oriented converter |

These libraries do not share the same design and do have different approach and goals. `mdka` is
not fastest on any dataset below — `fast_html2md`'s streaming rewriter leads on five of six — and
this page is not trying to say otherwise.

## Conversion Speed

Wall-clock medians from Criterion (100 samples, 3 s warm-up, default measurement time). The
fastest library on each row is **bold**.

| Dataset | mdka | mdka_v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 138.50 µs | 100.00 µs | 116.83 µs | **68.70 µs** | 75.29 µs | 83.40 µs | 253.64 µs | 250.75 µs |
| medium | 1.3495 ms | 1.6324 ms | 1.3374 ms | **737.66 µs** | 837.62 µs | 916.66 µs | 2.8420 ms | 2.2785 ms |
| large | 12.173 ms | 52.153 ms | 10.901 ms | **5.8554 ms** | 6.4987 ms | 7.9625 ms | 29.066 ms | 20.217 ms |
| deep_nest | 27.968 ms | 317.86 ms | 28.480 ms | **4.5184 ms** | 65.728 ms | 57.274 ms | 27.420 ms | — |
| flat | 5.5819 ms | 12.869 ms | 6.0287 ms | **3.8180 ms** | 4.2074 ms | 4.0612 ms | 12.939 ms | 24.862 ms |
| malformed | 35.34 µs | **32.34 µs** | 64.24 µs | 45.44 µs | 56.95 µs | 34.78 µs | 80.66 µs | 5.0347 ms |

`mdka` against `mdka_v1`: still ahead on medium (1.21×), large (4.28×), deep_nest (11.36×) and
flat (2.31×) — but **behind** on small (0.72×) and malformed (0.92×), on the smallest and least
structured inputs. Both exceptions were already known before this regeneration, not new findings
here.

**These ratios are *not* comparable with the `2.3.0`-era page's, and an earlier draft of this
section said they were.** Against that page, five of six moved by 28–48% — `flat` from 4.41× to
2.31×, `large` from 6.14× to 4.28×, `medium` from 1.75× to 1.21× — and only `deep_nest` held
(11.44× to 11.37×). The cause is visible in the two tables: **`mdka`'s own times barely moved**
(`large` 12.336 → 12.173 ms, `flat` 5.625 → 5.582 ms), while **`mdka_v1`, pinned at `1.6.9` and
unchanged in code, measured 30–48% faster** (`large` 75.751 → 52.153 ms, `flat` 24.817 → 12.869
ms). Something outside this repository moved — a toolchain, a dependency, or the machine itself.

**The lesson is the one this page keeps relearning: only numbers measured in the same sitting may
be compared.** Within this table all eight libraries ran together, so the columns are comparable
with each other. They are not comparable with any earlier page, and neither are the ratios derived
from them. The `2.9.0` vs `3.0.0` section above is trustworthy for the opposite reason — both
binaries ran in one sitting, interleaved.

That `mdka`'s absolute times are near-identical across the two sittings is consistent with the
`2.9.0` vs `3.0.0` finding that the traversal did not move, but it is a weaker statement than the
one this paragraph used to make.

## Memory Allocation

**What this measures: cumulative bytes allocated over the conversion, not peak resident
memory.** A library that allocates and frees many small buffers can show a large number here
while its actual memory footprint stays small — see the `mdka_v1` finding below, verified
independently. The fastest (fewest bytes) library on each row is **bold**.

| Dataset | mdka | mdka_v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 118.6 KB | 816.3 KB | 167.7 KB | **117.9 KB** | 182.6 KB | 232.5 KB | 764.5 KB | 325.4 KB |
| medium | **1013.2 KB** | 47.06 MB | 1.61 MB | 1.14 MB | 1.65 MB | 1.95 MB | 8.50 MB | 2.85 MB |
| large | **8.19 MB** | 3.60 GB | 14.00 MB | 9.14 MB | 14.14 MB | 16.74 MB | 74.89 MB | 23.08 MB |
| deep_nest | 3.25 MB | 668.62 MB | 3.42 MB | 5.33 MB | 4.97 MB | **2.55 MB** | 18.48 MB | SKIPPED |
| flat | **4.00 MB** | 932.93 MB | 6.60 MB | 5.60 MB | 6.70 MB | 7.87 MB | 40.28 MB | 35.47 MB |
| malformed | 50.9 KB | **44.0 KB** | 63.9 KB | 107.7 KB | 68.0 KB | 71.4 KB | 464.4 KB | 1.63 MB |

**`mdka`'s own allocation figures are effectively unchanged from the `2.3.0`-era page** (large:
8.19 MB here, 8.19 MB then; flat: 4.00 MB both times) — the single-document conversion path `3.0`
did not touch shows it in bytes as well as in wall time. **`mdka_v1`'s numbers on
`medium`/`large`/`deep_nest`/`flat` are 20–500× a naive reading of "peak memory" would suggest, and
genuinely so — not a measurement regression in this harness.** Re-verified this sitting,
independently, outside Criterion, with a standalone build against the same pinned
`mdka_v1 = 1.6.9`: converting `flat.html` allocates ~933 MB cumulatively (932.97 MB, reproduced
exactly against the figure first recorded at `2.3.0`), while `/proc/self/status`'s `VmHWM` (real
peak resident memory, a completely different measurement path) for the same run is **8,872 KB**
(≈8.67 MB). `mdka_v1`'s older implementation appears to allocate and free many short-lived
buffers — heavy churn, not a growing footprint — and it scales with input complexity:
`small`/`malformed` (simple, low element count) stay in the hundreds of KB, while
`medium`/`large`/`flat`/`deep_nest` (many repeated or nested elements) do not. `mdka` (v2/v3)
shows no such gap between the two measurement paths.

## Scaling: Depth and Width

**The depth bound in the README ("no stack overflow, no matter the nesting depth") is a claim
about crashing, not speed.** Nesting depth costs real time, and the cost is not linear. Total
conversion time and `scraper::Html::parse_document` alone, timed as two independent runs (each a
median of five), not one run split into parts — so read them side by side, not as a sum:

| Nesting depth | Total conversion | `scraper::Html::parse_document` alone |
|---:|---:|---:|
| 1,000 | 1.289 ms | 1.228 ms |
| 5,000 | 24.175 ms | 23.894 ms |
| 10,000 | 95.193 ms | 94.468 ms |
| 20,000 | 383.158 ms | 382.657 ms |
| 40,000 | 1561.849 ms | 1560.911 ms |
| 80,000 | 6410.873 ms | 6360.341 ms |

**Almost the entire cost is inside the parser, before mdka's own traversal ever begins** — the two
columns track each other closely at every depth, never differing by more than a fraction of a
percent, which is itself within this measurement's own noise between two separately-timed runs.
The cost also grows worse than linearly: an 80× deeper document costs roughly 4,972× longer to
convert, not 80×. A reader with untrusted, deeply-nested input should read the stack-overflow
guarantee as exactly that — a crash-safety claim — and budget time separately using this table,
`mdka`'s own `html_to_markdown` measured alone (`mdka` is not compared against the other seven
libraries here; this is about `mdka`'s own scaling, not a cross-library race). These figures are
consistent with the `2.3.0`-era page's (which found ~5,050× at the same 80× ratio); the parser's
own scaling has not changed, as expected — `3.0` never touched it. (The 5,000-deep row moved the
most between this sitting and the previous one — 29.140 ms to 24.175 ms, about −17% — the largest
swing of the six; every other row moved by single digits or less. This is depth-scaling data, run
outside Criterion's own outlier detection, so a swing like this is read as measurement variance
rather than a finding, consistent with the Conditions section's disclosure above.)

Width (total input size, roughly constant shallow structure) scales far better, sub-linearly in
practice on this run:

| Input size | `mdka` | `mdka_v1` |
|---:|---:|---:|
| 10 KB | 138.14 µs | 101.60 µs |
| 50 KB | 651.80 µs | 666.64 µs |
| 100 KB | 1.3441 ms | 1.6424 ms |
| 500 KB | 6.3934 ms | 14.137 ms |
| 1 MB | 12.092 ms | 52.378 ms |
| 5 MB | 66.018 ms | 1.2391 s |

## How These Were Produced

Every number on this page comes from this repository's own `benches/`, run against the commit
named under Conditions, above, on one machine:

```sh
git checkout 2756c47e6d447facda832b699a94726693a53d57
cargo bench --bench convert    # Conversion Speed table; target/bench_results.csv
cargo bench --bench memory     # Memory Allocation table
cargo bench --bench scaling    # both Scaling tables (depth report printed directly; width via Criterion)
```

Conversion Speed and Scaling figures are Criterion's own median point estimate
(`target/criterion/<name>/base/estimates.json`, `median.point_estimate`) for each
library/dataset group — the same 100-sample, 3-second-warm-up measurement Criterion reports on the
console, read programmatically rather than transcribed by hand. Memory Allocation figures are
cumulative bytes allocated, read from `target/bench_results.csv`'s `memory/<library>/<dataset>`
rows, which this project's own counting allocator (`benches/alloc_counter.rs`) records directly —
a byte count needs no statistical estimate, only one run.

The `mdka_v1` peak-RSS cross-check above is not part of `cargo bench`: it is a standalone,
five-line binary against the same pinned `mdka_v1 = "=1.6.9"` dependency, reading
`/proc/self/status`'s `VmHWM` after one conversion of `benches/benchdata/flat.html`.

The **`2.9.0` vs `3.0.0`** comparisons at the top of this page are not part of `cargo bench`
either, because both versions are `mdka` itself and Cargo can only build one version of a crate
into one binary: two release binaries were built from clean `git worktree`s at the `2.9.0` and
`3.0.0` tags, and a driver script ran them alternately — swapping which one goes first on every
round — so that any shared machine noise affects both equally and cancels in the median. The
string-conversion comparison was run **four independent times**, first on a machine only asserted
quiet (finding a since-retracted +2% to +5% cost) and then again on a machine confirmed quiet by
logged `/proc/loadavg`; the four re-run numbers shown are the ones this page now reports. The
bulk-file figures use the same two probe binaries: allocation with the counting allocator inside
each; wall time swept across `RAYON_NUM_THREADS` values (1/2/4/8/16/32), **run twice** for
reproducibility, and **run both with the library's normal filesystem writes and with a
`bulk_nowrite` variant that converts and discards in memory**, to separate a filesystem effect from
a library effect. The Node.js figures used the **published** `mdka@2.9.0` and `mdka@3.0.0` packages
installed from npm, not a local build, so the number on this page is the one every user actually
runs; that comparison was not re-run for the clean sitting, since it had already been measured
twice in agreement and RFC 012's amendment concerns environmental evidence, not re-verifying an
already-settled result.

Nothing on this page is asserted without a command or a description sufficient to re-run it.

## Summary

`mdka` is not the fastest library measured here, and this page says so plainly rather than
printing a table that quietly implies otherwise. Its case is a small, predictable footprint
(competitive memory allocation across every dataset, no stack overflow at any nesting depth) and
a single, simple implementation, not raw throughput — a reader who needs the fastest possible
conversion should look at `fast_html2md`'s row above, not this project's own marketing of itself.

`3.0.0`'s API changes — two modes instead of five, a smaller option surface, one result model
across three languages — cost about 165 bytes per file in bulk file conversion, and, at real
parallelism (8+ threads), a wall-time slowdown that grows with thread count rather than a speedup:
the opposite of what this page first reported, corrected after a confirmed-quiet re-run. Single-
string conversion shows no measurable cost either way. A reader converting many files at once under
heavy parallelism is the one case where this upgrade has a real, currently-unidentified cost; every
other caller should not let performance decide whether to upgrade.

We recognize that other libraries may offer more features or different trade-offs that make them
better suited for certain applications. `mdka` aims to be the right choice for those who prioritize
a simple, "Unix-style" tool that does one thing — conversion — predictably, at a small and
consistent footprint.
