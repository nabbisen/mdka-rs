# Benchmark Results

This chapter compares mdka with another Rust HTML-to-Markdown library —
`fast_html2md` — under controlled conditions.

The goal is to present the data fairly. These two libraries have different
internal designs and different strengths; the results reflect that honestly.

---

## Libraries Under Test

| Library | Version | HTML Parser | Approach |
|---|---|---|---|
| **mdka** | 0.3.0 | [scraper](https://crates.io/crates/scraper) (html5ever) | Full DOM tree; non-recursive DFS |
| **fast_html2md** | 0.0.61 | [lol_html](https://crates.io/crates/lol_html) (Cloudflare) | Streaming SAX-style rewriter |

**These are fundamentally different architectures**, and the tradeoffs are real:

- `lol_html` is a streaming rewriter that processes HTML in a single linear pass.
  It is very fast on typical inputs, but it does not build a full DOM tree.
- `html5ever` / scraper parses the full HTML5 DOM. It handles malformed HTML,
  missing tags, and deeply nested structures more faithfully, but carries
  more overhead.

---

## Test Environment

| Item | Value |
|---|---|
| OS | Ubuntu 24.04 |
| CPU | 2 logical cores |
| Rust | 1.84.1 |
| Benchmark harness | criterion 0.4 |
| Warm-up | 1 s |
| Measurement | 3 s |

All timings are **wall-clock median** of the criterion sample set.
Memory figures are **median heap allocation bytes** from a custom
`#[global_allocator]` counter (measured over 3 warmup + 3 measurement runs).

---

## Conversion Speed

| Dataset | Size | mdka | fast_html2md | Ratio |
|---|---|---|---|---|
| small | 12 KB | 578 µs | **180 µs** | fast 3.2× faster |
| medium | 102 KB | 5.55 ms | **1.84 ms** | fast 3.0× faster |
| large | 1 MB | 59.7 ms | **15.8 ms** | fast 3.8× faster |
| flat (3k elements) | 388 KB | 23.9 ms | **9.17 ms** | fast 2.6× faster |
| deep\_nest (5k levels) | 149 KB | **動作** | **CRASH** | mdka only |
| malformed | < 1 KB | 179 µs | **115 µs** | fast 1.6× faster |

On typical inputs, `fast_html2md` is **2–4× faster**. This is expected:
streaming rewriters skip the overhead of building and traversing a full DOM.

The `deep_nest` case is the exception: `fast_html2md` crashes with a stack
overflow at ~5,000 nesting levels. mdka handles it correctly because it uses
an iterative (non-recursive) traversal.

---

## Memory Allocation

| Dataset | mdka | fast_html2md | Ratio |
|---|---|---|---|
| small 11 KB | 240 KB | **154 KB** | fast uses 36% less |
| medium 99 KB | 2.03 MB | **1.52 MB** | fast uses 25% less |
| large 978 KB | 17.0 MB | **12.0 MB** | fast uses 29% less |
| flat 379 KB | 7.90 MB | **7.46 MB** | roughly equal |
| deep\_nest 145 KB | **4.71 MB** | 6.85 MB (CRASH) | mdka uses 31% less |
| malformed < 1 KB | **91.6 KB** | 145 KB | mdka uses 37% less |

On most inputs, `fast_html2md` allocates somewhat less memory.
On `deep_nest` and `malformed`, mdka allocates less — and is the only one
that completes successfully on deeply nested HTML.

---

## Scaling

Both libraries scale linearly (O(n)) with input size.
`fast_html2md` is consistently faster by a constant factor across all sizes.

| Input size | mdka | fast_html2md |
|---|---|---|
| 10 KB | 346 µs | 171 µs |
| 100 KB | 2.95 ms | 1.97 ms |
| 500 KB | 13.5 ms | 8.57 ms |
| 1 MB | 28.0 ms | 16.5 ms |

---

## Interpretation

These results are **reference data**, not a verdict.

`fast_html2md` is genuinely faster and uses less memory on well-formed, typical
HTML. If your primary need is maximum throughput on controlled input, it is a
strong choice.

mdka's position is different:

- It handles **any** HTML, including deeply nested, malformed, or SPA-generated
- It offers **five conversion modes** to tune pre-processing per use case
- It provides **consistent APIs** across Rust, Node.js, and Python
- Its output is **predictable** — the html5ever parser produces the same DOM
  from the same input, and the deterministic DFS produces the same Markdown

The right tool depends on your constraints. If you are processing user-submitted
web content, archiving arbitrary pages, or integrating with LLM pipelines where
stability matters, mdka's tradeoffs are worth considering.

---

## Running the Benchmarks Yourself

```bash
git clone https://github.com/example/mdka
cd mdka

# Speed benchmarks (mdka vs fast_html2md)
cargo bench --bench convert

# Memory benchmarks
cargo bench --bench memory

# Scaling analysis
cargo bench --bench scaling

# Allocation measurement tool
cargo run --example measure_mem
```

Results are saved to `target/bench_results.csv`.
