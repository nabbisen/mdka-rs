# Performance Characteristics

## The Focus of mdka

The Rust ecosystem offers a variety of excellent HTML-to-Markdown converters. Many of these projects prioritize feature-richness, complex edge-case handling, or high extensibility. 

`mdka` takes a different approach. Our mission is to provide a **"minimalist, lightweight, and memory-efficient"** converter, specifically optimized for resource-constrained environments or high-concurrency tasks where overhead must be kept to an absolute minimum. 

The benchmarks presented here are not intended to rank libraries or declare a "winner." Instead, they serve as internal metrics to verify whether `mdka` is successfully meeting its own design goals. We believe in choosing the right tool for the specific job, and we encourage developers to explore the diverse range of libraries available in the ecosystem to find the one that best fits their needs.

## The Evolution: v1 to v2

With the release of v2, `mdka` underwent a complete architectural overhaul. We moved away from the original implementation to a ground-up rewrite focused on:

- **Stack-Safe Traversal:** Implementing a non-recursive Deep First Search (DFS) to prevent stack overflow even with deeply nested HTML.
- **Optimized Memory Allocation:** Reducing unnecessary clones and leveraging Rust’s ownership model to minimize heap allocation.
- **Streamlined Processing:** Simplifying the conversion logic to achieve a predictable and lightweight execution path.

This rewrite resulted in a dramatic performance leap and a significantly reduced memory footprint compared to our previous version.

## Benchmark Results (2026-09-23)

Every table on this page was regenerated in one sitting, on one quiet machine, from the commit
below — not compared against a number measured on a different day, machine, or `mdka` version.
Reading a fresh run against an old page's own numbers can suggest a library got faster or slower
when only the *conditions* changed; the fix is not a caveat, it is never comparing across
sittings at all. Every figure below came from this one sitting — see
[How These Were Produced](#how-these-were-produced) to re-run any of them yourself.

### Conditions

| | |
|---|---|
| Date | 2026-09-23 |
| Commit | [`c9cbbb8`](https://github.com/nabbisen/mdka-rs/commit/c9cbbb875264f0cae3b29613a9dc9d4e6aed6501) |
| Machine | AMD Ryzen 9 9950X (16-core / 32-thread) |
| OS | CachyOS Linux, kernel 7.2.6-1-cachyos |
| Rust | 1.98.1 (48a229cea 2026-09-01) |
| Criterion | 0.8.2 |

All eight libraries are `[dev-dependencies]` pinned to the exact versions below; regenerating
this page never bumps them, so a stale claim never re-stales itself silently between runs.

## Libraries Under Test

| Library | Version | HTML parser | Approach |
|---|---:|---|---|
| **mdka** | 2.3.0 (`main` @ `c9cbbb8`) | `scraper` (`html5ever`) | Full DOM tree; non-recursive DFS |
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
| small | 134.66 µs | 99.97 µs | 115.01 µs | **68.78 µs** | 74.90 µs | 82.65 µs | 252.37 µs | 248.04 µs |
| medium | 1.3071 ms | 1.6261 ms | 1.3082 ms | **730.11 µs** | 829.69 µs | 927.73 µs | 2.7784 ms | 2.2611 ms |
| large | 11.769 ms | 50.456 ms | 10.733 ms | **5.8083 ms** | 6.2969 ms | 7.9239 ms | 25.761 ms | 19.604 ms |
| deep_nest | 25.517 ms | 312.64 ms | 28.128 ms | **4.4512 ms** | 64.934 ms | 56.091 ms | 27.660 ms | — |
| flat | 5.4251 ms | 12.716 ms | 5.9680 ms | **3.8046 ms** | 4.1382 ms | 4.0237 ms | 11.707 ms | 24.569 ms |
| malformed | 35.14 µs | **32.46 µs** | 63.08 µs | 44.17 µs | 55.14 µs | 34.89 µs | 79.79 µs | 4.9107 ms |

`mdka` against `mdka_v1`: still ahead on medium (1.24×), large (4.29×), deep_nest (12.25×) and
flat (2.34×) — but **behind** on small (0.74×) and malformed (0.92×), on the smallest and least
structured inputs. Both exceptions were already known before this regeneration, not new findings
here.

## Memory Allocation

**What this measures: cumulative bytes allocated over the conversion, not peak resident
memory.** A library that allocates and frees many small buffers can show a large number here
while its actual memory footprint stays small — see the `mdka_v1` finding below, verified
independently. The fastest (fewest bytes) library on each row is **bold**.

| Dataset | mdka | mdka_v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 118.5 KB | 816.3 KB | 167.7 KB | **117.9 KB** | 182.6 KB | 232.5 KB | 764.5 KB | 325.4 KB |
| medium | **1013.0 KB** | 47.06 MB | 1.61 MB | 1.14 MB | 1.65 MB | 1.95 MB | 8.50 MB | 2.85 MB |
| large | **8.19 MB** | 3.60 GB | 14.00 MB | 9.14 MB | 14.14 MB | 16.74 MB | 74.89 MB | 23.08 MB |
| deep_nest | 3.25 MB | 668.62 MB | 3.42 MB | 5.33 MB | 4.97 MB | **2.55 MB** | 18.48 MB | SKIPPED |
| flat | **4.00 MB** | 932.93 MB | 6.60 MB | 5.60 MB | 6.70 MB | 7.87 MB | 40.28 MB | 35.47 MB |
| malformed | 50.7 KB | **44.0 KB** | 63.9 KB | 107.7 KB | 68.0 KB | 71.4 KB | 464.4 KB | 1.63 MB |

**`mdka_v1`'s numbers on `medium`/`large`/`deep_nest`/`flat` are 20–500× the `2.0.0`-era page's
own figures for it, and genuinely so — not a measurement regression in this harness.** Verified
independently, outside Criterion, with a standalone build against the same pinned `mdka_v1 =
1.6.9`: converting `flat.html` allocates ~933 MB cumulatively, confirmed reproducible across five
runs, while `/proc/self/status`'s `VmHWM` (real peak resident memory, a completely different
measurement path) for the same run is **~8.9 MB**. `mdka_v1`'s older implementation appears to
allocate and free many short-lived buffers — heavy churn, not a growing footprint — and it scales
with input complexity: `small`/`malformed` (simple, low element count) stay in the hundreds of
KB, while `medium`/`large`/`flat`/`deep_nest` (many repeated or nested elements) do not. `mdka`
(v2) shows no such gap between the two measurement paths and tracks the `2.0.0`-era numbers
closely (`large`: 8.19 MB here vs. 8.00 MB then).

## Scaling: Depth and Width

**The depth bound in the README ("no stack overflow, no matter the nesting depth") is a claim
about crashing, not speed.** Nesting depth costs real time, and the cost is not linear. Total
conversion time and `scraper::Html::parse_document` alone, timed as two independent runs (each a
median of five), not one run split into parts — so read them side by side, not as a sum:

| Nesting depth | Total conversion | `scraper::Html::parse_document` alone |
|---:|---:|---:|
| 1,000 | 1.284 ms | 1.284 ms |
| 5,000 | 26.988 ms | 26.716 ms |
| 10,000 | 104.794 ms | 104.248 ms |
| 20,000 | 442.196 ms | 444.457 ms |
| 40,000 | 1665.205 ms | 1668.038 ms |
| 80,000 | 6481.221 ms | 6457.919 ms |

**Almost the entire cost is inside the parser, before mdka's own traversal ever begins** — the two
columns track each other closely at every depth, never differing by more than a fraction of a
percent, which is itself within this measurement's own noise between two separately-timed runs.
The cost also grows worse than linearly: an 80× deeper document costs roughly 5,050× longer to
convert, not 80×. A reader with untrusted, deeply-nested input should read the stack-overflow
guarantee as exactly that — a crash-safety claim — and budget time separately using this table,
`mdka`'s own `html_to_markdown` measured alone (`mdka` is not compared against the other seven
libraries here; this is about `mdka`'s own scaling, not a cross-library race).

Width (total input size, roughly constant shallow structure) scales far better, sub-linearly in
practice on this run:

| Input size | `mdka` | `mdka_v1` |
|---:|---:|---:|
| 10 KB | 132.83 µs | 99.78 µs |
| 50 KB | 629.07 µs | 653.59 µs |
| 100 KB | 1.2862 ms | 1.6341 ms |
| 500 KB | 6.1595 ms | 13.790 ms |
| 1 MB | 11.607 ms | 50.424 ms |
| 5 MB | 63.742 ms | 1.3023 s |

## How These Were Produced

Every number on this page comes from this repository's own `benches/`, run against the commit
named under Conditions, above, on one machine, in one sitting:

```sh
git checkout c9cbbb875264f0cae3b29613a9dc9d4e6aed6501
cargo bench --bench convert    # Conversion Speed table; target/bench_results.csv
cargo bench --bench memory     # Memory Allocation table
cargo bench --bench scaling    # both Scaling tables (depth report printed directly; width via Criterion)
```

The `mdka_v1` peak-RSS cross-check above is not part of `cargo bench`: it is a standalone,
five-line binary against the same pinned `mdka_v1 = "=1.6.9"` dependency, reading
`/proc/self/status`'s `VmHWM` after one conversion of `benches/benchdata/flat.html`. Nothing on
this page is asserted without a command or a description sufficient to re-run it.

## Summary

`mdka` is not the fastest library measured here, and this page says so plainly rather than
printing a table that quietly implies otherwise. Its case is a small, predictable footprint
(competitive memory allocation across every dataset, no stack overflow at any nesting depth) and
a single, simple implementation, not raw throughput — a reader who needs the fastest possible
conversion should look at `fast_html2md`'s row above, not this project's own marketing of itself.

We recognize that other libraries may offer more features or different trade-offs that make them
better suited for certain applications. `mdka` aims to be the right choice for those who prioritize
a simple, "Unix-style" tool that does one thing — conversion — predictably, at a small and
consistent footprint.
