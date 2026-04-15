//! 簡易計測: 6ライブラリ × 6データセットのヒープアロケーション量を stdout に出力
//!
//! 実行: cargo run --release --example quick_mem
//!
//! CountingAllocator でアロケーションバイト数と回数を計測する。
//! 各セルの値は n=5 回の計測の中央値。

use mdka::alloc_counter::{AllocSnapshot, CountingAllocator};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// ─── 計測ヘルパー ─────────────────────────────────────────────────────────

struct MemResult {
    allocated_bytes: usize,
    alloc_count: usize,
}

/// クロージャを n 回実行し、アロケーションバイト数の中央値を返す。
fn measure<F: Fn()>(f: &F, n: usize) -> MemResult {
    let mut samples: Vec<(usize, usize)> = (0..n)
        .map(|_| {
            let before = AllocSnapshot::now();
            f();
            let after = AllocSnapshot::now();
            let d = after.delta_since(&before);
            (d.allocated_bytes, d.alloc_count)
        })
        .collect();
    samples.sort_unstable_by_key(|s| s.0);
    let (bytes, count) = samples[n / 2];
    MemResult {
        allocated_bytes: bytes,
        alloc_count: count,
    }
}

fn fmt_bytes(b: usize) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else if b >= 1_024 {
        format!("{:.1} KB", b as f64 / 1_024.0)
    } else {
        format!("{} B", b)
    }
}

fn fmt_count(n: usize) -> String {
    if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

// ─── main ─────────────────────────────────────────────────────────────────

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let datasets = [
        ("small", format!("{manifest}/benches/benchdata/small.html")),
        (
            "medium",
            format!("{manifest}/benches/benchdata/medium.html"),
        ),
        ("large", format!("{manifest}/benches/benchdata/large.html")),
        ("flat", format!("{manifest}/benches/benchdata/flat.html")),
        (
            "deep_nest",
            format!("{manifest}/benches/benchdata/deep_nest.html"),
        ),
        (
            "malformed",
            format!("{manifest}/benches/benchdata/malformed.html"),
        ),
    ];

    // deep_nest でスタックオーバーフロー / 著しく遅いライブラリを除外
    // let skip_crash = []; // fast_html2md: stack overflow
    let skip_slow = ["deep_nest"]; // dom_smoothie: 極端に遅い

    // ── ヘッダ ────────────────────────────────────────────────────────────
    println!();
    println!("Heap allocation — median of 7 runs  (alloc bytes / alloc count)");
    println!("{}", "=".repeat(110));
    println!(
        "  {:<11}  {:>7} │ {:>18} {:>18} {:>18} {:>18}",
        "dataset", "size", "mdka", "html2md", "fast_html2md", "htmd"
    );
    println!(
        "  {:<11}  {:>7} │ {:>18} {:>18} {:>18} {:>18}",
        "", "", "html-to-markdown-rs", "html2text", "dom_smoothie", ""
    );
    println!("{}", "-".repeat(110));

    for (name, path) in &datasets {
        let html = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("  skip {name}: file not found");
                continue;
            }
        };
        let size = html.len();

        let m_mdka = measure(
            &|| {
                let _ = mdka::html_to_markdown(&html);
            },
            5,
        );

        let fmt_cell = |m: &MemResult| -> String {
            format!(
                "{} / {}",
                fmt_bytes(m.allocated_bytes),
                fmt_count(m.alloc_count)
            )
        };
        // let crash_cell = || "   CRASH         ".to_string();

        // let m_h2md = if skip_crash.contains(name) {
        //     None
        // } else {
        //     Some(measure(
        //         &|| {
        //             let _ = html2md::parse_html(&html);
        //         },
        //         5,
        //     ))
        // };
        let m_h2md = measure(
            &|| {
                let _ = html2md::parse_html(&html);
            },
            5,
        );

        // let m_fast = if skip_crash.contains(name) {
        //     None
        // } else {
        //     Some(measure(
        //         &|| {
        //             let _ = fast_html2md::rewrite_html(&html, false);
        //         },
        //         5,
        //     ))
        // };
        let m_fast = measure(
            &|| {
                let _ = fast_html2md::rewrite_html(&html, false);
            },
            5,
        );

        let m_htmd = measure(
            &|| {
                let _ = htmd::HtmlToMarkdown::new().convert(&html);
            },
            5,
        );

        let m_htm2 = measure(
            &|| {
                let _ = html_to_markdown_rs::convert(&html, None);
            },
            5,
        );

        // let m_h2t = if skip_slow.contains(name) {
        //     None
        // } else {
        //     Some(measure(
        //         &|| {
        //             let _ = html2text::from_read(html.as_bytes(), 80);
        //         },
        //         5,
        //     ))
        // };
        let m_h2t = measure(
            &|| {
                let _ = html2text::from_read(html.as_bytes(), 80);
            },
            5,
        );

        let m_ds = if skip_slow.contains(name) {
            None
        } else {
            Some(measure(
                &|| {
                    let _ = dom_smoothie::Readability::new(html.clone(), None, None)
                        .ok()
                        .and_then(|mut r| r.parse().ok())
                        .map(|a| a.text_content.to_string())
                        .unwrap_or_default();
                },
                5,
            ))
        };

        // 1行目: mdka / html2md / fast_html2md / htm2mdrs
        println!(
            "  {:<11}  {:>6}KB │ {:>18} {:>18} {:>18} {:>18}",
            name,
            size / 1024,
            fmt_cell(&m_mdka),
            fmt_cell(&m_h2md),
            fmt_cell(&m_fast),
            fmt_cell(&m_htmd),
        );
        // 2行目: html2text / dom_smoothie（インデント揃え）
        println!(
            "  {:<11}  {:>7} │ {:>18} {:>18} {:>18}",
            "",
            "",
            fmt_cell(&m_htm2),
            fmt_cell(&m_h2t),
            m_ds.as_ref()
                .map(fmt_cell)
                .unwrap_or_else(|| "  (skip)          ".to_string()),
        );
        println!();
    }

    println!("{}", "=".repeat(110));
    println!("  alloc bytes: ヒープに確保した総バイト数（解放分を含む）");
    println!("  alloc count: malloc 呼び出し回数");
    println!("  CRASH: deep_nest でスタックオーバーフローが発生するため計測を省略");
}
