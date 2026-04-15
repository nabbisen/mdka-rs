//! 簡易計測: 5ライブラリ × 6データセットの変換速度を stdout に出力
//! 実行: cargo run --release --example quick_compare

const SAMPLE_SIZE: usize = 7;

fn wall_us<F: Fn()>(f: F) -> u64 {
    let mut times: Vec<u64> = (0..SAMPLE_SIZE)
        .map(|_| {
            let t = std::time::Instant::now();
            f();
            t.elapsed().as_micros() as u64
        })
        .collect();
    times.sort_unstable();
    times[SAMPLE_SIZE / 2]
}

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

    // let skip_fast = ["deep_nest"];

    println!(
        "{:<12} {:>7} | {:>10} {:>12} {:>14} {:>10} {:>20} {:>11} {:>14}",
        "dataset",
        "size",
        "mdka",
        "html2md",
        "fast_html2md",
        "htmd",
        "html-to-markdown-rs",
        "html2text",
        "dom_smoothie"
    );
    let border = format!("{}", "-".repeat(122));
    println!("{}", border);

    for (name, path) in &datasets {
        let html = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("skip {name}: not found");
                continue;
            }
        };
        let size = html.len();

        let mdka_us = wall_us(|| {
            let _ = mdka::html_to_markdown(&html);
        });

        let html2md_us = wall_us(|| {
            let _ = html2md::parse_html(&html);
        });

        // let fast_str = if skip_fast.contains(name) {
        //     "CRASH".to_string()
        // } else {
        //     format!(
        //         "{}µs",
        //         wall_us(|| {
        //             let _ = fast_html2md::rewrite_html(&html, false);
        //         })
        //     )
        // };
        let fast_str = wall_us(|| {
            let _ = fast_html2md::rewrite_html(&html, false);
        });

        let htmd_us = wall_us(|| {
            let _ = htmd::HtmlToMarkdown::new().convert(&html);
        });

        let htm2_us = wall_us(|| {
            let _ = html_to_markdown_rs::convert(&html, None);
        });

        let h2t_us = wall_us(|| {
            let _ = html2text::from_read(html.as_bytes(), 80);
        });

        let ds_us = wall_us(|| {
            let _ = dom_smoothie::Readability::new(html.clone(), None, None)
                .ok()
                .and_then(|mut r| r.parse().ok())
                .map(|a| a.text_content.to_string())
                .unwrap_or_default();
        });

        let fmt_us = |us: u64| -> String {
            if us >= 1000 {
                format!("{:.1}ms", us as f64 / 1000.0)
            } else {
                format!("{}µs", us)
            }
        };

        println!(
            "{:<11} {:>6}KB | {:>10} {:>12} {:>14} {:>10} {:>20} {:>11} {:>14}",
            name,
            size / 1024,
            fmt_us(mdka_us),
            fmt_us(html2md_us),
            fmt_us(fast_str),
            fmt_us(htmd_us),
            fmt_us(htm2_us),
            fmt_us(h2t_us),
            fmt_us(ds_us)
        );
    }

    println!("{}", border);
}
