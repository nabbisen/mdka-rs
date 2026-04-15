//! Benchmark: scaling behaviour vs input size
//!
//! 10 KB → 50 KB → 100 KB → 500 KB → 1 MB → 5 MB
//! Verifies that each library scales linearly with input size.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

fn bench_scaling<F: Fn(&str)>(c: &mut Criterion, name: &str, f: F) {
    let datasets = load_scaling_datasets();
    let mut group = c.benchmark_group(format!("scaling/{}", name).as_str());
    for (name, html) in &datasets {
        if html.len() > 2_000_000 {
            group.sample_size(10);
        } else {
            group.sample_size(20);
        }
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), html, |b, html| {
            b.iter(|| f(black_box(html)))
        });
    }
    group.finish();
}

fn emit_scaling_report() {
    let datasets = load_scaling_datasets();
    let mut records = Vec::new();

    for (name, html) in &datasets {
        let size = html.len();
        let ns_mdka = wall_median_ns(
            &|| {
                let _ = mdka::html_to_markdown(html);
            },
            5,
        );
        records.push(CsvRecord {
            benchmark_name: format!("scaling/mdka/{}", name),
            input_size: size,
            threads: 1,
            time_ns: ns_mdka,
            memory_bytes: 0,
        });

        if name != &"5m".to_string() {
            let ns_html2md = wall_median_ns(
                &|| {
                    let _ = run_html2md(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/html2md/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_html2md,
                memory_bytes: 0,
            });

            let ns_fast = wall_median_ns(
                &|| {
                    let _ = run_fast_html2md(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/fast_html2md/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_fast,
                memory_bytes: 0,
            });

            let ns_fast = wall_median_ns(
                &|| {
                    let _ = run_htmd(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/htmd/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_fast,
                memory_bytes: 0,
            });

            let ns_htm2 = wall_median_ns(
                &|| {
                    let _ = run_html_to_markdown_rs(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/html-to-markdown-rs/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_htm2,
                memory_bytes: 0,
            });

            let ns_h2t = wall_median_ns(
                &|| {
                    let _ = run_html2text(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/html2text/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_h2t,
                memory_bytes: 0,
            });

            let ns_ds = wall_median_ns(
                &|| {
                    let _ = run_dom_smoothie(html);
                },
                5,
            );
            records.push(CsvRecord {
                benchmark_name: format!("scaling/dom_smoothie/{}", name),
                input_size: size,
                threads: 1,
                time_ns: ns_ds,
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
    bench_scaling(c, "mdka", |html| {
        run_mdka(html);
    });
    bench_scaling(c, "html2md", |html| {
        run_html2md(html);
    });
    bench_scaling(c, "fast_html2md", |html| {
        run_fast_html2md(html);
    });
    bench_scaling(c, "html-to-markdown-rs", |html| {
        run_html_to_markdown_rs(html);
    });
    bench_scaling(c, "html2text", |html| {
        run_html2text(html);
    });
    bench_scaling(c, "dom_smoothie", |html| {
        run_dom_smoothie(html);
    });
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
