use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use scraper::Html;
use std::hint::black_box;

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

// ── Depth scaling (RFC 012 §2.5/§4.4): nesting depth, not input size ────────
//
// `load_scaling_datasets` above varies total bytes at roughly constant,
// shallow structure -- "width". This generates the same shape
// `deep_nest.html`/the stack-overflow tests already use (`<div>` repeated,
// nested, around one leaf `<p>`), at several depths, entirely in memory: no
// new file to ship, and each depth is a single number rather than a whole
// document to maintain.
fn depth_datasets() -> Vec<(&'static str, String)> {
    [1_000usize, 5_000, 10_000, 20_000, 40_000, 80_000]
        .iter()
        .map(|&depth| {
            let open = "<div>".repeat(depth);
            let close = "</div>".repeat(depth);
            let name = match depth {
                1_000 => "1k",
                5_000 => "5k",
                10_000 => "10k",
                20_000 => "20k",
                40_000 => "40k",
                80_000 => "80k",
                _ => unreachable!(),
            };
            (name, format!("{open}<p>deep</p>{close}"))
        })
        .collect()
}

/// mdka only: the whole question is how much of mdka's own time the parser
/// takes as nesting gets deeper, not a cross-library comparison (the other
/// seven are not this RFC's claim). Reports total conversion time beside
/// `scraper::Html::parse_document` alone -- the parse the traversal then
/// walks -- so the split is a direct measurement, not an estimate.
fn emit_depth_report() {
    let datasets = depth_datasets();
    let mut records = Vec::new();

    println!("\nDepth scaling (mdka only) — median of 5 runs");
    println!("{:=<70}", "");
    println!(
        "  {:<6}  {:>12}  {:>14}  {:>8}",
        "depth", "total", "parse_document", "parse %"
    );
    println!("{:-<70}", "");

    for (name, html) in &datasets {
        let total_ns = wall_median_ns(
            || {
                let _ = black_box(mdka::html_to_markdown(html));
            },
            5,
        );
        let parse_ns = wall_median_ns(
            || {
                let _ = black_box(Html::parse_document(html));
            },
            5,
        );
        let pct = 100.0 * parse_ns as f64 / total_ns as f64;
        println!(
            "  {:<6}  {:>9.3} ms  {:>11.3} ms  {:>6.1}%",
            name,
            total_ns as f64 / 1_000_000.0,
            parse_ns as f64 / 1_000_000.0,
            pct
        );
        records.push(CsvRecord {
            benchmark_name: format!("depth/mdka/{name}/total"),
            input_size: html.len(),
            threads: 1,
            time_ns: total_ns,
            memory_bytes: 0,
        });
        records.push(CsvRecord {
            benchmark_name: format!("depth/mdka/{name}/parse_document"),
            input_size: html.len(),
            threads: 1,
            time_ns: parse_ns,
            memory_bytes: 0,
        });
    }
    println!("{:=<70}\n", "");
    append_csv("target/bench_results.csv", &records);
    eprintln!(
        "[depth] CSV → target/bench_results.csv ({} rows)",
        records.len()
    );
}

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
    emit_depth_report();
    bench_scaling_all(c);
    print_end();
}

criterion_group!(benches, setup);
criterion_main!(benches);
