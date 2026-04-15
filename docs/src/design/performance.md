# Performance Concern

## The Focus of mdka

The Rust ecosystem offers a variety of excellent HTML-to-Markdown converters. Many of these projects prioritize feature-richness, complex edge-case handling, or high extensibility. 

`mdka` takes a different approach. Our mission is to provide a **"minimalist, lightweight, and memory-efficient"** converter, specifically optimized for resource-constrained environments or high-concurrency tasks where overhead must be kept to an absolute minimum. 

The benchmarks presented here are not intended to rank libraries or declare a "winner." Instead, they serve as internal metrics to verify whether `mdka` is successfully meeting its own design goals. We believe in choosing the right tool for the specific job, and we encourage developers to explore the diverse range of libraries available in the ecosystem to find the one that best fits their needs.

## The Evolution: v1 to v2

With the release of v2, `mdka` underwent a complete architectural overhaul. We moved away from the original implementation to a ground-up rewrite focused on:

- **Stack-Safe Traversal:** Implementing a non-recursive Deep First Search (DFS) to prevent stack overflow even with deeply nested HTML.
- **Optimized Memory Allocation:** Reducing unnecessary clones and leveraging Rust’s ownership model to minimize peak memory usage.
- **Streamlined Processing:** Simplifying the conversion logic to achieve a predictable and lightweight execution path.

This rewrite resulted in a dramatic performance leap and a significantly reduced memory footprint compared to our previous version.

## Benchmark Results

The following data demonstrates how the v2 architecture has improved our efficiency and how it aligns with our goal of "reasonable speed with minimal resource consumption."

All libraries were benchmarked under the same conditions: Linux x86_64 6.19, Rust 1.94.1, Criterion 0.8, 28 logical cores, 3 s warm-up, and 3 s measurement. The figures below are wall-clock medians from Criterion. The log also records outliers for each run, so small differences should be read with some caution.

## Libraries Under Test

| Library | Version | HTML parser | Approach |
|---|---:|---|---|
| **mdka** | 2.0.0 | `scraper` (`html5ever`) | Full DOM tree; non-recursive DFS |
| **mdka_v1** | 1.6.9 | `html5ever` | Full DOM tree; older implementation |
| **html2md** | 0.2.15 | `html5ever` | DOM-based converter |
| **fast_html2md** | 0.0.61 | `lol_html` | Streaming rewriter |
| **htmd** | 0.5.4 | `html5ever` | DOM-based converter |
| **html_to_markdown_rs** | 3.1.0 | `html5ever` | DOM-based converter |
| **html2text** | 0.16.7 | `html5ever` | Text-oriented converter |
| **dom_smoothie** | 0.17.0 | `dom_query` (`html5ever`) | DOM-oriented converter |

These libraries do not share the same design and do have different approach and goals.

## Conversion Speed

| Dataset | mdka v2 | mdka v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 131.52 µs | 131.66 µs | 132.21 µs | **79.50 µs** | 90.47 µs | 107.82 µs | 350.92 µs | 317.37 µs |
| medium | 1.3040 ms | 2.2866 ms | 1.5266 ms | **887.59 µs** | 1.0562 ms | 1.1660 ms | 3.3999 ms | 2.7643 ms |
| large | 12.336 ms | 75.751 ms | 12.455 ms | **7.0399 ms** | 7.7896 ms | 9.6825 ms | 29.854 ms | 26.062 ms |
| deep_nest | 32.620 ms | 373.10 ms | 36.834 ms | **5.9868 ms** | 72.481 ms | 96.744 ms | 30.903 ms | 29.408 ms |
| flat | 5.6253 ms | 24.817 ms | 6.7911 ms | **4.2114 ms** | 5.5321 ms | 4.6975 ms | 14.023 ms | 29.408 ms |
| malformed | **31.712 µs** | 40.178 µs | 71.778 µs | 52.948 µs | 62.302 µs | 41.109 µs | 96.822 µs | 5.6401 ms |

mdka v2 is clearly ahead of mdka v1 in this run. The gain is small on the smallest input, but it becomes much more visible as the input gets larger or structurally harder: around 1.75× faster on medium, 6.1× on large, 11.4× on deep_nest, and 4.4× on flat. On malformed input, v2 is also faster than v1 and the fastest.

## Memory Allocation

| Dataset | mdka v2 | mdka_v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 113.5 KB | 240 KB | 231 KB | 154 KB | **93.6 KB** | 232.5 KB | 764.5 KB | 325.4 KB |
| medium | **984.6 KB** | 2.03 MB | 1.95 MB | 1.52 MB | 1.01 MB | 1.95 MB | 8.50 MB | 2.85 MB |
| large | 8.00 MB | 17.0 MB | 16.76 MB | 11.98 MB | **7.85 MB** | 16.76 MB | 74.89 MB | 23.08 MB |
| deep_nest | 3.00 MB | 4.71 MB | 2.55 MB | 6.85 MB | **1.96 MB** | 2.55 MB | 18.48 MB | — |
| flat | **3.93 MB** | 7.90 MB | 7.87 MB | 7.46 MB | 4.84 MB | 7.87 MB | 40.28 MB | 35.47 MB |
| malformed | **44.7 KB** | 91.6 KB | 71.4 KB | 145 KB | 62.3 KB | 71.4 KB | 464.4 KB | 1.63 MB |

In this run, mdka v2 uses less heap than v1.

## Summary

As shown in the results, the transition to v2 has allowed us to achieve our objectives of being lightweight and memory-efficient while maintaining competitive speed. 

We recognize that other libraries may offer more features or different trade-offs that make them better suited for certain applications. `mdka` aims to be the best choice for those who prioritize a simple, "Unix-style" tool that does one thing—conversion—with the smallest possible footprint.
