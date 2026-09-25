//! Node.js bindings for mdka (napi-rs v3)

use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── Option types ────────────────────────────────────────────────────────

/// Options for a conversion. Every field is optional. An **unrecognised key is
/// an error** (see [`StrictOptions`]), so a typo -- or an option that was removed in
/// 3.0 -- fails loudly instead of being ignored.
#[napi(object)]
pub struct JsConversionOptions {
    /// "balanced" | "minimal"
    pub mode: Option<String>,
    pub preserve_ids: Option<bool>,
    pub drop_interactive_shell: Option<bool>,
}

/// The keys `JsConversionOptions` accepts, as JavaScript spells them.
const VALID_KEYS: [&str; 3] = ["mode", "preserveIds", "dropInteractiveShell"];

/// Options removed in 3.0, with the reason each never mattered. napi drops a key it
/// does not know, which after 3.0 would have left a plain-JavaScript caller passing
/// `{ preserveClasses: true }` with no error and no warning -- while TypeScript
/// rejects it, Python raises `TypeError` and the CLI exits 1. A removed option is
/// obsolete, not misspelt, and the message says so.
const REMOVED_KEYS: [(&str, &str); 6] = [
    (
        "preserveClasses",
        "Markdown has no attribute syntax, so it never changed the output",
    ),
    (
        "preserveDataAttrs",
        "Markdown has no attribute syntax, so it never changed the output",
    ),
    (
        "preserveAriaAttrs",
        "Markdown has no attribute syntax, so it never changed the output",
    ),
    (
        "preserveUnknownAttrs",
        "Markdown has no attribute syntax, so it never changed the output",
    ),
    (
        "dropPresentationAttrs",
        "Markdown has no attribute syntax, so it never changed the output",
    ),
    ("unwrapUnknownWrappers", "it could not change the output"),
];

/// `JsConversionOptions`, checked as it crosses from JavaScript.
///
/// napi ignores object keys it has no field for. This wrapper reads the key list
/// first -- on the JavaScript thread, as the argument is converted -- and notes
/// anything outside [`VALID_KEYS`], naming it. A removed option gets its own message
/// (*removed in 3.0*), as a removed mode name does, rather than *unknown*. The
/// declared TypeScript type stays `JsConversionOptions` (`ts_arg_type`), so callers
/// see no difference.
///
/// **The violation is recorded, not raised, at the boundary.** Raising it there
/// would make an `async` function *throw* synchronously for a bad key while it
/// *rejects* for a bad mode, since the mode is parsed inside the body. Recording it
/// and reporting it from [`to_rust_opts`] gives every function one behaviour: a
/// synchronous function throws, and a Promise-returning function rejects.
pub struct StrictOptions {
    opts: JsConversionOptions,
    violation: Option<String>,
}

impl TypeName for StrictOptions {
    fn type_name() -> &'static str {
        "JsConversionOptions"
    }
    fn value_type() -> ValueType {
        ValueType::Object
    }
}

impl ValidateNapiValue for StrictOptions {}

impl FromNapiValue for StrictOptions {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> Result<Self> {
        let obj = unsafe { Object::from_napi_value(env, napi_val)? };
        let names = obj.get_property_names()?;
        let mut violation = None;
        for i in 0..names.get_array_length()? {
            let key: String = names.get_element(i)?;
            if VALID_KEYS.contains(&key.as_str()) {
                continue;
            }
            violation = Some(match REMOVED_KEYS.iter().find(|(k, _)| *k == key) {
                Some((_, why)) => format!(
                    "option '{key}' was removed in 3.0; {why}. Remove it: the output is the same without it."
                ),
                None => format!(
                    "unknown option '{key}'. Valid options: {}",
                    VALID_KEYS.join(", ")
                ),
            });
            break;
        }
        Ok(StrictOptions {
            opts: unsafe { JsConversionOptions::from_napi_value(env, napi_val)? },
            violation,
        })
    }
}

fn to_rust_opts(js: Option<StrictOptions>) -> Result<mdka::ConversionOptions> {
    let js = match js {
        Some(StrictOptions {
            violation: Some(message),
            ..
        }) => return Err(Error::from_reason(message)),
        Some(StrictOptions { opts, .. }) => opts,
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

// ─── Bulk conversion outcome ─────────────────────────────────────────────

/// What happened to one file in a bulk conversion.
///
/// **Exactly one of `dest` and `error` is set**, and `ok` says which
/// (`ok === (dest !== undefined)`). TypeScript cannot express "exactly one of",
/// which is why `ok` exists: check it, and the other fields follow.
///
/// A single-file conversion does not return one of these: it resolves to the
/// destination path, or rejects. Only a bulk call reports per file, because one
/// failure must not abort the others.
#[napi(object)]
pub struct FileOutcome {
    /// The input path, as it was passed in.
    pub src: String,
    /// The output path that was written. Set when `ok` is true.
    pub dest: Option<String>,
    /// Why this file failed. Set when `ok` is false.
    pub error: Option<String>,
    /// Whether this file was converted.
    pub ok: bool,
}

// ─── String conversion API ───────────────────────────────────────────────

/// Converts an HTML string to Markdown. Throws only if `options` is wrong: an
/// unknown or removed key, or an unknown mode. HTML itself cannot fail it.
#[napi]
pub fn html_to_markdown(
    html: String,
    #[napi(ts_arg_type = "JsConversionOptions | undefined | null")] options: Option<StrictOptions>,
) -> Result<String> {
    let opts = to_rust_opts(options)?;
    Ok(mdka::html_to_markdown_with(&html, &opts))
}

/// Asynchronous form of `htmlToMarkdown`: the conversion runs on a worker
/// thread. Rejects (or throws, for a bad key) only if `options` is wrong.
#[napi]
pub async fn html_to_markdown_async(
    html: String,
    #[napi(ts_arg_type = "JsConversionOptions | undefined | null")] options: Option<StrictOptions>,
) -> Result<String> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || mdka::html_to_markdown_with(&html, &opts))
        .await
        .map_err(|e| Error::from_reason(format!("task panicked: {e}")))
}

/// Converts multiple HTML strings to Markdown, each independently. Cannot
/// fail on the HTML, so it returns plain strings, not a result type (RFC 039
/// §3 A2). Its options are optional the same way every other Node function's
/// now are: `htmlToMarkdown`, `htmlToMarkdownAsync`, `htmlFileToMarkdown` and
/// `htmlFilesToMarkdown` had a separate `…With` twin until 3.0. There is no async
/// twin -- the underlying Rust call is CPU-bound and already parallel across
/// cores internally (via rayon) when the `parallel` feature is on, so a
/// `spawn_blocking` wrapper here would add a thread hop without shortening the
/// work.
#[napi]
pub fn html_to_markdown_many(
    htmls: Vec<String>,
    #[napi(ts_arg_type = "JsConversionOptions | undefined | null")] options: Option<StrictOptions>,
) -> Result<Vec<String>> {
    let opts = to_rust_opts(options)?;
    Ok(mdka::html_to_markdown_many_with(&htmls, &opts))
}

// ─── Single-file conversion API ──────────────────────────────────────────

/// Converts one HTML file and resolves to the path that was written.
///
/// When `outDir` is null or undefined, the `.md` file is written next to the input.
/// **A failure rejects the promise** with an `Error` whose message begins
/// `IO error: `; there is no result object to inspect, because the caller asked
/// about exactly one file. (`htmlFilesToMarkdown` differs on purpose: it reports
/// per file.)
///
/// ```js
/// // writes into the same directory
/// const dest = await htmlFileToMarkdown('index.html')
///
/// // writes into a different directory
/// const dest = await htmlFileToMarkdown('index.html', 'out/')
/// ```
#[napi]
pub async fn html_file_to_markdown(
    path: String,
    out_dir: Option<String>,
    #[napi(ts_arg_type = "JsConversionOptions | undefined | null")] options: Option<StrictOptions>,
) -> Result<String> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || -> std::result::Result<String, String> {
        mdka::html_file_to_markdown_with(&path, out_dir.as_deref(), &opts)
            .map(|dest| dest.to_string_lossy().into_owned())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| Error::from_reason(format!("task panicked: {e}")))?
    .map_err(Error::from_reason)
}

// ─── Bulk file conversion API ────────────────────────────────────────────

/// Converts many HTML files into `outDir` and resolves to one `FileOutcome`
/// per input, in input order.
///
/// **A failing file does not reject the promise:** its entry has `ok: false` and
/// `error` set, and the other files are converted regardless. The promise rejects
/// only when the call as a whole cannot proceed -- `outDir` cannot be created
/// (`cannot create out_dir: …`) or `options` is wrong. That is the design, not an
/// inconsistency with `htmlFileToMarkdown`: with one file, failing the call is the
/// answer; with many, one bad file must not hide the rest.
#[napi]
pub async fn html_files_to_markdown(
    paths: Vec<String>,
    out_dir: String,
    #[napi(ts_arg_type = "JsConversionOptions | undefined | null")] options: Option<StrictOptions>,
) -> Result<Vec<FileOutcome>> {
    let opts = to_rust_opts(options)?;
    tokio::task::spawn_blocking(move || -> std::result::Result<Vec<FileOutcome>, String> {
        use std::path::Path;
        let out = Path::new(&out_dir);
        std::fs::create_dir_all(out).map_err(|e| format!("cannot create out_dir: {e}"))?;
        let path_bufs: Vec<std::path::PathBuf> =
            paths.iter().map(std::path::PathBuf::from).collect();
        let raw = mdka::html_files_to_markdown_with(&path_bufs, out, &opts);
        Ok(raw
            .into_iter()
            .map(|o| {
                let src = o.src.to_string_lossy().into_owned();
                match o.result {
                    Ok(dest) => FileOutcome {
                        src,
                        dest: Some(dest.to_string_lossy().into_owned()),
                        error: None,
                        ok: true,
                    },
                    Err(e) => FileOutcome {
                        src,
                        dest: None,
                        error: Some(e.to_string()),
                        ok: false,
                    },
                }
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
