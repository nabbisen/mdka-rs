//! mdka CLI — HTML → Markdown コンバータ
//!
//! `mdka` ライブラリのすべての変換機能をコマンドラインから呼び出せる。
//!
//! # 使い方
//!
//! ```text
//! mdka [OPTIONS] [FILE...]
//!
//! Options:
//!   -o, --output <DIR>   出力ディレクトリ（省略時は入力と同じディレクトリ）
//!   -m, --mode <MODE>    balanced(既定)|strict|minimal|semantic|preserve
//!       --preserve-ids   id 属性を保持する
//!       --preserve-classes  class 属性を保持する
//!       --preserve-data  data-* 属性を保持する
//!       --preserve-aria  aria-* 属性を保持する
//!       --drop-shell     nav/header/footer/aside を除外する
//!   -h, --help           このヘルプを表示
//! ```

// alloc_counter は CLI バイナリのみで登録する。
// ライブラリ利用者のアロケータには一切干渉しない。
#[global_allocator]
static ALLOCATOR: mdka::alloc_counter::CountingAllocator = mdka::alloc_counter::CountingAllocator;

use std::io::{self, Read};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use mdka::options::{ConversionMode, ConversionOptions};

const USAGE: &str = "\
Usage:
  mdka [OPTIONS] [FILE...]

Options:
  -o, --output <DIR>      出力ディレクトリ（省略時は入力と同じディレクトリ）
  -m, --mode <MODE>       変換モード: balanced(既定) | strict | minimal | semantic | preserve
      --preserve-ids      id 属性を保持する
      --preserve-classes  class 属性を保持する
      --preserve-data     data-* 属性を保持する
      --preserve-aria     aria-* 属性を保持する
      --drop-shell        nav/header/footer/aside を除外する
  -h, --help              このヘルプを表示

モード説明:
  balanced  読みやすさと構造保持のバランス（汎用・既定）
  strict    属性削除を最小限に。デバッグ・比較用途
  minimal   本文と構造の要点のみ。LLM 前処理・圧縮
  semantic  意味属性・文書構造を優先。SPA / アクセシビリティ重視
  preserve  元情報を最大限保持。アーカイブ・監査用途

出力先:
  -o 未指定の場合、単一ファイルは入力と同じディレクトリに .md として出力
  複数ファイルは -o が必須

Examples:
  echo '<h1>Hello</h1>' | mdka
  mdka index.html                         # → index.md (同じディレクトリ)
  mdka -o out/ index.html                 # → out/index.md
  mdka --mode minimal --drop-shell *.html # nav/header/footer を除外
  mdka --mode preserve -o archive/ *.html # 最大限の情報を保持
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return;
    }

    // ── 引数解析 ──────────────────────────────────────────────────────
    let mut out_dir: Option<PathBuf> = None;
    let mut mode = ConversionMode::Balanced;
    let mut preserve_ids = false;
    let mut preserve_classes = false;
    let mut preserve_data = false;
    let mut preserve_aria_override: Option<bool> = None;
    let mut drop_shell = false;
    let mut file_args: Vec<String> = Vec::new();

    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                out_dir = Some(PathBuf::from(iter.next().unwrap_or_else(|| {
                    eprintln!("error: -o/--output requires a directory");
                    process::exit(1);
                })));
            }
            "-m" | "--mode" => {
                let m = iter.next().unwrap_or_default();
                mode = ConversionMode::from_str(&m).unwrap_or_else(|err| {
                    eprintln!(
                        "error: {err}. \
                               Valid: balanced|strict|minimal|semantic|preserve"
                    );
                    process::exit(1);
                });
            }
            "--preserve-ids" => preserve_ids = true,
            "--preserve-classes" => preserve_classes = true,
            "--preserve-data" => preserve_data = true,
            "--preserve-aria" => preserve_aria_override = Some(true),
            "--drop-shell" => drop_shell = true,
            _ => file_args.push(arg),
        }
    }

    // モードの既定設定に CLI フラグを上書き
    let mut opts = ConversionOptions::for_mode(mode);
    if preserve_ids {
        opts.preserve_ids = true;
    }
    if preserve_classes {
        opts.preserve_classes = true;
    }
    if preserve_data {
        opts.preserve_data_attrs = true;
    }
    if let Some(v) = preserve_aria_override {
        opts.preserve_aria_attrs = v;
    }
    if drop_shell {
        opts.drop_interactive_shell = true;
    }

    // ── 実行分岐 ──────────────────────────────────────────────────────
    match (file_args.is_empty(), file_args.len(), &out_dir) {
        // stdin → stdout
        (true, _, _) => {
            let mut html = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut html) {
                eprintln!("error: failed to read stdin: {e}");
                process::exit(1);
            }
            print!("{}", mdka::html_to_markdown_with(&html, &opts));
        }
        // 単一ファイル → out_dir または入力と同じディレクトリ
        (false, 1, _) => {
            match mdka::html_file_to_markdown_with(&file_args[0], out_dir.as_deref(), &opts) {
                Ok(r) => println!("{} -> {}", r.src.display(), r.dest.display()),
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            }
        }
        // 複数ファイル → out_dir 必須
        (false, _, None) => {
            eprintln!("error: -o/--output required when converting multiple files.\n\n{USAGE}");
            process::exit(1);
        }
        (false, _, Some(dir)) => {
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("error: cannot create '{}': {e}", dir.display());
                process::exit(1);
            }
            let paths: Vec<PathBuf> = file_args.iter().map(PathBuf::from).collect();
            let results = mdka::html_files_to_markdown_with(&paths, dir, &opts);
            let mut had_error = false;
            for (src, res) in results {
                match res {
                    Ok(dest) => println!("{} -> {}", src.display(), dest.display()),
                    Err(e) => {
                        eprintln!("error: {}: {e}", src.display());
                        had_error = true;
                    }
                }
            }
            if had_error {
                process::exit(1);
            }
        }
    }
}
