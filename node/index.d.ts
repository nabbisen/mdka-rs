/** mdka — HTML to Markdown converter (Rust, napi-rs v3) */

// ─── 変換オプション ────────────────────────────────────────────────────────

export interface ConversionOptions {
  /**
   * - `"balanced"` 既定。読みやすさと構造保持のバランス
   * - `"strict"`   属性削除最小。デバッグ・比較用
   * - `"minimal"`  本文と構造の要点のみ。LLM 前処理・圧縮
   * - `"semantic"` 意味属性・文書構造優先。SPA / アクセシビリティ
   * - `"preserve"` 元情報最大保持。アーカイブ・監査用
   */
  mode?: 'balanced' | 'strict' | 'minimal' | 'semantic' | 'preserve'
  preserveIds?: boolean
  preserveClasses?: boolean
  preserveDataAttrs?: boolean
  preserveAriaAttrs?: boolean
  dropInteractiveShell?: boolean
}

// ─── 変換結果 ─────────────────────────────────────────────────────────────

export interface ConvertResult {
  /** 変換した入力ファイルのパス */
  src: string
  /** 書き出した出力ファイルのパス */
  dest?: string
  /** 変換失敗時のエラーメッセージ（バルク変換のみ） */
  error?: string
}

// ─── 文字列変換 API ───────────────────────────────────────────────────────

export declare function htmlToMarkdown(html: string): string
export declare function htmlToMarkdownWith(html: string, options?: ConversionOptions): string
/** This function offloads the conversion task to a background thread pool using `tokio::task::spawn_blocking`,
 *  ensuring the V8 main thread remains unblocked.
 *  When processing multiple files (e.g., using Promise.all),
 *  tasks will run concurrently on the underlying thread pool. */
export declare function htmlToMarkdownAsync(html: string): Promise<string>
export declare function htmlToMarkdownWithAsync(html: string, options?: ConversionOptions): Promise<string>

// ─── 単体ファイル変換 API ─────────────────────────────────────────────────

/**
 * 単一の HTML ファイルを変換する（balanced モード）。
 *
 * `outDir` が省略された場合は入力と同じディレクトリに `.md` を出力する。
 *
 * @param path   - 入力 HTML ファイルのパス
 * @param outDir - 出力ディレクトリ（省略時は入力と同じディレクトリ）
 *
 * @example
 * // 同じディレクトリに index.md を生成
 * const r = await htmlFileToMarkdown('index.html')
 * console.log(r.src, '->', r.dest)
 *
 * // 別ディレクトリに出力
 * const r = await htmlFileToMarkdown('index.html', 'out/')
 */
export declare function htmlFileToMarkdown(
  path: string,
  outDir?: string | null,
): Promise<ConvertResult>

/**
 * 単一の HTML ファイルを指定オプションで変換する。
 *
 * @param path    - 入力 HTML ファイルのパス
 * @param outDir  - 出力ディレクトリ（省略時は入力と同じディレクトリ）
 * @param options - 変換オプション
 */
export declare function htmlFileToMarkdownWith(
  path: string,
  outDir?: string | null,
  options?: ConversionOptions,
): Promise<ConvertResult>

// ─── バルクファイル変換 API ───────────────────────────────────────────────

export declare function htmlFilesToMarkdown(paths: string[], outDir: string): Promise<ConvertResult[]>
export declare function htmlFilesToMarkdownWith(paths: string[], outDir: string, options?: ConversionOptions): Promise<ConvertResult[]>

// ─── バージョン ───────────────────────────────────────────────────────────

export declare function version(): string
