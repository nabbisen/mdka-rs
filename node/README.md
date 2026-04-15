# mdka — Node.js バインディング

Rust 製 HTML→Markdown コンバータ **mdka** の Node.js バインディングです。  
[napi-rs v3](https://napi.rs/) を使い、同期・非同期・ファイル並列変換の3つの API を提供します。

## インストール

```bash
npm install mdka
```

> プリビルドバイナリが同梱されているため、`cargo` や `Rust` は不要です。

## 使い方

### CommonJS

```js
const { htmlToMarkdown, htmlToMarkdownAsync, htmlFilesToMarkdown, version } = require('mdka')
```

### ESM / TypeScript

```ts
import { htmlToMarkdown, htmlToMarkdownAsync, htmlFilesToMarkdown, version } from 'mdka'
```

---

### `htmlToMarkdown(html: string): string` — 同期変換

```js
const md = htmlToMarkdown('<h1>Hello</h1><p><strong>world</strong></p>')
// => "# Hello\n\nworld\n"
```

### `htmlToMarkdownAsync(html: string): Promise<string>` — 非同期変換

Rust スレッドプールへオフロードするため、Node.js イベントループをブロックしません。

```js
const md = await htmlToMarkdownAsync('<h1>Hello</h1>')
// => "# Hello\n"

// 複数の変換を同時並行
const results = await Promise.all([
  htmlToMarkdownAsync(html1),
  htmlToMarkdownAsync(html2),
])
```

### `htmlFilesToMarkdown(paths: string[], outDir: string): Promise<ConvertResult[]>` — ファイル一括変換

rayon で並列変換し、各ファイルを `outDir` に `.md` として保存します。

```js
const results = await htmlFilesToMarkdown(
  ['docs/a.html', 'docs/b.html', 'docs/c.html'],
  './out'
)

for (const r of results) {
  if (r.error) {
    console.error(`${r.src}: ${r.error}`)
  } else {
    console.log(`${r.src} → ${r.dest}`)
  }
}
```

### `version(): string` — バージョン確認

```js
console.log(version()) // => "0.1.0"
```

---

## 対応 HTML 要素

| HTML | Markdown |
|---|---|
| `h1`–`h6` | `#`–`######` |
| `p`, `div` 等ブロック | 段落 |
| `ul` / `ol` (start= 対応) | `- ` / `1. ` (ネスト対応) |
| `blockquote` (ネスト対応) | `> ` |
| `pre/code[class=language-*]` | ```` ```lang ```` |
| `strong`, `b` / `em`, `i` | `**` / `*` |
| `a[href][title]` | `[text](url "title")` |
| `img[src][alt][title]` | `![alt](url "title")` |
| `hr` / `br` | `---` / `  \n` |
| `script`, `style` | 無視 |

## 動作要件

- Node.js >= 16
- Linux x86_64 / macOS arm64 / macOS x64 / Windows x64  
  (他プラットフォームはソースからビルド: `npm run build`)

## ソースからビルド

```bash
git clone https://github.com/example/mdka
cd mdka/node
npm run build
```

Rust 1.82 以上と Cargo が必要です。
