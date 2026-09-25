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
//!   -m, --mode <MODE>    balanced(default)|minimal
//!       --preserve-ids   Emit <a id="…"></a> anchors for elements with an id (on by default except in minimal)
//!       --no-preserve-ids  Turn anchor emission off, in any mode
//!       --drop-shell     Drop nav/header/footer/aside
//!   -h, --help           Show this help
//!   -V, --version        Show the version
//!       --               End of options; everything after is a path
//! ```
//!
//! Per-file progress (`in.html -> in.md`) are written
//! to stderr, not stdout (RFC 039 §3 A7), so converting stdin can be redirected
//! cleanly: `echo '<h1>Hi</h1>' | mdka > out.md`. A file argument writes a
//! sibling `.md` and leaves stdout empty; `mdka page.html > out.md` creates an
//! empty `out.md` (2.4.1 -- this comment claimed the opposite).

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
  -m, --mode <MODE>       Conversion mode: balanced(default) | minimal
      --preserve-ids      Emit <a id=\"…\"></a> anchors for elements with an id.
                          On by default in every mode except minimal
      --no-preserve-ids   Turn anchor emission off, in any mode
      --drop-shell        Drop nav/header/footer/aside
  -h, --help              Show this help
  -V, --version           Show the version
      --                  End of options; everything after is a path

Modes:
  balanced  General use (default)
  minimal   Body text and structure only, with no shell elements or id anchors;
            for LLM preprocessing and compaction

Output:
  Without -o, a single file is written beside its input as .md
  -o is required when converting multiple files
  Per-file progress (in.html -> in.md) goes to stderr,
  never stdout, so `... | mdka > out.md` (stdin) holds only the conversion.
  A file argument writes the .md beside it and prints nothing to stdout:
  `mdka page.html > out.md` gives an empty out.md -- use -o to choose where

Examples:
  echo '<h1>Hello</h1>' | mdka
  mdka index.html                         # → index.md (same directory)
  mdka -o out/ index.html                 # → out/index.md
  mdka --mode minimal --drop-shell -o out/ *.html  # drop nav/header/footer
  mdka --no-preserve-ids -o out/ index.html # anchors off, any mode
";

/// A flag that was removed in 3.0. Not an "unknown option": the user was told
/// about all of these for several releases, and a bare "unknown option" plus the
/// whole usage text would not say what to do. It says so, and stops -- exit 1,
/// on stderr, so a script that still passes the flag fails loudly rather than
/// quietly converting as though it had been understood.
fn removed_flag(flag: &str) -> ! {
    let reason = if flag == "--unwrap-wrappers" {
        "it could not change the output"
    } else {
        "Markdown has no attribute syntax, so it never changed the output"
    };
    eprintln!(
        "error: `{flag}` was removed in 3.0; {reason}. Remove it from the command line: \
         the output is the same without it."
    );
    process::exit(1);
}

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
    // `--preserve-ids` was write-only: on by default in every mode but one, with
    // no way to turn it off short of switching modes (which also drops the
    // shell and unwraps wrappers). `Option<bool>` lets an explicit
    // `--no-preserve-ids` override the mode's default the same way
    // `--preserve-ids` already could (RFC 039 §2.5, §3 A7).
    let mut preserve_ids_override: Option<bool> = None;
    let mut drop_shell = false;
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
                    // `FromStr` says *removed* for a name that existed until
                    // 2.9.0 and *unknown* for anything else; the list of valid
                    // modes is only useful for the second.
                    let valid = if err.starts_with("unknown conversion mode") {
                        ". Valid: balanced|minimal"
                    } else {
                        ""
                    };
                    eprintln!("error: {err}{valid}");
                    process::exit(1);
                });
            }
            "--preserve-ids" => preserve_ids_override = Some(true),
            "--no-preserve-ids" => preserve_ids_override = Some(false),
            "--preserve-classes" | "--preserve-data" | "--preserve-aria" | "--unwrap-wrappers" => {
                removed_flag(&arg)
            }
            "--drop-shell" => drop_shell = true,
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
    if let Some(v) = preserve_ids_override {
        opts.preserve_ids = v;
    }
    if drop_shell {
        opts.drop_interactive_shell = true;
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
                // Progress, not output (RFC 039 §3 A7): stderr, so it cannot mix
                // into a redirected stdin conversion. A file argument's
                // conversion is the file written, not stdout.
                Ok(dest) => eprintln!("{} -> {}", file_args[0], dest.display()),
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
            for outcome in results {
                let src = outcome.src.display();
                match outcome.result {
                    // Progress, not output -- see the single-file case above.
                    Ok(dest) => eprintln!("{src} -> {}", dest.display()),
                    Err(e) => {
                        eprintln!("error: {src}: {e}");
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
