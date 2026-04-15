'use strict'

const { platform, arch } = process

function getPlatformTriple() {
  const os  = platform === 'win32' ? 'win32' : platform === 'darwin' ? 'darwin' : 'linux'
  const cpu = arch === 'x64' ? 'x64' : arch === 'arm64' ? 'arm64' : arch
  const abi = platform === 'linux' ? '-gnu' : ''
  return `${os}-${cpu}${abi}`
}

let native
try {
  native = require(`./mdka_node.${getPlatformTriple()}.node`)
} catch (_) {
  try {
    native = require(`../target/release/libmdka_node.so`)
  } catch (_) {
    throw new Error(`mdka: native binary not found for ${getPlatformTriple()}. Run: npm run build`)
  }
}

module.exports = {
  /** HTML 文字列を Markdown に変換する（同期・balanced）。 */
  htmlToMarkdown:              native.htmlToMarkdown,
  /** HTML 文字列をオプション指定で Markdown に変換する（同期）。 */
  htmlToMarkdownWith:          native.htmlToMarkdownWith,
  /** HTML 文字列を Markdown に変換する（非同期・balanced）。 */
  htmlToMarkdownAsync:         native.htmlToMarkdownAsync,
  /** HTML 文字列をオプション指定で Markdown に変換する（非同期）。 */
  htmlToMarkdownWithAsync:     native.htmlToMarkdownWithAsync,
  /**
   * 単一 HTML ファイルを変換する（非同期・balanced）。
   * outDir が null/undefined の場合は入力と同じディレクトリに出力。
   */
  htmlFileToMarkdown:          native.htmlFileToMarkdown,
  /** 単一 HTML ファイルをオプション指定で変換する（非同期）。 */
  htmlFileToMarkdownWith:      native.htmlFileToMarkdownWith,
  /** 複数 HTML ファイルを rayon で並列変換する（非同期・balanced）。 */
  htmlFilesToMarkdown:         native.htmlFilesToMarkdown,
  /** 複数 HTML ファイルをオプション指定で並列変換する（非同期）。 */
  htmlFilesToMarkdownWith:     native.htmlFilesToMarkdownWith,
  /** mdka バージョン文字列。 */
  version:                     native.version,
}
