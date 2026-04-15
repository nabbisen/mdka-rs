//! Benchmark: single-pass conversion (mdka vs comparison libraries)
//!
//! Libraries under test:
//!   mdka              – scraper (html5ever), iterative DFS
//!   html2md           – lol_html streaming rewriter
//!   fast_html2md      – lol_html streaming rewriter
//!   htmd              – htmd 0.5.4 (html5ever)
//!   html_to_markdown  – html-to-markdown-rs 3.1.0 (html5ever)
//!   html2text         – html2text 0.16.7 (html5ever, outputs plain text)
//!   dom_smoothie      – dom_smoothie 0.17.0 (article extraction, text output)
//!
//! deep_nest is excluded from libraries that crash on recursive traversal.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

// ── skip config ───────────────────────────────────────────────────

// fast_html2md and html_to_markdown_rs crash on deep_nest
const SKIP_HTML2MD: [&str; 0] = [];
const SKIP_FAST_HTML2MD: [&str; 0] = [];
const SKIP_HTML_TO_MARKDOWN_RS: [&str; 0] = [];
const SKIP_HTML2TEXT: [&str; 0] = [];
const SKIP_DOM_SMOOTHIE: [&str; 1] = ["deep_nest"];
// const skip_deep_nest = [
//     "html2md",      // crashed
//     "fast_html2md", // crashed
//     "html2text",    // too slow
//     "dom_smoothie", // too slow
// ];

// ── individual benches ───────────────────────────────────────────────────────

fn bench_mdka(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("mdka");
    for (name, html) in &datasets {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_mdka(black_box(html)))
        });
    }
    group.finish();
}

fn bench_html2md(c: &mut Criterion) {
    let datasets = load_datasets();
    // crashes on deep_nest (stack overflow in recursive traversal)
    // let skip = ["deep_nest"];
    let mut group = c.benchmark_group("html2md");
    for (name, html) in datasets.iter().filter(|(n, _)| !SKIP_HTML2MD.contains(n)) {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_html2md(black_box(html)))
        });
    }
    group.finish();
}

fn bench_fast_html2md(c: &mut Criterion) {
    let datasets = load_datasets();
    // crashes on deep_nest (stack overflow in recursive traversal)
    // let skip = ["deep_nest"];
    let mut group = c.benchmark_group("fast_html2md");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_FAST_HTML2MD.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_fast_html2md(black_box(html)))
        });
    }
    group.finish();
}

fn bench_htmd(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("htmd");
    for (name, html) in &datasets {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_htmd(black_box(html)))
        });
    }
    group.finish();
}

fn bench_html_to_markdown_rs(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("html_to_markdown_rs");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_HTML_TO_MARKDOWN_RS.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_html_to_markdown_rs(black_box(html)))
        });
    }
    group.finish();
}

fn bench_html2text(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("html2text");
    for (name, html) in datasets.iter().filter(|(n, _)| !SKIP_HTML2TEXT.contains(n)) {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_html2text(black_box(html)))
        });
    }
    group.finish();
}

fn bench_dom_smoothie(c: &mut Criterion) {
    let datasets = load_datasets();
    let mut group = c.benchmark_group("dom_smoothie");
    for (name, html) in datasets
        .iter()
        .filter(|(n, _)| !SKIP_DOM_SMOOTHIE.contains(n))
    {
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
            b.iter(|| run_dom_smoothie(black_box(html)))
        });
    }
    group.finish();
}

// ── compare (side-by-side) ───────────────────────────────────────────────────

fn bench_compare(c: &mut Criterion) {
    let datasets = load_datasets();
    let targets = ["small", "medium", "large", "flat", "malformed"];

    for target in &targets {
        let html = match datasets.iter().find(|(n, _)| n == target) {
            Some((_, h)) => h,
            None => continue,
        };
        let mut group = c.benchmark_group(format!("compare/{}", target));
        group.throughput(Throughput::Bytes(html.len() as u64));

        group.bench_function("mdka", |b| b.iter(|| run_mdka(black_box(html))));
        if !SKIP_HTML2MD.contains(target) {
            group.bench_function("html2md", |b| b.iter(|| run_html2md(black_box(html))));
        }
        if !SKIP_FAST_HTML2MD.contains(target) {
            group.bench_function("fast_html2md", |b| {
                b.iter(|| run_fast_html2md(black_box(html)))
            });
        }
        group.bench_function("htmd", |b| b.iter(|| run_htmd(black_box(html))));
        if !SKIP_HTML_TO_MARKDOWN_RS.contains(target) {
            group.bench_function("html_to_markdown_rs", |b| {
                b.iter(|| run_html_to_markdown_rs(black_box(html)))
            });
        }
        if !SKIP_HTML2TEXT.contains(target) {
            group.bench_function("html2text", |b| b.iter(|| run_html2text(black_box(html))));
        }
        if !SKIP_DOM_SMOOTHIE.contains(target) {
            group.bench_function("dom_smoothie", |b| {
                b.iter(|| run_dom_smoothie(black_box(html)))
            });
        }

        group.finish();
    }
}

// ── CSV output ───────────────────────────────────────────────────────────────

fn emit_convert_csv() {
    let datasets = load_datasets();
    let mut records = Vec::new();

    for (name, html) in &datasets {
        let ns = |f: &dyn Fn()| wall_median_ns(f, 7);

        records.push(CsvRecord {
            benchmark_name: format!("convert/mdka/{}", name),
            input_size: html.len(),
            threads: 1,
            time_ns: ns(&|| {
                let _ = run_mdka(html);
            }),
            memory_bytes: 0,
        });
        if !SKIP_HTML2MD.contains(name) {
            records.push(CsvRecord {
                benchmark_name: format!("convert/html2md/{}", name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns(&|| {
                    let _ = run_html2md(html);
                }),
                memory_bytes: 0,
            });
        }
        if !SKIP_FAST_HTML2MD.contains(name) {
            records.push(CsvRecord {
                benchmark_name: format!("convert/fast_html2md/{}", name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns(&|| {
                    let _ = run_fast_html2md(html);
                }),
                memory_bytes: 0,
            });
        }
        records.push(CsvRecord {
            benchmark_name: format!("convert/htmd/{}", name),
            input_size: html.len(),
            threads: 1,
            time_ns: ns(&|| {
                let _ = run_htmd(html);
            }),
            memory_bytes: 0,
        });
        if !SKIP_HTML_TO_MARKDOWN_RS.contains(name) {
            records.push(CsvRecord {
                benchmark_name: format!("convert/html_to_markdown_rs/{}", name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns(&|| {
                    let _ = run_html_to_markdown_rs(html);
                }),
                memory_bytes: 0,
            });
        }
        if !SKIP_HTML2TEXT.contains(name) {
            records.push(CsvRecord {
                benchmark_name: format!("convert/html2text/{}", name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns(&|| {
                    let _ = run_html2text(html);
                }),
                memory_bytes: 0,
            });
        }
        if !SKIP_DOM_SMOOTHIE.contains(name) {
            records.push(CsvRecord {
                benchmark_name: format!("convert/dom_smoothie/{}", name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns(&|| {
                    let _ = run_dom_smoothie(html);
                }),
                memory_bytes: 0,
            });
        }
    }
    append_csv("target/bench_results.csv", &records);
    eprintln!(
        "[convert] CSV → target/bench_results.csv ({} rows)",
        records.len()
    );
}

fn setup(c: &mut Criterion) {
    print_env_info();
    emit_convert_csv();
    bench_mdka(c);
    bench_html2md(c);
    bench_fast_html2md(c);
    bench_htmd(c);
    bench_html_to_markdown_rs(c);
    bench_html2text(c);
    bench_dom_smoothie(c);
    bench_compare(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
