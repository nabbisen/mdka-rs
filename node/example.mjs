/**
 * mdka Node.js binding — Usage (ESM)
 * How to run:
 *     node example.mjs
 */
import { createRequire } from 'module'
const require = createRequire(import.meta.url)
const { htmlToMarkdown, htmlToMarkdownAsync, htmlFilesToMarkdown, version } = require('./index')

import { writeFileSync, mkdtempSync, rmSync } from 'fs'
import { join } from 'path'
import { tmpdir } from 'os'

console.log(`mdka v${version()}\n`)

// ── 同期変換 ────────────────────────────────────────────────────────────────
const html = `
<h1>mdka デモ</h1>
<p>Rust 製の高速 HTML → Markdown コンバータです。</p>
<h2>特徴</h2>
<ul>
  <li><strong>省メモリ</strong>: v1 比で劇的に向上</li>
  <li><strong>高速</strong>: 非再帰 DFS + シングルパス正規化</li>
  <li><strong>堅牢</strong>: 10,000 段ネストでもクラッシュしない</li>
</ul>
<blockquote>
  <p>大サイズファイルにも対応可能 (10KB〜5MB)</p>
</blockquote>
<pre><code class="language-js">const md = htmlToMarkdown('&lt;h1&gt;Hello&lt;/h1&gt;')</code></pre>
`

console.log('=== 同期変換 ===')
console.log(htmlToMarkdown(html))

// ── 非同期変換 ──────────────────────────────────────────────────────────────
console.log('=== 非同期並行変換 (Promise.all) ===')
const items = [
  '<h1>タイトル</h1>',
  '<p>段落テキスト <em>強調</em> と <strong>太字</strong></p>',
  '<ol start="5"><li>five</li><li>six</li></ol>',
  '<blockquote><blockquote><p>ネスト引用</p></blockquote></blockquote>',
]
const results = await Promise.all(items.map(h => htmlToMarkdownAsync(h)))
results.forEach((md, i) => console.log(`[${i}] ${md.trim()}`))

// ── ファイル変換 ────────────────────────────────────────────────────────────
console.log('\n=== ファイル並列変換 ===')
const tmp = mkdtempSync(join(tmpdir(), 'mdka-demo-'))
const outDir = join(tmp, 'out')

const files = ['index', 'about', 'blog'].map(name => {
  const p = join(tmp, `${name}.html`)
  writeFileSync(p, `<h1>${name}</h1><p>${name} のコンテンツ</p>`)
  return p
})

const fileResults = await htmlFilesToMarkdown(files, outDir)
for (const r of fileResults) {
  const status = r.error ? `❌ ${r.error}` : `✅ ${r.dest}`
  console.log(`  ${r.src.split('/').pop()} → ${status}`)
}

rmSync(tmp, { recursive: true })
console.log('\n完了')
