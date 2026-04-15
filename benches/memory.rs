use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Instant;

use mdka::alloc_counter::{AllocSnapshot, CountingAllocator};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

// ── サンプリング処理の共通化 ─────────────────────────────────────────────

#[derive(Clone)]
struct MemSample {
    allocated_bytes: usize,
    _alloc_count: u64,
    time_ns: u64,
}

fn sample_target(run_fn: fn(&str) -> String, html: &str) -> MemSample {
    let mut v: Vec<_> = (0..5)
        .map(|_| {
            let t = Instant::now();
            let before = AllocSnapshot::now();
            let _ = black_box(run_fn(html));
            let after = AllocSnapshot::now();

            let d = after.delta_since(&before);
            MemSample {
                allocated_bytes: d.allocated_bytes,
                _alloc_count: d.alloc_count as u64,
                time_ns: t.elapsed().as_nanos() as u64,
            }
        })
        .collect();

    v.sort_by_key(|s| s.allocated_bytes);
    v.remove(v.len() / 2) // 中央値
}

fn fmt_bytes(b: usize) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else {
        format!("{:.1} KB", b as f64 / 1_024.0)
    }
}

// ── ベンチマーク実行 ─────────────────────────────────────────────────────

fn bench_memory_all(c: &mut Criterion) {
    let datasets = load_datasets();

    for target in TARGETS {
        let mut group = c.benchmark_group(format!("memory/{}/alloc", target.name));
        for (name, html) in &datasets {
            if is_skipped(target.name, name) {
                continue;
            }
            group.throughput(Throughput::Bytes(html.len() as u64));
            group.bench_with_input(BenchmarkId::new("", name), html, |b, html| {
                b.iter(|| {
                    let _ = black_box((target.run_fn)(html));
                })
            });
        }
        group.finish();
    }
}

// ── サマリーテーブル + CSV 出力 ──────────────────────────────────────────

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
        let mut samples = std::collections::HashMap::new();

        for target in TARGETS {
            if !is_skipped(target.name, name) {
                let s = sample_target(target.run_fn, html);
                samples.insert(target.name, s.clone());

                records.push(CsvRecord {
                    benchmark_name: format!("memory/{}/{}", target.name, name),
                    input_size: html.len(),
                    threads: 1,
                    time_ns: s.time_ns,
                    memory_bytes: s.allocated_bytes,
                });
            }
        }

        let get_fmt = |t: &str| {
            samples.get(t).map_or_else(
                || {
                    if is_skipped(t, name) {
                        "SKIPPED".into()
                    } else {
                        "-".into()
                    }
                },
                |s| fmt_bytes(s.allocated_bytes),
            )
        };

        println!(
            "  {:<12}  {:>7}KB  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}",
            name,
            html.len() / 1024,
            get_fmt("mdka"),
            get_fmt("fast_html2md"),
            get_fmt("html_to_markdown_rs"),
            get_fmt("html2text"),
            get_fmt("dom_smoothie"),
        );
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
    bench_memory_all(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
