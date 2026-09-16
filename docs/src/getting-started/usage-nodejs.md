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

const html = '<nav>menu</nav><h1>Title</h1><p>Body</p>'

// Strip nav/header/footer — useful for content extraction
const md = htmlToMarkdownWith(html, {
  mode: 'minimal',
  dropInteractiveShell: true,
})

// Async version
async function main() {
  const mdAsync = await htmlToMarkdownWithAsync(html, { mode: 'semantic' })
  console.log(mdAsync)
}
main()
```

Available mode strings: `"balanced"` (default), `"strict"`, `"minimal"`,
`"semantic"`, `"preserve"`.

## Single File Conversion

```js
const { htmlFileToMarkdown, htmlFileToMarkdownWith } = require('mdka')

async function main() {
  // Output to same directory: page.html → page.md
  const sameDir = await htmlFileToMarkdown('page.html')
  console.log(`${sameDir.src} → ${sameDir.dest}`)

  // Output to specific directory
  const outDir = await htmlFileToMarkdown('page.html', 'out/')

  // With options
  const withOpts = await htmlFileToMarkdownWith('page.html', 'out/', {
    mode: 'minimal',
    dropInteractiveShell: true,
  })
}
main()
```

## Bulk Parallel Conversion

```js
const { htmlFilesToMarkdown, htmlFilesToMarkdownWith } = require('mdka')

const files = ['a.html', 'b.html', 'c.html']

async function main() {
  const results = await htmlFilesToMarkdown(files, 'out/')

  for (const r of results) {
    if (r.error) console.error(`${r.src}: ${r.error}`)
    else         console.log(`${r.src} → ${r.dest}`)
  }

  // With options
  const withOpts = await htmlFilesToMarkdownWith(files, 'out/', {
    mode: 'semantic',
  })
}
main()
```

Two inputs whose output names collide — `a/index.html` and `b/index.html` both
becoming `out/index.md` — are not both converted. The first in the array wins
and each later one comes back with `error` set, rather than silently
overwriting.

**Three** of the deprecated attribute options are accepted here and have **no
effect**: `preserveClasses`, `preserveDataAttrs` and `preserveAriaAttrs`.
Markdown has no attribute syntax to carry them into.

The other two — `preserveUnknownAttrs` and `dropPresentationAttrs` — exist on
the Rust `ConversionOptions` but are **not fields of `JsConversionOptions`**,
which has seven. In TypeScript, passing either is a compile error:

```
TS2353: Object literal may only specify known properties, and
'preserveUnknownAttrs' does not exist in type 'JsConversionOptions'.
```

Use `mode`.

## TypeScript

Type definitions are bundled. No `@types/` package is needed:

```ts
import {
  htmlToMarkdown,
  htmlToMarkdownWith,
  htmlToMarkdownAsync,
  htmlFileToMarkdown,
  htmlFilesToMarkdown,
  JsConversionOptions,
  ConvertResult,
} from 'mdka'

const html: string = '<h1>Title</h1>'

const opts: JsConversionOptions = {
  mode: 'minimal',
  dropInteractiveShell: true,
}
const md: string = htmlToMarkdownWith(html, opts)
```

The options type is exported as **`JsConversionOptions`**, not
`ConversionOptions` — the name comes from the napi-rs binding rather than from
the Rust `ConversionOptions` it mirrors.
