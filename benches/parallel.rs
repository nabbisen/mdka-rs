//! ベンチマーク: 並列変換性能
//!
//! ## 概要
//! rayon のスレッド数を 1/2/max と変えながら medium.html × 100 回の
//! スループットを計測する。
//!
//! ## 注意: 2コア環境での挙動
//! 本環境は 2 論理コアのため、threads=2 では rayon のタスク分配・
//! スレッド同期コストが変換コストを上回り、逆に遅くなる場合がある。
//! 4コア以上の環境では線形スケーリングが期待できる。
//!
//! ## html_files_to_markdown での並列効果
//! CPU バウンドの純変換より I/O バウンドのファイル処理の方が
//! 並列化の恩恵が大きい（ファイル読み書き待ちにCPUを使えるため）。

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rayon::prelude::*;
use std::{hint::black_box, sync::Arc};

#[path = "bench_common.rs"]
mod bench_common;
use bench_common::*;

const REPEAT: usize = 100;

fn make_pool(threads: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("thread pool")
}

fn parallel_run(html: &str, pool: &rayon::ThreadPool) -> Vec<String> {
    let inputs: Vec<&str> = std::iter::repeat(html).take(REPEAT).collect();
    pool.install(|| {
        inputs
            .par_iter()
            .map(|h| mdka::html_to_markdown(black_box(h)))
            .collect()
    })
}

fn bench_parallel(c: &mut Criterion) {
    let datasets = load_datasets();
    let html = Arc::new(
        datasets
            .iter()
            .find(|(n, _)| *n == "medium")
            .map(|(_, h)| h.clone())
            .expect("medium dataset"),
    );

    let max = rayon::current_num_threads().max(1);
    let counts: Vec<usize> = [1usize, 2, 4, max]
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // ── シーケンシャルベースライン ──
    {
        let html = Arc::clone(&html);
        let mut group = c.benchmark_group("parallel/sequential");
        group.sample_size(10);
        group.throughput(Throughput::Bytes((html.len() * REPEAT) as u64));
        group.bench_function("baseline", move |b| {
            let html = Arc::clone(&html);
            b.iter(|| {
                (0..REPEAT)
                    .map(|_| mdka::html_to_markdown(black_box(&*html)))
                    .collect::<Vec<_>>()
            })
        });
        group.finish();
    }

    // ── rayon スレッド数ごと ──
    let mut group = c.benchmark_group("parallel/rayon");
    group.sample_size(10);
    group.throughput(Throughput::Bytes((html.len() * REPEAT) as u64));

    for &threads in &counts {
        let pool = Arc::new(make_pool(threads));
        let html_ref = Arc::clone(&html);
        let pool_ref = Arc::clone(&pool);

        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &threads,
            move |b, _| {
                let html = Arc::clone(&html_ref);
                let pool = Arc::clone(&pool_ref);
                b.iter(|| parallel_run(&html, &pool))
            },
        );
    }
    group.finish();
}

/// I/O バウンドのファイル並列変換（ディスクあり）
fn bench_files_to_disk(c: &mut Criterion) {
    use std::fs;
    use std::path::PathBuf;

    let tmp = std::env::temp_dir().join("mdka_parallel_bench");
    fs::create_dir_all(&tmp).unwrap();
    let out = tmp.join("out");
    fs::create_dir_all(&out).unwrap();

    // 20ファイル分の入力を準備（medium × 20）
    let manifest = std::env!("CARGO_MANIFEST_DIR");
    let src = fs::read_to_string(format!("{}/benches/benchdata/medium.html", manifest)).unwrap();
    let paths: Vec<PathBuf> = (0..20)
        .map(|i| {
            let p = tmp.join(format!("input_{:02}.html", i));
            fs::write(&p, &src).unwrap();
            p
        })
        .collect();

    let max = rayon::current_num_threads().max(1);
    let counts: Vec<usize> = [1usize, 2, max]
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    for &threads in &counts {
        let pool = make_pool(threads);
        let mut group = c.benchmark_group(format!("files_to_disk/threads/{}", threads));
        group.sample_size(10);
        group.throughput(Throughput::Bytes((src.len() * 20) as u64));
        group.bench_function("convert", |b| {
            b.iter(|| {
                // ファイルI/O込みの並列変換
                pool.install(|| {
                    paths.par_iter().for_each(|p| {
                        let html = std::fs::read_to_string(p).unwrap();
                        let md = mdka::html_to_markdown(black_box(&html));
                        let out_p = out.join(p.file_stem().unwrap()).with_extension("md");
                        std::fs::write(out_p, md).unwrap();
                    });
                });
            })
        });
        group.finish();
    }

    let _ = fs::remove_dir_all(&tmp);
}

/// スピードアップ比を stderr + CSV に出力
fn emit_speedup_report() {
    let datasets = load_datasets();
    let html = datasets
        .iter()
        .find(|(n, _)| *n == "medium")
        .map(|(_, h)| h.as_str())
        .expect("medium dataset");

    let max = rayon::current_num_threads().max(1);
    let counts: Vec<usize> = [1usize, 2, 4, max]
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    eprintln!(
        "\n=== Parallel Speedup (medium × {} iterations, {} cores) ===",
        REPEAT, max
    );
    eprintln!(
        "{:<10} {:>12} {:>14} {:>10}",
        "threads", "time_ms", "throughput", "speedup"
    );
    eprintln!("{}", "─".repeat(50));

    let mut records = Vec::new();
    let mut baseline_ns: Option<u64> = None;

    for &threads in &counts {
        let pool = make_pool(threads);
        // ウォームアップ
        parallel_run(html, &pool);
        let ns = wall_median_ns(
            || {
                parallel_run(html, &pool);
            },
            5,
        );
        let ms = ns as f64 / 1e6;
        let mbps = (html.len() * REPEAT) as f64 / 1_048_576.0 / (ns as f64 / 1e9);
        let base = *baseline_ns.get_or_insert(ns);
        let speedup = base as f64 / ns as f64;
        eprintln!(
            "{:<10} {:>12.1} {:>12.1} MB/s {:>8.2}x",
            threads, ms, mbps, speedup
        );

        records.push(CsvRecord {
            benchmark_name: format!("parallel/rayon/threads/{}", threads),
            input_size: html.len() * REPEAT,
            threads,
            time_ns: ns,
            memory_bytes: 0,
        });
    }
    eprintln!();
    append_csv("target/bench_results.csv", &records);
    eprintln!(
        "[parallel] CSV → target/bench_results.csv ({} rows)",
        records.len()
    );
}

fn setup(c: &mut Criterion) {
    emit_speedup_report();
    bench_parallel(c);
    bench_files_to_disk(c);
}

criterion_group!(benches, setup);
criterion_main!(benches);
