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
//!   -m, --mode <MODE>    balanced(default)|minimal  (strict|semantic|preserve: deprecated aliases of balanced)
//!       --preserve-ids   Emit <a id="…"></a> anchors for elements with an id (on by default except in minimal)
//!       --no-preserve-ids  Turn anchor emission off, in any mode
//!       --preserve-classes  [deprecated, no effect] Keep class attributes
//!       --preserve-data  [deprecated, no effect] Keep data-* attributes
//!       --preserve-aria  [deprecated, no effect] Keep aria-* attributes
//!       --drop-shell     Drop nav/header/footer/aside
//!       --unwrap-wrappers  [deprecated, no effect today] Unwrap div/span/section/article/main tags, keeping their content and separation
//!   -h, --help           Show this help
//!   -V, --version        Show the version
//!       --               End of options; everything after is a path
//! ```
//!
//! Deprecation notices and per-file progress (`in.html -> in.md`) are written
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
                          [deprecated] strict | semantic | preserve: aliases of balanced, removed in 3.0
      --preserve-ids      Emit <a id=\"…\"></a> anchors for elements with an id.
                          On by default in every mode except minimal
      --no-preserve-ids   Turn anchor emission off, in any mode
      --preserve-classes  [deprecated, no effect] Keep class attributes (Markdown has no attribute syntax)
      --preserve-data     [deprecated, no effect] Keep data-* attributes (same reason)
      --preserve-aria     [deprecated, no effect] Keep aria-* attributes (same reason)
      --drop-shell        Drop nav/header/footer/aside
      --unwrap-wrappers   [deprecated, no effect today] Unwrap div/span/section/article/main tags, keeping their content and separation
  -h, --help              Show this help
  -V, --version           Show the version
      --                  End of options; everything after is a path

Modes:
  balanced  General use (default)
  minimal   Body text and structure only, with no shell elements or id anchors;
            for LLM preprocessing and compaction
  strict | semantic | preserve
            [deprecated, removed in 3.0] Aliases of balanced: identical
            output, kept for compatibility, and each prints a warning on
            stderr. Use balanced. Only balanced and minimal convert differently.

Output:
  Without -o, a single file is written beside its input as .md
  -o is required when converting multiple files
  Deprecation notices and per-file progress (in.html -> in.md) go to stderr,
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

/// The wording after the prefix matches Node's and Python's own
/// deprecation-warning wording (see `node/src/lib.rs`'s and
/// `python/src/lib.rs`'s `warn_deprecated_field`), so a user moving between
/// bindings does not find a different story on each. The prefix is the CLI's
/// one warning shape, `mdka: warning: `, shared with `warn_deprecated_mode`:
/// `progname: warning:` is the ordinary Unix form, and `warning: mdka:` read as
/// though mdka were the subject.
fn warn_deprecated_flag(flag: &str) {
    eprintln!(
        "mdka: warning: `{flag}` has no effect and is deprecated (see \
         https://nabbisen.github.io/mdka-rs/api/options.html). Markdown has \
         no attribute syntax, so this option was never expressible in the \
         output."
    );
}

/// `--unwrap-wrappers` (deprecated 2.9.0). Its own wording, not
/// `warn_deprecated_flag`'s: that one says Markdown has no attribute syntax,
/// which is false for this flag. Same `mdka: warning: ` shape.
fn warn_deprecated_unwrap_flag() {
    eprintln!(
        "mdka: warning: `--unwrap-wrappers` has no effect and is deprecated: unwrapping leaves \
         no Markdown-visible trace today, so this option cannot change the output. It is removed \
         from the 3.0 surface; if wrapper handling becomes expressible it returns as a new option \
         (see https://nabbisen.github.io/mdka-rs/api/options.html#unwrap_unknown_wrappers)."
    );
}

/// stderr, never stdout: stdout is the converted Markdown, and a warning there
/// would corrupt a pipe. The mode still works and its output is unchanged.
fn warn_deprecated_mode(mode: &str) {
    eprintln!(
        "mdka: warning: --mode {mode} is an alias of balanced and produces identical output; \
         it is removed in 3.0"
    );
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
    // `--preserve-ids` was write-only: on by default in 4 of 5 modes, with
    // no way to turn it off short of switching modes (which also drops the
    // shell and unwraps wrappers). `Option<bool>` lets an explicit
    // `--no-preserve-ids` override the mode's default the same way
    // `--preserve-ids` already could (RFC 039 §2.5, §3 A7).
    let mut preserve_ids_override: Option<bool> = None;
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
            "--preserve-ids" => preserve_ids_override = Some(true),
            "--no-preserve-ids" => preserve_ids_override = Some(false),
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

    // Deprecated alias modes (RFC 041 §9). Only when the caller named one:
    // `mode` starts as Balanced, so a run with no `--mode` is silent. Matched
    // on the name, not the variants -- naming a `#[deprecated]` variant would
    // itself warn, and this is the one place that must not be silenced.
    if matches!(mode.as_str(), "strict" | "semantic" | "preserve") {
        warn_deprecated_mode(mode.as_str());
    }

    // CLI flags override the mode's defaults
    let mut opts = ConversionOptions::for_mode(mode);
    if let Some(v) = preserve_ids_override {
        opts.preserve_ids = v;
    }
    // preserve_classes/preserve_data_attrs/preserve_aria_attrs are deprecated
    // no-ops (RFC 005 Slice B2); the flags are kept as no-op passthroughs for
    // command-line compatibility rather than removed. Unlike Node
    // (`process.emitWarning`) and Python (`DeprecationWarning`), the CLI
    // previously said nothing at all when one was passed -- silent on the
    // one surface most likely to sit unread in a script (RFC 039 §2.5, §3
    // A7). Warned only when explicitly passed, matching both bindings: the
    // mode's own defaults for these fields never trigger it.
    #[allow(deprecated)]
    {
        if preserve_classes {
            warn_deprecated_flag("--preserve-classes");
            opts.preserve_classes = true;
        }
        if preserve_data {
            warn_deprecated_flag("--preserve-data");
            opts.preserve_data_attrs = true;
        }
        if let Some(v) = preserve_aria_override {
            warn_deprecated_flag("--preserve-aria");
            opts.preserve_aria_attrs = v;
        }
    }
    if drop_shell {
        opts.drop_interactive_shell = true;
    }
    // Deprecated no-op (RFC 048 §7), warned only when the flag was passed.
    #[allow(deprecated)]
    if unwrap_wrappers {
        warn_deprecated_unwrap_flag();
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
                // Progress, not output (RFC 039 §3 A7): stderr, so it cannot mix
                // into a redirected stdin conversion. A file argument's
                // conversion is the file written, not stdout.
                Ok(r) => eprintln!("{} -> {}", r.src.display(), r.dest.display()),
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
                    // Progress, not output -- see the single-file case above.
                    Ok(dest) => eprintln!("{} -> {}", src.display(), dest.display()),
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
