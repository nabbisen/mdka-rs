const SAMPLE_SIZE: usize = 15;

fn wall_us<F: Fn()>(f: F) -> u64 {
    let mut times: Vec<u64> = (0..SAMPLE_SIZE)
        .map(|_| {
            let t = std::time::Instant::now();
            f();
            t.elapsed().as_micros() as u64
        })
        .collect();
    times.sort();
    times[SAMPLE_SIZE / 2]
}

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let datasets = [
        ("small", "benches/benchdata/small.html"),
        ("medium", "benches/benchdata/medium.html"),
        ("large", "benches/benchdata/large.html"),
        ("flat", "benches/benchdata/flat.html"),
        ("malformed", "benches/benchdata/malformed.html"),
    ];
    // let skip_crash = ["deep_nest"];

    println!(
        "{:<11} {:>8} {:>12} {:>12} {:>13} {:>12} {:>20} {:>12} {:>12}",
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

    for (label, rel) in &datasets {
        let path = format!("{}/{}", manifest, rel);
        let html = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("skip {}", label);
                continue;
            }
        };
        let size_kb = html.len() / 1024;

        let t_mdka = wall_us(|| {
            let _ = mdka::html_to_markdown(&html);
        });
        let t_html2md = wall_us(|| {
            let _ = html2md::parse_html(&html);
        });
        // let t_fast = if skip_crash.contains(label) {
        //     0
        // } else {
        //     wall_us(|| {
        //         let _ = fast_html2md::rewrite_html(&html, false);
        //     })
        // };
         let t_fast = wall_us(|| {
                let _ = fast_html2md::rewrite_html(&html, false);
            });
        let t_htmd = wall_us(|| {
            let _ = htmd::HtmlToMarkdown::new().convert(&html);
        });
        let t_htm2 = wall_us(|| {
            let _ = html_to_markdown_rs::convert(&html, None);
        });
        let t_h2t = wall_us(|| {
            let _ = html2text::from_read(html.as_bytes(), 80);
        });
        let t_ds = wall_us(|| {
            let _ = dom_smoothie::Readability::new(html.as_str(), None, None)
                .ok()
                .and_then(|mut r| r.parse().ok())
                .map(|a| a.text_content.to_string())
                .unwrap_or_default();
        });

        // let fast_str = if skip_crash.contains(label) {
        //     "CRASH".to_string()
        // } else {
        //     format!("{} µs", t_fast)
        // };
        // let fast_str = format!("{} µs", t_fast);
        println!(
            "{:<10} {:>7}KB {:>9} µs {:>9} µs {:>13} µs {:>9} µs {:>17} µs {:>9} µs {:>9} µs",
            label, size_kb, t_mdka, t_html2md, t_fast, t_htmd, t_htm2, t_h2t, t_ds
        );
    }

    println!("{}", border);
}
