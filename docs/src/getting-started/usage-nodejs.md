# Usage — Node.js

## Installation

```bash
npm install mdka
```

## Basic Conversion

```js
const { htmlToMarkdown } = require('mdka')

const html = `
  <h1>Hello</h1>
  <p>mdka converts <strong>HTML</strong> to <em>Markdown</em>.</p>
`
const md = htmlToMarkdown(html)
console.log(md)
// # Hello
//
// mdka converts **HTML** to *Markdown*.
```

## Async Conversion

`htmlToMarkdownAsync` offloads work to a Rust thread pool, keeping the
Node.js event loop free:

```js
const { htmlToMarkdownAsync } = require('mdka')

const md = await htmlToMarkdownAsync(html)

// Concurrent conversion of many pages
const results = await Promise.all(pages.map(p => htmlToMarkdownAsync(p.html)))
```

## Conversion with Options

```js
const { htmlToMarkdownWith, htmlToMarkdownWithAsync } = require('mdka')

// Strip nav/header/footer — useful for content extraction
const md = htmlToMarkdownWith(html, {
  mode: 'minimal',
  dropInteractiveShell: true,
})

// Async version
const md = await htmlToMarkdownWithAsync(html, { mode: 'semantic' })
```

Available mode strings: `"balanced"` (default), `"strict"`, `"minimal"`,
`"semantic"`, `"preserve"`.

## Single File Conversion

```js
const { htmlFileToMarkdown, htmlFileToMarkdownWith } = require('mdka')

// Output to same directory: page.html → page.md
const result = await htmlFileToMarkdown('page.html')
console.log(`${result.src} → ${result.dest}`)

// Output to specific directory
const result = await htmlFileToMarkdown('page.html', 'out/')

// With options
const result = await htmlFileToMarkdownWith('page.html', 'out/', {
  mode: 'minimal',
  dropInteractiveShell: true,
})
```

## Bulk Parallel Conversion

```js
const { htmlFilesToMarkdown, htmlFilesToMarkdownWith } = require('mdka')
const path = require('path')

const files = ['a.html', 'b.html', 'c.html']
const results = await htmlFilesToMarkdown(files, 'out/')

for (const r of results) {
  if (r.error) console.error(`${r.src}: ${r.error}`)
  else         console.log(`${r.src} → ${r.dest}`)
}

// With options
const results = await htmlFilesToMarkdownWith(files, 'out/', {
  mode: 'semantic',
  preserveAriaAttrs: true,
})
```

## TypeScript

Type definitions are bundled. No `@types/` package is needed:

```ts
import {
  htmlToMarkdown,
  htmlToMarkdownWith,
  htmlToMarkdownAsync,
  htmlFileToMarkdown,
  htmlFilesToMarkdown,
  ConversionOptions,
  ConvertResult,
} from 'mdka'

const opts: ConversionOptions = {
  mode: 'minimal',
  dropInteractiveShell: true,
}
const md: string = htmlToMarkdownWith(html, opts)
```
