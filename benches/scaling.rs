use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

fn bench_scaling_all(c: &mut Criterion) {
    let datasets = load_scaling_datasets();

    for target in TARGETS {
        let mut group = c.benchmark_group(format!("scaling/{}", target.name));
        for (name, html) in &datasets {
            if is_skipped(target.name, name) {
                continue;
            }
            group.sample_size(if html.len() > 2_000_000 { 10 } else { 20 });
            group.throughput(Throughput::Bytes(html.len() as u64));
            group.bench_with_input(BenchmarkId::from_parameter(name), html, |b, html| {
                b.iter(|| (target.run_fn)(black_box(html)))
            });
        }
        group.finish();
    }
}

fn emit_scaling_report() {
    let datasets = load_scaling_datasets();
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
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/{}/{}", target.name, name),
                input_size: html.len(),
                threads: 1,
                time_ns: ns,
                memory_bytes: 0,
            });
        }
    }
    append_csv("target/bench_results.csv", &records);
    eprintln!(
        "[scaling] CSV → target/bench_results.csv ({} rows)",
        records.len()
    );
}

fn setup(c: &mut Criterion) {
    print_env_info();
    emit_scaling_report();
    bench_scaling_all(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
