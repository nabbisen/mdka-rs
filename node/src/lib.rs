//! Node.js バインディング for mdka (napi-rs v3)

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
}

fn to_rust_opts(js: Option<JsConversionOptions>) -> mdka::ConversionOptions {
    let js = match js {
        Some(j) => j,
        None => return mdka::ConversionOptions::default(),
    };
    let mode = js
        .mode
        .as_deref()
        .and_then(mdka::ConversionMode::from_str)
        .unwrap_or_default();
    let mut opts = mdka::ConversionOptions::for_mode(mode);
    if let Some(v) = js.preserve_ids {
        opts.preserve_ids = v;
    }
    if let Some(v) = js.preserve_classes {
        opts.preserve_classes = v;
    }
    if let Some(v) = js.preserve_data_attrs {
        opts.preserve_data_attrs = v;
    }
    if let Some(v) = js.preserve_aria_attrs {
        opts.preserve_aria_attrs = v;
    }
    if let Some(v) = js.drop_interactive_shell {
        opts.drop_interactive_shell = v;
    }
    opts
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
pub fn html_to_markdown_with(html: String, options: Option<JsConversionOptions>) -> String {
    mdka::html_to_markdown_with(&html, &to_rust_opts(options))
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
    let opts = to_rust_opts(options);
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
    let opts = to_rust_opts(options);
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
    let opts = to_rust_opts(options);
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
