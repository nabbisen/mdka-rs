//! Node.js bindings for mdka (napi-rs v3)

use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── Option types ────────────────────────────────────────────────────────

#[napi(object)]
pub struct JsConversionOptions {
    /// "balanced" | "minimal"; "strict", "semantic" and "preserve" are deprecated aliases of "balanced", removed in 3.0
    pub mode: Option<String>,
    pub preserve_ids: Option<bool>,
    pub preserve_classes: Option<bool>,
    pub preserve_data_attrs: Option<bool>,
    pub preserve_aria_attrs: Option<bool>,
    pub drop_interactive_shell: Option<bool>,
    pub unwrap_unknown_wrappers: Option<bool>,
}

/// `process.emitWarning(message, 'DeprecationWarning')`. `#[deprecated]` does
/// not cross FFI, so Node callers see nothing unless this is emitted
/// explicitly.
fn emit_deprecation_warning(env: &Env, message: String) -> Result<()> {
    let global = env.get_global()?;
    let process: Object = global.get_named_property("process")?;
    let emit_warning: Function<FnArgs<(String, String)>, Unknown> =
        process.get_named_property("emitWarning")?;
    emit_warning.apply(process, (message, "DeprecationWarning".to_string()).into())?;
    Ok(())
}

/// Warning for a field that is `#[deprecated]` on the Rust side. Only called
/// when the field was **explicitly passed** (`Some(_)`), never for a default
/// -- warning on every call regardless of intent would just get the warning
/// suppressed wholesale.
fn warn_deprecated_field(env: &Env, field: &str) -> Result<()> {
    emit_deprecation_warning(
        env,
        format!(
            "mdka: `{field}` has no effect and is deprecated (see https://nabbisen.github.io/mdka-rs/api/options.html). \
             Markdown has no attribute syntax, so this option was never expressible \
             in the output."
        ),
    )
}

/// Warning for `unwrapUnknownWrappers` (deprecated 2.9.0). Its own message, not
/// `warn_deprecated_field`'s: that one says Markdown has no attribute syntax,
/// which is false for this option. Only when the caller passed it.
fn warn_deprecated_unwrap_wrappers(env: &Env) -> Result<()> {
    emit_deprecation_warning(
        env,
        "mdka: `unwrapUnknownWrappers` has no effect and is deprecated: unwrapping leaves no \
         Markdown-visible trace today, so this option cannot change the output. It is removed \
         from the 3.0 surface; if wrapper handling becomes expressible it returns as a new \
         option (see https://nabbisen.github.io/mdka-rs/api/options.html#unwrap_unknown_wrappers)."
            .to_string(),
    )
}

/// Warning for `mode: 'strict' | 'semantic' | 'preserve'` (RFC 041 §9), which
/// are aliases of `'balanced'`. Same rule as the fields: only when the caller
/// named the mode, so a call with no `mode` is silent.
fn warn_deprecated_mode(env: &Env, mode: &str) -> Result<()> {
    emit_deprecation_warning(
        env,
        format!(
            "mdka: mode '{mode}' is an alias of 'balanced' and produces identical output; \
             it is deprecated and removed in 3.0. Use 'balanced' \
             (see https://nabbisen.github.io/mdka-rs/api/modes.html)."
        ),
    )
}

/// `env: None` for the `_async` entry points: napi-rs requires an async
/// `#[napi]` function's whole future to be `Send`, and `Env` is not `Send`
/// (confirmed by trying it — `error: future cannot be sent between threads
/// safely`, `Env` captured as a parameter). So deprecation warnings can only
/// be emitted from the synchronous entry point, `html_to_markdown_with`,
/// which is also the only one that receives an `Env`. The async paths still
/// apply the (silently no-op) deprecated fields' values -- correctness is
/// unaffected, only the warning is unavailable there.
fn to_rust_opts(
    env: Option<&Env>,
    js: Option<JsConversionOptions>,
) -> Result<mdka::ConversionOptions> {
    let js = match js {
        Some(j) => j,
        None => return Ok(mdka::ConversionOptions::default()),
    };

    let mode = match js.mode.as_deref() {
        Some(x) => match mdka::ConversionMode::from_str(x) {
            Ok(x) => x,
            Err(err) => return Err(Error::from_reason(err.to_string())),
        },
        None => mdka::ConversionMode::default(),
    };

    // Matched on the name, not the variants: naming a `#[deprecated]` variant
    // would itself warn, and this is the one place that must not be silenced.
    if let Some(env) = env
        && matches!(mode.as_str(), "strict" | "semantic" | "preserve")
    {
        warn_deprecated_mode(env, mode.as_str())?;
    }

    let mut opts = mdka::ConversionOptions::for_mode(mode);

    if let Some(v) = js.preserve_ids {
        opts.preserve_ids = v;
    }
    // preserve_classes/preserve_data_attrs/preserve_aria_attrs are deprecated
    // no-ops (RFC 005 Slice B2); the JS-facing fields are kept as no-op
    // passthroughs for API compatibility rather than removed.
    #[allow(deprecated)]
    {
        if let Some(v) = js.preserve_classes {
            if let Some(env) = env {
                warn_deprecated_field(env, "preserveClasses")?;
            }
            opts.preserve_classes = v;
        }
        if let Some(v) = js.preserve_data_attrs {
            if let Some(env) = env {
                warn_deprecated_field(env, "preserveDataAttrs")?;
            }
            opts.preserve_data_attrs = v;
        }
        if let Some(v) = js.preserve_aria_attrs {
            if let Some(env) = env {
                warn_deprecated_field(env, "preserveAriaAttrs")?;
            }
            opts.preserve_aria_attrs = v;
        }
    }
    if let Some(v) = js.drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }
    // Deprecated no-op (RFC 048 §7); kept as a passthrough until 3.0 removes it.
    #[allow(deprecated)]
    if let Some(v) = js.unwrap_unknown_wrappers {
        if let Some(env) = env {
            warn_deprecated_unwrap_wrappers(env)?;
        }
        opts.unwrap_unknown_wrappers = v;
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
pub fn html_to_markdown_with(
    html: String,
    options: Option<JsConversionOptions>,
    env: Env,
) -> Result<String> {
    match to_rust_opts(Some(&env), options) {
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
    let opts = to_rust_opts(None, options)?;
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
    env: Env,
) -> Result<Vec<String>> {
    let opts = to_rust_opts(Some(&env), options)?;
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
    let opts = to_rust_opts(None, options)?;
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
    let opts = to_rust_opts(None, options)?;
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
