//! Node.js バインディング for mdka (napi-rs v3)

use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── オプション型 ──────────────────────────────────────────────────────────

#[napi(object)]
pub struct JsConversionOptions {
    /// "balanced" | "strict" | "minimal" | "semantic" | "preserve"
    pub mode: Option<String>,
    pub preserve_ids: Option<bool>,
    pub preserve_classes: Option<bool>,
    pub preserve_data_attrs: Option<bool>,
    pub preserve_aria_attrs: Option<bool>,
    pub drop_interactive_shell: Option<bool>,
    pub unwrap_unknown_wrappers: Option<bool>,
}

/// `process.emitWarning(message, 'DeprecationWarning')` for a field that is
/// `#[deprecated]` on the Rust side. `#[deprecated]` does not cross FFI, so
/// Node callers see nothing unless this is emitted explicitly. Only called
/// when the field was **explicitly passed** (`Some(_)`), never for a default
/// -- warning on every call regardless of intent would just get the warning
/// suppressed wholesale.
fn warn_deprecated_field(env: &Env, field: &str) -> Result<()> {
    let global = env.get_global()?;
    let process: Object = global.get_named_property("process")?;
    let emit_warning: Function<FnArgs<(String, String)>, Unknown> =
        process.get_named_property("emitWarning")?;
    let message = format!(
        "mdka: `{field}` has no effect and is deprecated (see RFC 005). \
         Markdown has no attribute syntax, so this option was never expressible \
         in the output."
    );
    emit_warning.apply(process, (message, "DeprecationWarning".to_string()).into())?;
    Ok(())
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
    if let Some(v) = js.unwrap_unknown_wrappers {
        opts.unwrap_unknown_wrappers = v;
    }

    Ok(opts)
}

// ─── 変換結果 ─────────────────────────────────────────────────────────────

/// ファイル変換の結果。
#[napi(object)]
pub struct ConvertResult {
    /// 変換した入力ファイルのパス。
    pub src: String,
    /// 書き出した出力ファイルのパス。
    pub dest: Option<String>,
    /// 変換失敗時のエラーメッセージ（バルク変換のみ）。
    pub error: Option<String>,
}

// ─── 文字列変換 API ───────────────────────────────────────────────────────

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

// ─── 単体ファイル変換 API ─────────────────────────────────────────────────

/// 単一の HTML ファイルを変換する（既定モード）。
///
/// `outDir` が null/undefined の場合は入力と同じディレクトリに `.md` を出力。
///
/// ```js
/// // 同じディレクトリに出力
/// const r = await htmlFileToMarkdown('index.html')
/// console.log(r.src, '->', r.dest)
///
/// // 別ディレクトリに出力
/// const r = await htmlFileToMarkdown('index.html', 'out/')
/// ```
#[napi]
pub async fn html_file_to_markdown(path: String, out_dir: Option<String>) -> Result<ConvertResult> {
    html_file_to_markdown_with(path, out_dir, None).await
}

/// 単一の HTML ファイルを指定オプションで変換する。
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

// ─── バルクファイル変換 API ───────────────────────────────────────────────

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

// ─── バージョン ───────────────────────────────────────────────────────────

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
