# Benchmark Results

This section summarizes the latest run as reference data, not as a verdict.

## Test Setup

All libraries were benchmarked under the same conditions: Linux x86_64 6.19, Rust 1.94.1, Criterion 0.8, 28 logical cores, 3 s warm-up, and 3 s measurement. The figures below are wall-clock medians from Criterion. The log also records outliers for each run, so small differences should be read with some caution.

## Libraries Under Test

| Library | Version | HTML parser | Approach |
|---|---:|---|---|
| **mdka** | 2.0.0-rc.3 | `scraper` (`html5ever`) | Full DOM tree; non-recursive DFS |
| **mdka_v1** | 1.6.9 | `html5ever` | Full DOM tree; older implementation |
| **html2md** | 0.2.15 | `html5ever`-based | DOM-based converter |
| **fast_html2md** | 0.0.61 | `lol_html` | Streaming rewriter |
| **htmd** | 0.5.4 | `html5ever`-based | DOM-based converter |
| **html_to_markdown_rs** | 3.1.0 | `html5ever`-based | DOM-based converter |
| **html2text** | 0.16.7 | parser not stated in the log | Text-oriented converter |
| **dom_smoothie** | 0.17.0 | parser not stated in the log | DOM-oriented converter |

These libraries do not share the same design. `lol_html` is a streaming rewriter and can be very fast on clean input. mdka uses a full HTML5 parse through `scraper` / `html5ever`, which adds overhead, but it is a better fit when malformed HTML, deep nesting, or stable output matter.

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

`fast_html2md` as a streaming rewriter remains the fastest option on the more typical clean-input cases here.

## Memory Allocation

| Dataset | mdka v2 | mdka_v1 | html2md | fast_html2md | htmd | html_to_markdown_rs | html2text | dom_smoothie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 113.5 KB | 240 KB | 231 KB | 154 KB | **93.6 KB** | 232.5 KB | 764.5 KB | 325.4 KB |
| medium | **984.6 KB** | 2.03 MB | 1.95 MB | 1.52 MB | 1.01 MB | 1.95 MB | 8.50 MB | 2.85 MB |
| large | 8.00 MB | 17.0 MB | 16.76 MB | 11.98 MB | **7.85 MB** | 16.76 MB | 74.89 MB | 23.08 MB |
| deep_nest | 3.00 MB | 4.71 MB | 2.55 MB | 6.85 MB | **1.96 MB** | 2.55 MB | 18.48 MB | — |
| flat | **3.93 MB** | 7.90 MB | 7.87 MB | 7.46 MB | 4.84 MB | 7.87 MB | 40.28 MB | 35.47 MB |
| malformed | **44.7 KB** | 91.6 KB | 71.4 KB | 145 KB | 62.3 KB | 71.4 KB | 464.4 KB | 1.63 MB |

In this run, mdka v2 uses less heap than `fast_html2md` on every listed dataset, and it stays well below `html2text` and `dom_smoothie`. `html_to_markdown_rs` is smaller on `deep_nest`, but mdka still keeps its allocation profile compact there while using the same non-recursive traversal strategy.

## Takeaway

The latest run suggests a more settled position for mdka v2 than v1. It is not the fastest converter in the set, and it does not need to be. The project still appears to be aiming for a quieter balance: stable output from HTML5 parsing, crash resistance, predictable traversal, and a memory profile that stays restrained across a wide range of inputs.

For clean HTML where peak throughput is the only concern, `fast_html2md` still deserves a careful look. For projects that value resilience, mode control, and steady behavior across messy input, mdka v2 looks more mature than mdka v1 in this benchmark set.

`htmd` is also competitive, especially on medium and large inputs, and shows a relatively efficient memory profile on several datasets.
