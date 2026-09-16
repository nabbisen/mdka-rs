//! mdka CLI — HTML to Markdown converter
//!
//! Exposes every conversion feature of the `mdka` library on the command line.
//!
//! # Usage
//!
//! ```text
//! mdka [OPTIONS] [FILE...]
//!
//! Options:
//!   -o, --output <DIR>   Output directory (defaults to the input's directory)
//!   -m, --mode <MODE>    balanced(default)|strict|minimal|semantic|preserve
//!       --preserve-ids   Keep id attributes
//!       --preserve-classes  [deprecated, no effect] Keep class attributes
//!       --preserve-data  [deprecated, no effect] Keep data-* attributes
//!       --preserve-aria  [deprecated, no effect] Keep aria-* attributes
//!       --drop-shell     Drop nav/header/footer/aside
//!       --unwrap-wrappers  Unwrap div/span/section/article/main that carry no meaning
//!   -h, --help           Show this help
//!   -V, --version        Show the version
//!       --               End of options; everything after is a path
//! ```

use std::io::{self, Read};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use mdka::options::{ConversionMode, ConversionOptions};

const USAGE: &str = "\
Usage:
  mdka [OPTIONS] [FILE...]

Options:
  -o, --output <DIR>      Output directory (defaults to the input's directory)
  -m, --mode <MODE>       Conversion mode: balanced(default) | strict | minimal | semantic | preserve
      --preserve-ids      Keep id attributes
      --preserve-classes  [deprecated, no effect] Keep class attributes (Markdown has no attribute syntax)
      --preserve-data     [deprecated, no effect] Keep data-* attributes (same reason)
      --preserve-aria     [deprecated, no effect] Keep aria-* attributes (same reason)
      --drop-shell        Drop nav/header/footer/aside
      --unwrap-wrappers   Unwrap div/span/section/article/main that carry no meaning
  -h, --help              Show this help
  -V, --version           Show the version
      --                  End of options; everything after is a path

Modes:
  balanced  Balances readability with structural fidelity (general purpose, default)
  strict    Removes as few attributes as possible; for debugging and comparison
  minimal   Body text and structure only; for LLM preprocessing and compaction
  semantic  Favours semantic attributes and document structure; for SPAs and accessibility
  preserve  Retains as much of the original as possible; for archiving and auditing

Output:
  Without -o, a single file is written beside its input as .md
  -o is required when converting multiple files

Examples:
  echo '<h1>Hello</h1>' | mdka
  mdka index.html                         # → index.md (same directory)
  mdka -o out/ index.html                 # → out/index.md
  mdka --mode minimal --drop-shell -o out/ *.html  # drop nav/header/footer
  mdka --mode preserve -o archive/ *.html # retain as much as possible
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return;
    }

    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("mdka {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // ── Argument parsing ──────────────────────────────────────────────
    let mut out_dir: Option<PathBuf> = None;
    let mut mode = ConversionMode::Balanced;
    let mut preserve_ids = false;
    let mut preserve_classes = false;
    let mut preserve_data = false;
    let mut preserve_aria_override: Option<bool> = None;
    let mut drop_shell = false;
    let mut unwrap_wrappers = false;
    let mut file_args: Vec<String> = Vec::new();

    let mut iter = args.into_iter().peekable();
    let mut only_files = false;
    while let Some(arg) = iter.next() {
        if only_files {
            file_args.push(arg);
            continue;
        }
        match arg.as_str() {
            // End of options: everything after is a path, however it starts.
            // This is what keeps a file named `-x.html` reachable now that
            // unknown `-` arguments are rejected.
            "--" => only_files = true,
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
            "--unwrap-wrappers" => unwrap_wrappers = true,
            // An unrecognised `-`-prefixed argument used to be taken as a file
            // path, so `mdka --version` reported "No such file or directory"
            // and a typo like `--drop-shel` silently converted nothing
            // (audit A-17). `-` alone is left alone: it is a conventional
            // stdin placeholder, not a flag.
            _ if arg.starts_with('-') && arg != "-" => {
                eprintln!("error: unknown option '{arg}'\n\n{USAGE}");
                process::exit(1);
            }
            _ => file_args.push(arg),
        }
    }

    // CLI flags override the mode's defaults
    let mut opts = ConversionOptions::for_mode(mode);
    if preserve_ids {
        opts.preserve_ids = true;
    }
    // preserve_classes/preserve_data_attrs/preserve_aria_attrs are deprecated
    // no-ops (RFC 005 Slice B2); the flags are kept as no-op passthroughs for
    // command-line compatibility rather than removed.
    #[allow(deprecated)]
    {
        if preserve_classes {
            opts.preserve_classes = true;
        }
        if preserve_data {
            opts.preserve_data_attrs = true;
        }
        if let Some(v) = preserve_aria_override {
            opts.preserve_aria_attrs = v;
        }
    }
    if drop_shell {
        opts.drop_interactive_shell = true;
    }
    if unwrap_wrappers {
        opts.unwrap_unknown_wrappers = true;
    }

    // ── Dispatch ──────────────────────────────────────────────────────
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
        // single file → out_dir, or the input's own directory
        (false, 1, _) => {
            match mdka::html_file_to_markdown_with(&file_args[0], out_dir.as_deref(), &opts) {
                Ok(r) => println!("{} -> {}", r.src.display(), r.dest.display()),
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            }
        }
        // multiple files → out_dir is required
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
