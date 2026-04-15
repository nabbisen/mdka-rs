//! ベンチマーク共通ユーティリティ

use std::path::Path;

// ─── データセットロード ──────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct Dataset {
    pub name: &'static str,
    pub html: String,
}

#[allow(dead_code)]
pub fn load_datasets() -> Vec<(&'static str, String)> {
    let files: &[(&str, &str)] = &[
        ("small", "benches/benchdata/small.html"),
        ("medium", "benches/benchdata/medium.html"),
        ("large", "benches/benchdata/large.html"),
        ("deep_nest", "benches/benchdata/deep_nest.html"),
        ("flat", "benches/benchdata/flat.html"),
        ("malformed", "benches/benchdata/malformed.html"),
    ];
    let manifest = std::env!("CARGO_MANIFEST_DIR");
    files
        .iter()
        .map(|(name, rel)| {
            let path = Path::new(manifest).join(rel);
            let html = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Cannot read {}", path.display()));
            (*name, html)
        })
        .collect()
}

#[allow(dead_code)]
pub fn load_scaling_datasets() -> Vec<(&'static str, String)> {
    let files: &[(&str, &str)] = &[
        ("10k", "benches/benchdata/small.html"),
        ("50k", "benches/benchdata/scale_50k.html"),
        ("100k", "benches/benchdata/medium.html"),
        ("500k", "benches/benchdata/scale_500k.html"),
        ("1m", "benches/benchdata/large.html"),
        ("5m", "benches/benchdata/scale_5m.html"),
    ];
    let manifest = std::env!("CARGO_MANIFEST_DIR");
    files
        .iter()
        .map(|(name, rel)| {
            let path = Path::new(manifest).join(rel);
            let html = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Cannot read {}", path.display()));
            (*name, html)
        })
        .collect()
}

// ─── 環境情報 ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn print_env_info() {
    eprintln!("\n╔══════════════════════════════════╗");
    eprintln!("║   mdka Benchmark Environment     ║");
    eprintln!("╠══════════════════════════════════╣");
    eprintln!("║ logical CPUs : {:>17} ║", rayon::current_num_threads());
    if let Ok(v) = std::process::Command::new("rustc")
        .arg("--version")
        .output()
    {
        let s = String::from_utf8_lossy(&v.stdout);
        eprintln!("║ {:34} ║", s.trim());
    }
    if let Ok(v) = std::process::Command::new("uname").args(["-sr"]).output() {
        let s = String::from_utf8_lossy(&v.stdout);
        eprintln!("║ OS : {:30} ║", s.trim());
    }
    eprintln!("╚══════════════════════════════════╝\n");
}

#[allow(dead_code)]
pub fn print_end() {
    eprintln!("════════════════════════════════════\n");
}

// ── helpers ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[inline]
pub fn run_mdka(html: &str) -> String {
    mdka::html_to_markdown(html)
}

#[allow(dead_code)]
#[inline]
pub fn run_html2md(html: &str) -> String {
    html2md::parse_html(html)
}

#[allow(dead_code)]
#[inline]
pub fn run_fast_html2md(html: &str) -> String {
    let commonmark = false;
    fast_html2md::rewrite_html(html, commonmark)
}

#[allow(dead_code)]
#[inline]
pub fn run_htmd(html: &str) -> String {
    htmd::HtmlToMarkdown::new()
        .convert(html)
        .unwrap_or_default()
}

#[allow(dead_code)]
#[inline]
pub fn run_html_to_markdown_rs(html: &str) -> String {
    html_to_markdown_rs::convert(html, None)
        .unwrap_or_default()
        .content
        .unwrap_or_default()
}

#[allow(dead_code)]
#[inline]
pub fn run_html2text(html: &str) -> String {
    html2text::from_read(html.as_bytes(), 80).unwrap_or_default()
}

#[allow(dead_code)]
#[inline]
pub fn run_dom_smoothie(html: &str) -> String {
    match dom_smoothie::Readability::new(html, None, None) {
        Ok(mut r) => r
            .parse()
            .map(|a| a.text_content.to_string())
            .unwrap_or_default(),
        Err(_) => String::new(),
    }
}

// ─── CSV 出力 ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct CsvRecord {
    pub benchmark_name: String,
    pub input_size: usize,
    pub threads: usize,
    pub time_ns: u64,
    pub memory_bytes: usize,
}

#[allow(dead_code)]
pub fn append_csv(path: &str, records: &[CsvRecord]) {
    use std::io::Write;
    std::fs::create_dir_all(
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new(".")),
    )
    .ok();
    let exists = std::path::Path::new(path).exists();
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Cannot open CSV");
    if !exists {
        writeln!(f, "benchmark_name,input_size,threads,time_ns,memory_bytes").unwrap();
    }
    for r in records {
        writeln!(
            f,
            "{},{},{},{},{}",
            r.benchmark_name, r.input_size, r.threads, r.time_ns, r.memory_bytes
        )
        .unwrap();
    }
}

/// criterion の `Criterion` から最後の測定値を取り出せないので、
/// 独自の wall-clock 計測で中央値 (ns) を返す。
#[allow(dead_code)]
pub fn wall_median_ns<F: FnMut()>(mut f: F, reps: u32) -> u64 {
    let mut times: Vec<u64> = (0..reps)
        .map(|_| {
            let t = std::time::Instant::now();
            f();
            t.elapsed().as_nanos() as u64
        })
        .collect();
    times.sort_unstable();
    times[times.len() / 2]
}
