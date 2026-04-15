//! Benchmark: heap allocation per conversion (mdka vs comparison libraries)
//!
//! Uses CountingAllocator to measure bytes allocated during each call.
//! Results are printed as a summary table and appended to bench_results.csv.

use std::hint::black_box;
use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use mdka::alloc_counter::{AllocSnapshot, CountingAllocator};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

// ── sample helpers ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MemSample {
    allocated_bytes: usize,
    _alloc_count: u64,
    time_ns: u64,
}

fn sample_once<F: Fn()>(f: F) -> MemSample {
    let t = Instant::now();
    let before = AllocSnapshot::now();
    f();
    let after = AllocSnapshot::now();
    let ns = t.elapsed().as_nanos() as u64;
    let d = after.delta_since(&before);
    MemSample {
        allocated_bytes: d.allocated_bytes,
        _alloc_count: d.alloc_count as u64,
        time_ns: ns,
    }
}

fn median_sample<F: Fn() -> MemSample>(f: F, n: usize) -> MemSample {
    let mut v: Vec<_> = (0..n).map(|_| f()).collect();
    v.sort_by_key(|s| s.allocated_bytes);
    v.remove(n / 2)
}

fn sample_mdka(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_mdka(html));
            })
        },
        5,
    )
}
fn sample_html2md(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_html2md(html));
            })
        },
        5,
    )
}
fn sample_fast_html2md(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_fast_html2md(html));
            })
        },
        5,
    )
}
fn sample_htmd(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_htmd(html));
            })
        },
        5,
    )
}
fn sample_html_to_markdown_rs(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_html_to_markdown_rs(html));
            })
        },
        5,
    )
}
fn sample_html2text(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_html2text(html));
            })
        },
        5,
    )
}
fn sample_dom_smoothie(html: &str) -> MemSample {
    median_sample(
        || {
            sample_once(|| {
                let _ = black_box(run_dom_smoothie(html));
            })
        },
        5,
    )
}

fn fmt_bytes(b: usize) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else {
        format!("{:.1} KB", b as f64 / 1_024.0)
    }
}

// ── criterion benches ─────────────────────────────────────────────────────────

const SKIP_HTML2MD: [&str; 0] = [];
const SKIP_FAST_HTML2MD: [&str; 0] = [];
const SKIP_HTML_TO_MARKDOWN_RS: [&str; 0] = [];
const SKIP_HTML2TEXT: [&str; 0] = [];
const SKIP_DOM_SMOOTHIE: [&str; 1] = ["deep_nest"];
// const skip_deep_nest: [&str; 4] = [
//     "html2md",      // crashed
//     "fast_html2md", // crashed
//     "html2text",    // too slow
//     "dom_smoothie", // too slow
// ];

fn bench_memory_mdka(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/mdka/alloc");
    for (name, html) in &datasets {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_mdka(html));
            })
        });
    }
    group.finish();
}

fn bench_memory_html2md(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/html2md/alloc");
    for (name, html) in datasets.iter().filter(|(n, _)| !SKIP_HTML2MD.contains(n)) {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_html2md(html));
            })
        });
    }
    group.finish();
}

fn bench_memory_fast_html2md(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/fast_html2md/alloc");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_FAST_HTML2MD.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(fast_html2md::rewrite_html(html, false));
            })
        });
    }
    group.finish();
}

fn bench_memory_htmd(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/htmd/alloc");
    for (name, html) in &datasets {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_htmd(html));
            })
        });
    }
    group.finish();
}

fn bench_memory_html_to_markdown_rs(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/html-to-markdown-rs/alloc");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_HTML_TO_MARKDOWN_RS.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_html_to_markdown_rs(html));
            })
        });
    }
    group.finish();
}

fn bench_memory_html2text(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/html2text/alloc");
    for (name, html) in datasets.iter().filter(|(n, _)| !SKIP_HTML2TEXT.contains(n)) {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_html2text(html));
            })
        });
    }
    group.finish();
}

fn bench_memory_dom_smoothie(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("memory/dom_smoothie/alloc");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_DOM_SMOOTHIE.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
            b.iter(|| {
                let _ = black_box(run_dom_smoothie(html));
            })
        });
    }
    group.finish();
}

// ── summary table + CSV ───────────────────────────────────────────────────────

fn emit_memory_summary() {
    let datasets = load_datasets();

    println!("\n{:=<92}", "");
    println!(
        "  {:<12}  {:>8}  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}",
        "dataset", "size", "mdka", "fast_html2md", "htm2mdrs", "html2text", "dom_smoothie"
    );
    println!(
        "  {:-<12}  {:->8}  {:->12}  {:->12}  {:->12}  {:->12}  {:->12}",
        "", "", "", "", "", "", ""
    );

    let mut records = Vec::new();

    for (name, html) in &datasets {
        let s_mdka = sample_mdka(html);
        let s_html2md = sample_html2md(html);
        // let s_fast = if !skip_fh2md.contains(name) {
        //     Some(sample_fast_html2md(html))
        // } else {
        //     None
        // };
        let s_fast = sample_fast_html2md(html);
        let s_htmd = sample_htmd(html);
        let s_htm2 = sample_html_to_markdown_rs(html);
        let s_h2t = sample_html2text(html);
        let s_ds = if !SKIP_DOM_SMOOTHIE.contains(name) {
            Some(sample_dom_smoothie(html))
        } else {
            None
        };

        println!(
            "  {:<12}  {:>7}KB  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}",
            name,
            html.len() / 1024,
            fmt_bytes(s_mdka.allocated_bytes),
            fmt_bytes(s_html2md.allocated_bytes),
            // s_fast
            //     .as_ref()
            //     .map(|s| fmt_bytes(s.allocated_bytes))
            //     .unwrap_or("CRASH".into()),
            fmt_bytes(s_fast.allocated_bytes),
            fmt_bytes(s_htmd.allocated_bytes),
            fmt_bytes(s_htm2.allocated_bytes),
            fmt_bytes(s_h2t.allocated_bytes),
            s_ds.as_ref()
                .map(|s| fmt_bytes(s.allocated_bytes))
                .unwrap_or("TOO SLOW".into()),
        );

        let push = |records: &mut Vec<CsvRecord>, lib: &str, s: &MemSample| {
            records.push(CsvRecord {
                benchmark_name: format!("memory/{}/{}", lib, name),
                input_size: html.len(),
                threads: 1,
                time_ns: s.time_ns,
                memory_bytes: s.allocated_bytes,
            });
        };
        push(&mut records, "mdka", &s_mdka);
        push(&mut records, "html2md", &s_html2md);
        // if let Some(s) = &s_fast {
        //     push(&mut records, "fast_html2md", s);
        // }
        push(&mut records, "fast_html2md", &s_fast);
        push(&mut records, "htmd", &s_mdka);
        push(&mut records, "html_to_markdown_rs", &s_htm2);
        push(&mut records, "html2text", &s_h2t);
        // push(&mut records, "dom_smoothie", &s_ds);
        if let Some(s) = &s_ds {
            push(&mut records, "dom_smoothie", s);
        }
    }
    println!("{:=<92}\n", "");
    append_csv("target/bench_results.csv", &records);
    eprintln!(
        "[memory] CSV → target/bench_results.csv ({} rows)",
        records.len()
    );
}

fn setup(c: &mut Criterion) {
    print_env_info();
    emit_memory_summary();
    bench_memory_mdka(c);
    bench_memory_html2md(c);
    bench_memory_fast_html2md(c);
    bench_memory_htmd(c);
    bench_memory_html_to_markdown_rs(c);
    bench_memory_html2text(c);
    bench_memory_dom_smoothie(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
