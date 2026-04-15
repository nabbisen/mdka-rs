'use strict'

const assert = require('assert/strict')
const { htmlToMarkdown, htmlToMarkdownAsync, htmlFilesToMarkdown, version } = require('./index')
const fs = require('fs')
const path = require('path')
const os = require('os')

let passed = 0
let failed = 0

async function run(name, fn) {
  try {
    await fn()
    console.log(`  ✅ ${name}`)
    passed++
  } catch (e) {
    console.error(`  ❌ ${name}`)
    console.error(`     ${e.message}`)
    failed++
  }
}

;(async () => {
  console.log('\n=== mdka Node.js binding tests ===\n')

  // ── 同期 API ──────────────────────────────────────────────────────────
  await run('version() returns string', () => {
    const v = version()
    assert.equal(typeof v, 'string')
    assert.match(v, /^\d+\.\d+\.\d+/)
  })

  await run('htmlToMarkdown: h1', () => {
    const md = htmlToMarkdown('<h1>Hello</h1>')
    assert.ok(md.includes('# Hello'), `got: ${JSON.stringify(md)}`)
  })

  await run('htmlToMarkdown: h1-h6', () => {
    for (let i = 1; i <= 6; i++) {
      const md = htmlToMarkdown(`<h${i}>T</h${i}>`)
      assert.ok(md.includes('#'.repeat(i) + ' T'), `h${i} failed: ${md}`)
    }
  })

  await run('htmlToMarkdown: paragraph', () => {
    const md = htmlToMarkdown('<p>Hello world</p>')
    assert.ok(md.trim() === 'Hello world', `got: ${JSON.stringify(md)}`)
  })

  await run('htmlToMarkdown: strong / em', () => {
    const md = htmlToMarkdown('<p><strong>bold</strong> and <em>italic</em></p>')
    assert.ok(md.includes('**bold**'), `got: ${md}`)
    assert.ok(md.includes('*italic*'), `got: ${md}`)
  })

  await run('htmlToMarkdown: unordered list', () => {
    const md = htmlToMarkdown('<ul><li>A</li><li>B</li><li>C</li></ul>')
    assert.ok(md.includes('- A'), `got: ${md}`)
    assert.ok(md.includes('- B'), `got: ${md}`)
  })

  await run('htmlToMarkdown: ordered list with start', () => {
    const md = htmlToMarkdown('<ol start="3"><li>Three</li><li>Four</li></ol>')
    assert.ok(md.includes('3. Three'), `got: ${md}`)
    assert.ok(md.includes('4. Four'), `got: ${md}`)
  })

  await run('htmlToMarkdown: link', () => {
    const md = htmlToMarkdown('<a href="https://example.com" title="Ex">Click</a>')
    assert.ok(md.includes('[Click](https://example.com "Ex")'), `got: ${md}`)
  })

  await run('htmlToMarkdown: image', () => {
    const md = htmlToMarkdown('<img src="img.png" alt="Alt text">')
    assert.ok(md.includes('![Alt text](img.png)'), `got: ${md}`)
  })

  await run('htmlToMarkdown: code block with language', () => {
    const md = htmlToMarkdown('<pre><code class="language-rust">fn main() {}</code></pre>')
    assert.ok(md.includes('```rust\nfn main() {}'), `got: ${md}`)
  })

  await run('htmlToMarkdown: blockquote nested', () => {
    const md = htmlToMarkdown('<blockquote><blockquote><p>deep</p></blockquote></blockquote>')
    assert.ok(md.includes('> > '), `expected "> > ", got: ${JSON.stringify(md)}`)
  })

  await run('htmlToMarkdown: script/style ignored', () => {
    const md = htmlToMarkdown('<script>alert(1)</script><p>Text</p><style>body{}</style>')
    assert.ok(!md.includes('alert'), `script leaked: ${md}`)
    assert.ok(md.includes('Text'), `text missing: ${md}`)
  })

  await run('htmlToMarkdown: markdown escape', () => {
    const md = htmlToMarkdown('<p>2 * 3</p>')
    assert.ok(md.includes('\\*'), `expected escape, got: ${md}`)
  })

  await run('htmlToMarkdown: deep nest no crash', () => {
    const open = '<div>'.repeat(5000)
    const close = '</div>'.repeat(5000)
    const md = htmlToMarkdown(`${open}<p>deep</p>${close}`)
    assert.ok(md.includes('deep'), `got: ${md.slice(0, 100)}`)
  })

  await run('htmlToMarkdown: empty string', () => {
    const md = htmlToMarkdown('')
    assert.ok(md.trim() === '', `got: ${JSON.stringify(md)}`)
  })

  await run('htmlToMarkdown: output ends with single newline', () => {
    const md = htmlToMarkdown('<p>Hello</p>')
    assert.ok(md.endsWith('\n'), 'no trailing newline')
    assert.ok(!md.endsWith('\n\n'), 'double trailing newline')
  })

  // ── 非同期 API ────────────────────────────────────────────────────────
  await run('htmlToMarkdownAsync: basic', async () => {
    const md = await htmlToMarkdownAsync('<h1>Async</h1>')
    assert.ok(md.includes('# Async'), `got: ${md}`)
  })

  await run('htmlToMarkdownAsync: concurrent calls', async () => {
    const inputs = ['<h1>A</h1>', '<h2>B</h2>', '<p>C</p>', '<ul><li>D</li></ul>']
    const results = await Promise.all(inputs.map(h => htmlToMarkdownAsync(h)))
    assert.ok(results[0].includes('# A'),  `got: ${results[0]}`)
    assert.ok(results[1].includes('## B'), `got: ${results[1]}`)
    assert.ok(results[2].includes('C'),    `got: ${results[2]}`)
    assert.ok(results[3].includes('- D'),  `got: ${results[3]}`)
  })

  await run('htmlToMarkdownAsync: large input', async () => {
    const big = '<p>' + 'word '.repeat(10000) + '</p>'
    const md = await htmlToMarkdownAsync(big)
    assert.ok(md.includes('word'), `got nothing: ${md.slice(0, 50)}`)
  })

  // ── ファイル変換 API ──────────────────────────────────────────────────
  await run('htmlFilesToMarkdown: basic', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const outDir = path.join(tmp, 'out')
    fs.mkdirSync(outDir)
    const htmlFile = path.join(tmp, 'page.html')
    fs.writeFileSync(htmlFile, '<h1>File Test</h1><p>Content</p>')

    const results = await htmlFilesToMarkdown([htmlFile], outDir)
    assert.equal(results.length, 1)
    assert.ok(!results[0].error, `error: ${results[0].error}`)
    assert.ok(results[0].dest,   'dest missing')
    const content = fs.readFileSync(results[0].dest, 'utf8')
    assert.ok(content.includes('# File Test'), `content: ${content}`)

    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFilesToMarkdown: multiple files parallel', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const outDir = path.join(tmp, 'out')
    fs.mkdirSync(outDir)
    const files = [1, 2, 3, 4].map(i => {
      const p = path.join(tmp, `f${i}.html`)
      fs.writeFileSync(p, `<h${i}>H${i}</h${i}>`)
      return p
    })

    const results = await htmlFilesToMarkdown(files, outDir)
    assert.equal(results.length, 4)
    for (const r of results) {
      assert.ok(!r.error, `error for ${r.src}: ${r.error}`)
    }

    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFilesToMarkdown: nonexistent file returns error', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const results = await htmlFilesToMarkdown(['/nonexistent/file.html'], tmp)
    assert.equal(results.length, 1)
    assert.ok(results[0].error, 'expected error for missing file')
    fs.rmSync(tmp, { recursive: true })
  })

  // ── 結果サマリ ────────────────────────────────────────────────────────
  console.log(`\n${'─'.repeat(40)}`)
  console.log(`  passed: ${passed}  failed: ${failed}`)
  if (failed > 0) {
    console.log('  RESULT: ❌ FAIL')
    process.exit(1)
  } else {
    console.log('  RESULT: ✅ ALL PASS')
  }
})()

// ── ConversionOptions / モード別テスト ───────────────────────────────────
;(async () => {
  const {
    htmlToMarkdownWith, htmlToMarkdownWithAsync, htmlFilesToMarkdownWith
  } = require('./index')
  const fs = require('fs'), path = require('path'), os = require('os')

  console.log('\n=== ConversionOptions / mode tests ===\n')

  await run('htmlToMarkdownWith: minimal drops nav', () => {
    const md = htmlToMarkdownWith(
      '<nav><a href="/">Home</a></nav><main><p>Content</p></main>',
      { mode: 'minimal', dropInteractiveShell: true }
    )
    assert.ok(!md.includes('Home'),   `nav leaked: ${md}`)
    assert.ok(md.includes('Content'), `content missing: ${md}`)
  })

  await run('htmlToMarkdownWith: balanced keeps aria', () => {
    const md = htmlToMarkdownWith(
      '<button aria-label="close">X</button>',
      { mode: 'balanced' }
    )
    // aria-label は前処理で保持されるが MD には出ない（テキストのみ）
    assert.ok(md.includes('X'), `button text missing: ${md}`)
  })

  await run('htmlToMarkdownWith: strict keeps class in output html (pre-process)', () => {
    // strict モードは class を保持するが変換後 MD には影響しない
    const md = htmlToMarkdownWith('<p class="intro">Hello</p>', { mode: 'strict' })
    assert.ok(md.includes('Hello'), `text missing: ${md}`)
  })

  await run('htmlToMarkdownWith: preserve mode keeps comments via pre-processing', () => {
    const md = htmlToMarkdownWith('<!-- meta --><p>Text</p>', { mode: 'preserve' })
    assert.ok(md.includes('Text'), `text missing: ${md}`)
  })

  await run('htmlToMarkdownWith: semantic keeps aria-label text', () => {
    const md = htmlToMarkdownWith(
      '<article aria-labelledby="t"><h1 id="t">Title</h1><p>Body</p></article>',
      { mode: 'semantic', preserveAriaAttrs: true }
    )
    assert.ok(md.includes('# Title'), `heading missing: ${md}`)
    assert.ok(md.includes('Body'),    `body missing: ${md}`)
  })

  await run('htmlToMarkdownWithAsync: mode option respected', async () => {
    const md = await htmlToMarkdownWithAsync(
      '<nav>nav</nav><p>Main</p>',
      { mode: 'minimal', dropInteractiveShell: true }
    )
    assert.ok(md.includes('Main'), `main missing: ${md}`)
    assert.ok(!md.includes('nav'), `nav leaked: ${md}`)
  })

  await run('htmlFilesToMarkdownWith: minimal mode', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-mode-'))
    const out = path.join(tmp, 'out')
    fs.mkdirSync(out)
    const f = path.join(tmp, 'test.html')
    fs.writeFileSync(f, '<nav>nav</nav><h1>Title</h1><p>Body</p>')
    const results = await htmlFilesToMarkdownWith([f], out, {
      mode: 'minimal', dropInteractiveShell: true
    })
    assert.equal(results.length, 1)
    const content = fs.readFileSync(results[0].dest, 'utf8')
    assert.ok(content.includes('# Title'), `title missing: ${content}`)
    fs.rmSync(tmp, { recursive: true })
  })

  await run('ConversionOptions: unknown mode falls back to balanced', () => {
    const md = htmlToMarkdownWith('<h1>Hi</h1>', { mode: 'nonexistent' })
    assert.ok(md.includes('# Hi'), `got: ${md}`)
  })

  console.log(`\n${'─'.repeat(40)}`)
  console.log(`  passed: ${passed}  failed: ${failed}`)
  if (failed > 0) { process.exit(1) }
})()

// ── htmlFileToMarkdown テスト ──────────────────────────────────────────────
;(async () => {
  const { htmlFileToMarkdown, htmlFileToMarkdownWith } = require('./index')
  const fs = require('fs'), path = require('path'), os = require('os')

  console.log('\n=== htmlFileToMarkdown tests ===\n')

  await run('htmlFileToMarkdown: same dir output (outDir=null)', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-single-'))
    const src = path.join(tmp, 'page.html')
    fs.writeFileSync(src, '<h1>Single File</h1><p>Content</p>')

    const r = await htmlFileToMarkdown(src)
    assert.ok(r.src,  'src missing')
    assert.ok(r.dest, 'dest missing')
    const expectedDest = path.join(tmp, 'page.md')
    assert.equal(r.dest, expectedDest, `expected ${expectedDest}, got ${r.dest}`)
    assert.ok(fs.existsSync(expectedDest), 'output file not created')
    const content = fs.readFileSync(expectedDest, 'utf8')
    assert.ok(content.includes('# Single File'), `got: ${content}`)
    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFileToMarkdown: explicit outDir', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-single-'))
    const outDir = path.join(tmp, 'out')
    fs.mkdirSync(outDir)
    const src = path.join(tmp, 'article.html')
    fs.writeFileSync(src, '<h2>Article</h2><p>Body</p>')

    const r = await htmlFileToMarkdown(src, outDir)
    assert.equal(r.dest, path.join(outDir, 'article.md'))
    const content = fs.readFileSync(r.dest, 'utf8')
    assert.ok(content.includes('## Article'), `got: ${content}`)
    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFileToMarkdown: nonexistent file rejects', async () => {
    try {
      await htmlFileToMarkdown('/no/such/file.html')
      assert.fail('should have thrown')
    } catch (e) {
      assert.ok(e instanceof Error, `expected Error, got ${e}`)
    }
  })

  await run('htmlFileToMarkdownWith: mode option', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-single-'))
    const src = path.join(tmp, 'spa.html')
    fs.writeFileSync(src, '<nav>nav</nav><h1>Title</h1><p>Body</p>')

    const r = await htmlFileToMarkdownWith(src, null, {
      mode: 'minimal', dropInteractiveShell: true
    })
    const content = fs.readFileSync(r.dest, 'utf8')
    assert.ok(content.includes('# Title'), `title missing: ${content}`)
    assert.ok(!content.includes('nav'),    `nav leaked: ${content}`)
    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFileToMarkdown consistency with htmlFilesToMarkdown', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-consist-'))
    const out1 = path.join(tmp, 'out1')
    const out2 = path.join(tmp, 'out2')
    fs.mkdirSync(out1); fs.mkdirSync(out2)
    const src = path.join(tmp, 'test.html')
    fs.writeFileSync(src, '<h1>Hello</h1><p>World <strong>bold</strong></p>')

    const { htmlFilesToMarkdown } = require('./index')
    const r1 = await htmlFileToMarkdown(src, out1)
    const results = await htmlFilesToMarkdown([src], out2)

    const c1 = fs.readFileSync(r1.dest, 'utf8')
    const c2 = fs.readFileSync(results[0].dest, 'utf8')
    assert.equal(c1, c2, `single vs bulk output mismatch:\n${c1}\nvs\n${c2}`)
    fs.rmSync(tmp, { recursive: true })
  })

  console.log(`\n${'─'.repeat(40)}`)
  console.log(`  passed: ${passed}  failed: ${failed}`)
  if (failed > 0) { process.exit(1) }
})()
