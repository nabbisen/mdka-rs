//! Node.js bindings for mdka (napi-rs v3)

use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── Option types ────────────────────────────────────────────────────────

#[napi(object)]
pub struct JsConversionOptions {
    /// "balanced" | "minimal"
    pub mode: Option<String>,
    pub preserve_ids: Option<bool>,
    pub drop_interactive_shell: Option<bool>,
}

fn to_rust_opts(js: Option<JsConversionOptions>) -> Result<mdka::ConversionOptions> {
    let js = match js {
        Some(j) => j,
        None => return Ok(mdka::ConversionOptions::default()),
    };

    // `FromStr` says *removed* for a mode name that existed until 2.9.0 and
    // *unknown* for anything else; the message is passed through unchanged.
    let mode = match js.mode.as_deref() {
        Some(x) => mdka::ConversionMode::from_str(x).map_err(Error::from_reason)?,
        None => mdka::ConversionMode::default(),
    };

    let mut opts = mdka::ConversionOptions::for_mode(mode);

    if let Some(v) = js.preserve_ids {
        opts.preserve_ids = v;
    }
    if let Some(v) = js.drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }

    Ok(opts)
}

// ─── Conversion result ───────────────────────────────────────────────────

/// Result of a file conversion.
#[napi(object)]
pub struct ConvertResult {
    /// Path of the input file that was converted.
    pub src: String,
    /// Path of the output file that was written.
    pub dest: Option<String>,
    /// Error message when the conversion failed (bulk conversion only).
    pub error: Option<String>,
}

// ─── String conversion API ───────────────────────────────────────────────

#[napi]
pub fn html_to_markdown(html: String) -> String {
    mdka::html_to_markdown(&html)
}

#[napi]
pub fn html_to_markdown_with(html: String, options: Option<JsConversionOptions>) -> Result<String> {
    match to_rust_opts(options) {
        Ok(x) => Ok(mdka::html_to_markdown_with(&html, &x)),
        Err(err) => Err(err),
    }
}

#[napi]
pub async fn html_to_markdown_async(html: String) -> Result<String> {
    tokio::task::spawn_blocking(move || mdka::html_to_markdown(&html))
        .await
        .map_err(|e| Error::from_reason(format!("task panicked: {e}")))
}

#[napi]
pub async fn html_to_markdown_with_async(
    html: String,
    options: Option<JsConversionOptions>,
) -> Result<String> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || mdka::html_to_markdown_with(&html, &opts))
        .await
        .map_err(|e| Error::from_reason(format!("task panicked: {e}")))
}

/// Converts multiple HTML strings to Markdown, each independently. Cannot
/// fail, so it returns plain strings, not a result type (RFC 039 §3 A2). One
/// function, not four: `options` is optional the same way
/// `htmlToMarkdownWith`'s is, rather than a separate with/without pair, and
/// there is no async twin -- the underlying Rust call is CPU-bound and
/// already parallel across cores internally (via rayon) when the `parallel`
/// feature is on, so a `spawn_blocking` wrapper here would add a thread hop
/// without shortening the work.
#[napi]
pub fn html_to_markdown_many(
    htmls: Vec<String>,
    options: Option<JsConversionOptions>,
) -> Result<Vec<String>> {
    let opts = to_rust_opts(options)?;
    Ok(mdka::html_to_markdown_many_with(&htmls, &opts))
}

// ─── Single-file conversion API ──────────────────────────────────────────

/// Converts a single HTML file (default mode).
///
/// When `outDir` is null or undefined, the `.md` file is written next to the input.
///
/// ```js
/// // writes into the same directory
/// const r = await htmlFileToMarkdown('index.html')
/// console.log(r.src, '->', r.dest)
///
/// // writes into a different directory
/// const r = await htmlFileToMarkdown('index.html', 'out/')
/// ```
#[napi]
pub async fn html_file_to_markdown(path: String, out_dir: Option<String>) -> Result<ConvertResult> {
    html_file_to_markdown_with(path, out_dir, None).await
}

/// Converts a single HTML file with the given options.
#[napi]
pub async fn html_file_to_markdown_with(
    path: String,
    out_dir: Option<String>,
    options: Option<JsConversionOptions>,
) -> Result<ConvertResult> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || -> std::result::Result<ConvertResult, String> {
        let out_dir_ref = out_dir.as_deref();
        mdka::html_file_to_markdown_with(&path, out_dir_ref, &opts)
            .map(|r| ConvertResult {
                src: r.src.to_string_lossy().into_owned(),
                dest: Some(r.dest.to_string_lossy().into_owned()),
                error: None,
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| Error::from_reason(format!("task panicked: {e}")))?
    .map_err(Error::from_reason)
}

// ─── Bulk file conversion API ────────────────────────────────────────────

#[napi]
pub async fn html_files_to_markdown(
    paths: Vec<String>,
    out_dir: String,
) -> Result<Vec<ConvertResult>> {
    html_files_to_markdown_with(paths, out_dir, None).await
}

#[napi]
pub async fn html_files_to_markdown_with(
    paths: Vec<String>,
    out_dir: String,
    options: Option<JsConversionOptions>,
) -> Result<Vec<ConvertResult>> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || -> std::result::Result<Vec<ConvertResult>, String> {
        use std::path::Path;
        let out = Path::new(&out_dir);
        std::fs::create_dir_all(out).map_err(|e| format!("cannot create out_dir: {e}"))?;
        let path_bufs: Vec<std::path::PathBuf> =
            paths.iter().map(std::path::PathBuf::from).collect();
        let raw = mdka::html_files_to_markdown_with(&path_bufs, out, &opts);
        Ok(raw
            .into_iter()
            .map(|(p, res)| match res {
                Ok(dest) => ConvertResult {
                    src: p.to_string_lossy().into_owned(),
                    dest: Some(dest.to_string_lossy().into_owned()),
                    error: None,
                },
                Err(e) => ConvertResult {
                    src: p.to_string_lossy().into_owned(),
                    dest: None,
                    error: Some(e.to_string()),
                },
            })
            .collect())
    })
    .await
    .map_err(|e| Error::from_reason(format!("task panicked: {e}")))?
    .map_err(Error::from_reason)
}

// ─── Version ─────────────────────────────────────────────────────────────

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
