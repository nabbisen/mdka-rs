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

const html = '<h1>Hello</h1>'
const pages = [
  { html: '<h1>A</h1>' },
  { html: '<p>B</p>' },
]

async function main() {
  const md = await htmlToMarkdownAsync(html)

  // Concurrent conversion of many pages
  const results = await Promise.all(pages.map(p => htmlToMarkdownAsync(p.html)))
  console.log(md, results)
}
main()
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
  const mdAsync = await htmlToMarkdownWithAsync(html, { mode: 'balanced' })
  console.log(mdAsync)
}
main()
```

Available mode strings: `"balanced"` (default) and `"minimal"`. `"strict"`,
`"semantic"` and `"preserve"` were aliases of `"balanced"` and were removed in
3.0: passing one **throws** (or, from an `Async` function, **rejects**) with
`conversion mode 'strict' was removed in 3.0; it was an alias of 'balanced'. Use
'balanced'.` — see [Conversion Modes](../api/modes.md).

## Converting Multiple Strings

```js
const { htmlToMarkdownMany } = require('mdka')

const pages = ['<h1>A</h1>', '<p>B</p>', '<ul><li>C</li></ul>']
const results = htmlToMarkdownMany(pages)
// ['# A\n', 'B\n', '- C\n']

// With options — the same shape as htmlToMarkdownWith's
const withOpts = htmlToMarkdownMany(pages, { mode: 'minimal' })
```

Each input is converted independently and the results keep the input order.

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
    mode: 'minimal',
  })
}
main()
```

Two inputs whose output names collide — `a/index.html` and `b/index.html` both
becoming `out/index.md` — are not both converted. The first in the array wins
and each later one comes back with `error` set, rather than silently
overwriting.

**Four options were removed in 3.0:** `preserveClasses`, `preserveDataAttrs`,
`preserveAriaAttrs` and `unwrapUnknownWrappers`. None of them ever changed the
output. In TypeScript, passing one is a compile error:

```
TS2353: Object literal may only specify known properties, and
'preserveClasses' does not exist in type 'JsConversionOptions'.
```

**In plain JavaScript it is not an error and not a warning: it is ignored.** Nothing
tells you an option you still pass is gone, so search your code for these four
names. The output is the same with or without them.

`JsConversionOptions` now has three fields: `mode`, `preserveIds` and
`dropInteractiveShell`.

## Package Version

```js
const { version } = require('mdka')
console.log(version()) // e.g. "2.3.0"
```

## TypeScript

Type definitions are bundled. No `@types/` package is needed:

```ts
import {
  htmlToMarkdown,
  htmlToMarkdownWith,
  htmlToMarkdownAsync,
  htmlToMarkdownMany,
  htmlFileToMarkdown,
  htmlFilesToMarkdown,
  type JsConversionOptions,
  type ConvertResult,
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
