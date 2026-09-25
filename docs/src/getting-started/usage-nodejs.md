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

Every Node.js function takes its options as an optional last argument — there is no
separate `…With` function (they were removed in 3.0):

```js
const { htmlToMarkdown, htmlToMarkdownAsync } = require('mdka')

const html = '<nav>menu</nav><h1>Title</h1><p>Body</p>'

// Strip nav/header/footer — useful for content extraction
const md = htmlToMarkdown(html, {
  mode: 'minimal',
  dropInteractiveShell: true,
})

// Async version
async function main() {
  const mdAsync = await htmlToMarkdownAsync(html, { mode: 'balanced' })
  console.log(mdAsync)
}
main()
```

Available mode strings: `"balanced"` (default) and `"minimal"`. `"strict"`,
`"semantic"` and `"preserve"` were aliases of `"balanced"` and were removed in
3.0: passing one **throws** (or, from `htmlToMarkdownAsync` and the file functions, **rejects**) with
`conversion mode 'strict' was removed in 3.0; it was an alias of 'balanced'. Use
'balanced'.` — see [Conversion Modes](../api/modes.md).

## Converting Multiple Strings

```js
const { htmlToMarkdownMany } = require('mdka')

const pages = ['<h1>A</h1>', '<p>B</p>', '<ul><li>C</li></ul>']
const results = htmlToMarkdownMany(pages)
// ['# A\n', 'B\n', '- C\n']

// With options — the same shape as htmlToMarkdown's
const withOpts = htmlToMarkdownMany(pages, { mode: 'minimal' })
```

Each input is converted independently and the results keep the input order.

## Single File Conversion

```js
const { htmlFileToMarkdown } = require('mdka')

async function main() {
  // Output to same directory: page.html → page.md
  // Resolves to the path that was written — a string.
  const dest = await htmlFileToMarkdown('page.html')
  console.log(`page.html → ${dest}`)

  // Output to specific directory
  const outDir = await htmlFileToMarkdown('page.html', 'out/')

  // With options (outDir may be null to keep the default)
  const withOpts = await htmlFileToMarkdown('page.html', 'out/', {
    mode: 'minimal',
    dropInteractiveShell: true,
  })
}
main()
```

A single file **fails the call**: if it cannot be read or written, the promise
**rejects** with an `Error` (`IO error: …`). There is no result object to inspect.

## Bulk Parallel Conversion

```js
const { htmlFilesToMarkdown } = require('mdka')

const files = ['a.html', 'b.html', 'c.html']

async function main() {
  const results = await htmlFilesToMarkdown(files, 'out/')

  // One FileOutcome per input, in input order: src, ok, and exactly one of
  // dest (when ok) and error (when not).
  for (const r of results) {
    if (r.ok) console.log(`${r.src} → ${r.dest}`)
    else      console.error(`${r.src}: ${r.error}`)
  }

  // With options
  const withOpts = await htmlFilesToMarkdown(files, 'out/', {
    mode: 'minimal',
  })
}
main()
```

**A failing file does not reject the promise and does not stop the others**; its entry
has `ok: false`. The promise rejects only if the call as a whole cannot proceed —
the output directory cannot be created. That is the difference from
`htmlFileToMarkdown`, on purpose: with one file, failing the call is the answer; with
many, one bad file must not hide the rest.

Two inputs whose output names collide — `a/index.html` and `b/index.html` both
becoming `out/index.md` — are not both converted. The first in the array wins
and each later one comes back with `ok: false` and `error` set, rather than
silently overwriting.

**Four options were removed in 3.0:** `preserveClasses`, `preserveDataAttrs`,
`preserveAriaAttrs` and `unwrapUnknownWrappers`. None of them ever changed the
output. In TypeScript, passing one is a compile error:

```
TS2353: Object literal may only specify known properties, and
'preserveClasses' does not exist in type 'JsConversionOptions'.
```

**In plain JavaScript it is an error too, from 3.0.** An option key that is not
recognised is rejected with a message that **names it** — a key removed in 3.0 says
`option 'preserveClasses' was removed in 3.0; …` and a misspelt one says
`unknown option 'mdoe'. Valid options: mode, preserveIds, dropInteractiveShell`. The
synchronous functions throw it and the Promise-returning ones reject with it. (Until
3.0 napi silently dropped unknown keys, so a removed option that was still passed
told you nothing.)

`JsConversionOptions` has three fields: `mode`, `preserveIds` and `dropInteractiveShell`.

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
  htmlToMarkdownAsync,
  htmlToMarkdownMany,
  htmlFileToMarkdown,
  htmlFilesToMarkdown,
  type JsConversionOptions,
  type FileOutcome,
} from 'mdka'

const html: string = '<h1>Title</h1>'

const opts: JsConversionOptions = {
  mode: 'minimal',
  dropInteractiveShell: true,
}
const md: string = htmlToMarkdown(html, opts)
```

The options type is exported as **`JsConversionOptions`**, not
`ConversionOptions` — the name comes from the napi-rs binding rather than from
the Rust `ConversionOptions` it mirrors.
