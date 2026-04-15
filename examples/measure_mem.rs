//! メモリ使用量を CountingAllocator で計測して表示する
//!
//! 実行: cargo run --example measure_mem

#[global_allocator]
static ALLOCATOR: mdka::alloc_counter::CountingAllocator = mdka::alloc_counter::CountingAllocator;

use mdka::alloc_counter::AllocSnapshot;

fn measure_allocs<F: Fn(&str)>(html: &str, f: F) -> usize {
    let mut allocs: Vec<usize> = (0..3)
        .map(|_| {
            let b = AllocSnapshot::now();
            f(html);
            AllocSnapshot::now().delta_since(&b).allocated_bytes
        })
        .collect();

    allocs.sort_unstable();
    allocs[1] // 中央値を返す
}

fn fmt(b: usize) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else {
        format!("{:.1} KB", b as f64 / 1_024.0)
    }
}

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let files = [
        ("small", "benches/benchdata/small.html"),
        ("medium", "benches/benchdata/medium.html"),
        ("large", "benches/benchdata/large.html"),
        ("flat", "benches/benchdata/flat.html"),
        ("deep_nest", "benches/benchdata/deep_nest.html"),
        ("malformed", "benches/benchdata/malformed.html"),
    ];

    println!(
        "| {:12} | {:>7} | {:>12} | {:>16} | {:>8} |",
        "dataset", "size", "mdka alloc", "fast_html2md alloc", "ratio"
    );
    let border = format!(
        "|{:-<14}|{:-<9}|{:-<14}|{:-<18}|{:-<10}|",
        "", "", "", "", ""
    );
    println!("{}", border);

    for (label, rel) in &files {
        let path = format!("{}/{}", manifest, rel);
        let html = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("skip {label}: file not found");
                continue;
            }
        };
        let size = html.len();
        let mdka_alloc = measure_allocs(&html, |html| {
            let _ = mdka::html_to_markdown(html);
        });
        let fast_alloc = measure_allocs(&html, |html| {
            let _ = fast_html2md::rewrite_html(html, false);
        });
        let ratio = if fast_alloc > 0 {
            mdka_alloc as f64 / fast_alloc as f64
        } else {
            0.0
        };
        println!(
            "| {:12} | {:>6}KB | {:>12} | {:>16} | {:>7.2}x |",
            label,
            size / 1024,
            fmt(mdka_alloc),
            fmt(fast_alloc),
            ratio
        );
    }

    println!("{}", border);
}
