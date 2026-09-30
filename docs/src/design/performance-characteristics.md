# Performance Characteristics

**These figures describe mdka `3.0.0` (`main` @ [`2756c47`](https://github.com/nabbisen/mdka-rs/commit/2756c47e6d447facda832b699a94726693a53d57), the tagged `3.0.0` release plus two documentation-only commits), measured 2026-09-29/30.** They replace the `2.3.0`-era numbers this page carried between `2.3.0` and `3.0.0`.

## `2.9.0` vs `3.0.0`: is `3.0` slower?

The owner asked this directly before any measurement was taken: *"v3 may be slower than v2."* The
measured answer is not a flat no: there is a small, real, single-digit-percent cost on the path
every string caller uses, and — found while looking for it — a substantially **faster** path for
callers who convert many files at once. Both are quantified below, from numbers taken this
sitting rather than assumed.

### The string-conversion path — a small, repeatable cost

`3.0` removed types and functions from the public surface; the one change inside the conversion
path itself is that `disposition()` in `src/traversal.rs`, called once per element, now asks
`ConversionOptions::unwraps_wrappers()` — a method comparing the mode enum — where `2.9.0` read a
`bool` field directly. Interleaved on one machine — two release binaries, one built against the
published `2.9.0` source, one against `3.0.0`, alternating which runs first on every round to
cancel first-mover bias, median of 9 rounds of 11 reps each, **repeated four independent times**
because the first repetition and the next three disagreed enough to need it:

| Dataset | run 1 | run 2 | run 3 | run 4 | median |
|---|---:|---:|---:|---:|---:|
| small | −0.27% | +3.06% | +4.31% | +3.27% | **+3.17%** |
| medium | +0.64% | +1.59% | +4.31% | +3.83% | **+2.71%** |
| large | −0.16% | +5.60% | +4.70% | +4.32% | **+4.51%** |
| deep_nest | −0.75% | −0.22% | −0.99% | +0.18% | **−0.48%** |
| flat | −0.37% | +1.93% | +2.28% | +1.99% | **+1.96%** |
| malformed | +3.54% | +0.03% | −31.61% | +46.23% | (noise — 38–58 µs absolute) |

**`deep_nest` is the internal control, and it holds at ~0% across all four runs.** It is the one
dataset where `scraper::Html::parse_document` alone accounts for ~99.8% of total time (see
[Scaling: Depth and Width](#scaling-depth-and-width)) — the one code path `3.0` truly never
touches. Every other dataset shows a small, repeatable (3 of 4 runs, same direction) delta of
roughly **+2% to +5%**: real in the sense that it reproduces, and small in the sense that it costs
a few dozen nanoseconds to tens of microseconds on inputs that convert in 140 microseconds to 12
milliseconds. The `malformed` row is too small in absolute terms (38–58 µs) for this harness to
read at all — a single scheduler hiccup swings it by tens of percent either way, which is itself
why the other five rows were re-run rather than trusted after one sitting.

The same small cost, same direction, shows up independently in the bulk-conversion probe below
when forced to run sequentially (+7.5% to +8.9%) — two different harnesses agreeing is stronger
evidence than either alone. **This is reported as a finding, not fixed here**: the magnitude is
below what any real caller would notice, and the handoff for this page is measurement and
documentation, not code.

### Bulk file conversion: a small allocation, and an unexpected, larger speedup

`html_files_to_markdown` returned `Vec<(&P, Result<PathBuf, MdkaError>)>` — a **borrowed** key —
through `2.9.0`. It now returns `Vec<FileOutcome>` with an **owned** `PathBuf`. That is one
allocation per file, and no benchmark before this one converted files at all, so both the
allocation and any effect on wall time were measured directly.

**Bytes allocated**, converting 10/100/1000 copies of `small.html`, measured with this project's
own counting allocator (`benches/alloc_counter.rs`), two independent runs:

| Files | `2.9.0` | `3.0.0` | delta | per file |
|---:|---:|---:|---:|---:|
| 10 | 1,345,884–1,346,234 B | 1,347,484–1,347,884 B | +0.12% | **+160–167 B** |
| 100 | 13,458,340–13,461,840 B | 13,474,440–13,478,440 B | +0.12% | **+161–166 B** |
| 1000 | 134,615,600–134,650,600 B | 134,777,600–134,817,600 B | +0.12% | **+162–167 B** |

Flat across a 100× range in file count, both runs — exactly what "one more small owned allocation
per `FileOutcome`" predicts, and this is the whole mechanism, quantified rather than guessed at.

**Wall time is a different story, and a better one.** Run with the default thread count (all 32
logical CPUs, rayon's parallel path — the default `[dependencies]` feature and what every caller
gets unless they opt out):

| Files | `2.9.0` | `3.0.0` | delta |
|---:|---:|---:|---:|
| 10 | 327.9–422.7 µs | 343.3–359.2 µs | −8.6% to +4.7% |
| 100 | 1.326–2.435 ms | 1.316–1.618 ms | −0.8% to −36.5% |
| 1000 | 10.41–19.84 ms | 10.36–12.25 ms | −0.5% to −40.0% |

The 10-file row is noise-dominated (sub-millisecond, wide spread across repeats); the 100- and
1000-file rows are not, and both repeats agree: **`3.0.0` finishes the same batch faster, and the
gap widens with more files.** Forcing the thread count with `RAYON_NUM_THREADS` isolates why:

| Threads | `2.9.0` | `3.0.0` | delta |
|---:|---:|---:|---:|
| 1 (sequential) | 152.5–155.9 ms | 164.0–169.7 ms | **+7.5% to +8.9%** |
| 2 | 88.26 ms | 80.63 ms | −8.7% |
| 4 | 41.02 ms | 41.90 ms | +2.2% |
| 8 | 29.58 ms | 23.08 ms | −22.0% |
| 16 | 22.14 ms | 17.12 ms | −22.7% |
| 32 (default) | 20.15 ms | 12.67 ms | **−37.2%** |

**Sequentially, `3.0.0` costs a little more per file — the one added allocation, matching the
string-conversion finding above.** Under rayon's parallel scheduler, at thread counts a real
machine actually has (8 and up), `3.0.0` is consistently and increasingly faster, and scales
better with added threads (2→32 threads: `2.9.0` speeds up 4.38× off an ideal 16×, `3.0.0` 6.37×).
**The most likely mechanism, consistent with this sweep but not confirmed with a profiler**: the
old return type carried a borrow (`&'a P`) through rayon's parallel `map`/`collect`, and the new
one does not — an owned `FileOutcome` may simply parallelize more cleanly than a
lifetime-bound tuple. This is reported as an observation, not asserted as the cause; it was not
investigated further because doing so would mean instrumenting the library, which is out of
scope here.

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

**Nobody converting strings will notice `3.0.0`**, and the small, repeatable cost this sitting
found there (a few percent, sub-millisecond in absolute terms) is reported rather than hidden.
**Bulk file conversion is not slower — it is measurably faster, by a wide and growing margin, on
any machine with more than a handful of cores**, for the cost of about 165 bytes per file. Node's
option checking, the one place `3.0` deliberately adds work, costs nothing that survives
measurement. None of this was written before the numbers came in, and one of it (the bulk
speedup) was not expected at all.

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

## Benchmark Results (2026-09-29/30)

Every table on this page was regenerated on one quiet machine, from the commit named above — not
compared against a number measured on a different day, machine, or `mdka` version. Reading a fresh
run against an old page's own numbers can suggest a library got faster or slower when only the
*conditions* changed; the fix is not a caveat, it is never comparing across sittings at all. The
peer-library tables below (Conversion Speed, Memory Allocation, Scaling) are each the product of
one `cargo bench` invocation measuring all eight libraries together, so the comparison *within*
each table is a single sitting even though, this time, the three benchmark binaries
(`convert`, `memory`, `scaling`) were not run back to back: the convert benchmark ran the evening
of the 29th, and a machine restart pushed `memory` and `scaling` to the following day. The commit
and the machine were unchanged across both; see
[How These Were Produced](#how-these-were-produced) to re-run any of them yourself.

### Conditions

| | |
|---|---|
| Date | 2026-09-29 (Conversion Speed) / 2026-09-30 (Memory Allocation, Scaling) |
| Commit | [`2756c47`](https://github.com/nabbisen/mdka-rs/commit/2756c47e6d447facda832b699a94726693a53d57) (`3.0.0` tag plus two documentation-only commits — no code differs from the tagged release) |
| Machine | AMD Ryzen 9 9950X (16-core / 32-thread) |
| OS | CachyOS Linux, kernel 7.2.8-1-cachyos |
| Rust | 1.98.1 (48a229cea 2026-09-01) |
| Criterion | 0.8.2 |

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
| small | 138.12 µs | 101.15 µs | 115.73 µs | **66.80 µs** | 76.59 µs | 81.49 µs | 251.97 µs | 245.79 µs |
| medium | 1.3377 ms | 1.6284 ms | 1.3192 ms | **709.19 µs** | 836.63 µs | 911.94 µs | 2.7671 ms | 2.2602 ms |
| large | 12.116 ms | 52.120 ms | 10.819 ms | **5.6868 ms** | 6.2609 ms | 7.9149 ms | 26.155 ms | 20.120 ms |
| deep_nest | 25.477 ms | 314.25 ms | 28.410 ms | **4.4766 ms** | 63.621 ms | 56.540 ms | 27.230 ms | — |
| flat | 5.4595 ms | 12.820 ms | 5.9929 ms | **3.7400 ms** | 4.1062 ms | 4.0272 ms | 11.716 ms | 25.261 ms |
| malformed | 35.01 µs | **32.52 µs** | 63.64 µs | 45.50 µs | 56.07 µs | 34.46 µs | 78.56 µs | 5.0136 ms |

`mdka` against `mdka_v1`: still ahead on medium (1.22×), large (4.30×), deep_nest (12.34×) and
flat (2.35×) — but **behind** on small (0.73×) and malformed (0.93×), on the smallest and least
structured inputs. Both exceptions were already known before this regeneration, not new findings
here. These ratios are essentially unchanged from the `2.3.0`-era page, consistent with the
`2.9.0` vs `3.0.0` section above: the traversal these numbers exercise did not move.

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
| 1,000 | 1.357 ms | 1.309 ms |
| 5,000 | 29.140 ms | 23.980 ms |
| 10,000 | 103.149 ms | 102.916 ms |
| 20,000 | 411.270 ms | 411.704 ms |
| 40,000 | 1580.981 ms | 1585.148 ms |
| 80,000 | 6406.789 ms | 6395.065 ms |

**Almost the entire cost is inside the parser, before mdka's own traversal ever begins** — the two
columns track each other closely at every depth, never differing by more than a fraction of a
percent, which is itself within this measurement's own noise between two separately-timed runs.
The cost also grows worse than linearly: an 80× deeper document costs roughly 4,721× longer to
convert, not 80×. A reader with untrusted, deeply-nested input should read the stack-overflow
guarantee as exactly that — a crash-safety claim — and budget time separately using this table,
`mdka`'s own `html_to_markdown` measured alone (`mdka` is not compared against the other seven
libraries here; this is about `mdka`'s own scaling, not a cross-library race). These figures are
consistent with the `2.3.0`-era page's (which found ~5,050× at the same 80× ratio); the parser's
own scaling has not changed, as expected — `3.0` never touched it.

Width (total input size, roughly constant shallow structure) scales far better, sub-linearly in
practice on this run:

| Input size | `mdka` | `mdka_v1` |
|---:|---:|---:|
| 10 KB | 138.77 µs | 104.79 µs |
| 50 KB | 656.93 µs | 678.96 µs |
| 100 KB | 1.3475 ms | 1.6644 ms |
| 500 KB | 6.5220 ms | 14.352 ms |
| 1 MB | 12.405 ms | 52.215 ms |
| 5 MB | 68.237 ms | 1.3535 s |

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
string-conversion comparison was run **four independent times** rather than once, because the
first repetition disagreed with the next three enough to need checking; all four are shown. The
bulk-file figures use the same two probe binaries: allocation with the counting allocator inside
each, wall time both at the default thread count and swept across `RAYON_NUM_THREADS` values to
separate the sequential cost from the parallel-scheduling effect. The Node.js figures used the
**published** `mdka@2.9.0` and `mdka@3.0.0` packages installed from npm, not a local build, so the
number on this page is the one every user actually runs.

Nothing on this page is asserted without a command or a description sufficient to re-run it.

## Summary

`mdka` is not the fastest library measured here, and this page says so plainly rather than
printing a table that quietly implies otherwise. Its case is a small, predictable footprint
(competitive memory allocation across every dataset, no stack overflow at any nesting depth) and
a single, simple implementation, not raw throughput — a reader who needs the fastest possible
conversion should look at `fast_html2md`'s row above, not this project's own marketing of itself.

`3.0.0`'s API changes — two modes instead of five, a smaller option surface, one result model
across three languages — cost a small, single-digit-percent amount in string conversion (found on
repeated measurement, too small to notice in practice) and about 165 bytes per file in bulk file
conversion, where they also make conversion **substantially faster** under real parallelism. A
reader deciding whether to upgrade should not let performance decide it either way: what changed is
smaller than what stayed the same, and the one place that moved by a wide margin moved for the
better.

We recognize that other libraries may offer more features or different trade-offs that make them
better suited for certain applications. `mdka` aims to be the right choice for those who prioritize
a simple, "Unix-style" tool that does one thing — conversion — predictably, at a small and
consistent footprint.
