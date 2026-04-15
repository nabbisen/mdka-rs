use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

// ── ベンチマーク実行 (単一パス) ──────────────────────────────────────────

fn bench_convert_all(c: &mut Criterion) {
    let datasets = load_datasets();

    for target in TARGETS {
        let mut group = c.benchmark_group(target.name);
        for (name, html) in &datasets {
            if is_skipped(target.name, name) {
                continue;
            }
            group.throughput(Throughput::Bytes(html.len() as u64));
            group.bench_with_input(BenchmarkId::new("convert", name), html, |b, html| {
                b.iter(|| (target.run_fn)(black_box(html)))
            });
        }
        group.finish();
    }
}

// ── 比較ベンチマーク (side-by-side) ─────────────────────────────────────

fn bench_compare(c: &mut Criterion) {
    let datasets = load_datasets();
    let targets_data = ["small", "medium", "large", "flat", "malformed"];

    for data_name in &targets_data {
        let html = match datasets.iter().find(|(n, _)| n == data_name) {
            Some((_, h)) => h,
            None => continue,
        };
        let mut group = c.benchmark_group(format!("compare/{}", data_name));
        group.throughput(Throughput::Bytes(html.len() as u64));

        for target in TARGETS {
            if is_skipped(target.name, data_name) {
                continue;
            }
            group.bench_function(target.name, |b| b.iter(|| (target.run_fn)(black_box(html))));
        }
        group.finish();
    }
}

// ── CSV 出力 ─────────────────────────────────────────────────────────────

fn emit_convert_csv() {
    let datasets = load_datasets();
    let mut records = Vec::new();

    for (name, html) in &datasets {
        for target in TARGETS {
            if is_skipped(target.name, name) {
                continue;
            }
            let ns = wall_median_ns(
                || {
                    let _ = (target.run_fn)(html);
                },
                7,
            );
            records.push(CsvRecord {
                benchmark_name: format!("convert/{}/{}", target.name, name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns,
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
    bench_convert_all(c);
    bench_compare(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
