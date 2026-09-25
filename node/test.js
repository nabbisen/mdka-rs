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

; (async () => {
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
    // RFC 010: `*` is escaped only where it could open or close emphasis.
    assert.strictEqual(htmlToMarkdown('<p>2 * 3</p>'), '2 * 3\n')
    const md = htmlToMarkdown('<p>2 *3*</p>')
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
    assert.ok(results[0].includes('# A'), `got: ${results[0]}`)
    assert.ok(results[1].includes('## B'), `got: ${results[1]}`)
    assert.ok(results[2].includes('C'), `got: ${results[2]}`)
    assert.ok(results[3].includes('- D'), `got: ${results[3]}`)
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
    assert.ok(results[0].dest, 'dest missing')
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

  // ── 3.0: FileOutcome, and one failing file does not abort the batch ───────
  await run('htmlFilesToMarkdown: every entry is a FileOutcome with src, dest|error and ok', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const outDir = path.join(tmp, 'out')
    const good = path.join(tmp, 'good.html')
    const missing = path.join(tmp, 'missing.html')
    fs.writeFileSync(good, '<h1>Good</h1>')

    const [a, b] = await htmlFilesToMarkdown([good, missing], outDir)

    // The outcome names its own file, so a failure needs no lookup.
    assert.equal(a.src, good)
    assert.equal(b.src, missing)
    // `ok` is present in Node (it was not before 3.0) and says which of dest/error is set.
    assert.strictEqual(a.ok, true)
    assert.strictEqual(a.dest, path.join(outDir, 'good.md'))
    assert.strictEqual(a.error, undefined)
    assert.strictEqual(b.ok, false)
    assert.strictEqual(b.dest, undefined)
    assert.match(b.error, /^IO error: /)
    for (const o of [a, b]) {
      assert.strictEqual(o.ok, o.dest !== undefined, 'ok must equal (dest is set)')
      assert.strictEqual(o.ok, o.error === undefined, 'exactly one of dest and error is set')
    }
    fs.rmSync(tmp, { recursive: true })
  })

  await run('htmlFilesToMarkdown: a failing file does not reject or abort the batch; survivors keep their destinations', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const outDir = path.join(tmp, 'out')
    const files = ['a', 'missing', 'b'].map((n) => path.join(tmp, `${n}.html`))
    fs.writeFileSync(files[0], '<h1>A</h1>')
    fs.writeFileSync(files[2], '<h1>B</h1>')

    const results = await htmlFilesToMarkdown(files, outDir) // must resolve, not reject
    assert.deepEqual(results.map((r) => r.src), files, 'outcomes come back in input order')
    assert.deepEqual(results.map((r) => r.ok), [true, false, true])
    assert.ok(fs.existsSync(path.join(outDir, 'a.md')) && fs.existsSync(path.join(outDir, 'b.md')))
    assert.equal(results[2].dest, path.join(outDir, 'b.md'), 'the file after the failure was still converted')
    fs.rmSync(tmp, { recursive: true })
  })

  // What differs by arity, on purpose: one file rejects, many report per file --
  // but a failure of the *call* (the output directory cannot be created) still
  // rejects, since there is no per-file outcome to put it in.
  await run('htmlFilesToMarkdown: an uncreatable outDir rejects the call', async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-'))
    const blocker = path.join(tmp, 'blocker')
    fs.writeFileSync(blocker, 'not a directory')
    const f = path.join(tmp, 'a.html')
    fs.writeFileSync(f, '<h1>A</h1>')
    await assert.rejects(htmlFilesToMarkdown([f], path.join(blocker, 'out')), /^Error: cannot create out_dir: /)
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
  ; (async () => {
    const {
      htmlToMarkdown, htmlToMarkdownAsync, htmlFilesToMarkdown, htmlFileToMarkdown,
      htmlToMarkdownMany
    } = require('./index')
    const fs = require('fs'), path = require('path'), os = require('os')

    console.log('\n=== ConversionOptions / mode tests ===\n')

    await run('htmlToMarkdown: minimal drops nav', () => {
      const md = htmlToMarkdown(
        '<nav><a href="/">Home</a></nav><main><p>Content</p></main>',
        { mode: 'minimal', dropInteractiveShell: true }
      )
      assert.ok(!md.includes('Home'), `nav leaked: ${md}`)
      assert.ok(md.includes('Content'), `content missing: ${md}`)
    })

    await run('htmlToMarkdown: balanced keeps aria', () => {
      const md = htmlToMarkdown(
        '<button aria-label="close">X</button>',
        { mode: 'balanced' }
      )
      // aria-label は前処理で保持されるが MD には出ない（テキストのみ）
      assert.ok(md.includes('X'), `button text missing: ${md}`)
    })

    await run('htmlToMarkdown: balanced emits the heading anchor', () => {
      // Previously asserted md.includes('# Title'). RFC 005 Slice B1 gave
      // preserve_ids (default true outside minimal) a real effect: it now
      // emits an anchor as the heading's own leading content, so the
      // substring '# Title' no longer appears -- '# <a id="t"></a>Title'
      // does instead. Asserting the exact string, not a weakened substring,
      // so this test keeps proving what it actually converts to.
      const md = htmlToMarkdown(
        '<article aria-labelledby="t"><h1 id="t">Title</h1><p>Body</p></article>',
        { mode: 'balanced' }
      )
      assert.strictEqual(md, '# <a id="t"></a>Title\n\nBody\n', `unexpected output: ${md}`)
    })

    await run('htmlToMarkdown: a wrapper keeps its separation in both modes', () => {
      // Bare-sibling-text fixture, not a block-element fixture: neighbouring
      // blocks' own spacing dominates the output otherwise. Minimal unwraps
      // the wrapper and balanced renders it; RFC 036 §5.2 / slice 036d made
      // the two write the same bytes.
      const html = 'Before<div class="wrap"><span>inner</span></div>After'
      for (const mode of ['balanced', 'minimal']) {
        assert.strictEqual(htmlToMarkdown(html, { mode }), 'Before\n\ninner\n\nAfter\n', mode)
      }
    })

    // Warnings run in a fresh child process rather than sharing this file's
    // process: process.emitWarning delivers the 'warning' event
    // asynchronously (queued, not inline), so a warning from an earlier test
    // in this same file could still be in flight when a later test's listener
    // attaches. A clean subprocess has no such history.
    function runWarningCheck(optionsLiteral) {
      const { execFileSync } = require('child_process')
      const indexPath = path.resolve(__dirname, 'index.js')
      const script = `
        const { htmlToMarkdown } = require(${JSON.stringify(indexPath)})
        const warnings = []
        process.on('warning', (w) => warnings.push({ name: w.name, message: w.message }))
        htmlToMarkdown('<p class="x">Hi</p>', ${optionsLiteral})
        setImmediate(() => { process.stdout.write(JSON.stringify(warnings)) })
      `
      const out = execFileSync(process.execPath, ['-e', script], { encoding: 'utf8' })
      return JSON.parse(out)
    }

    await run('htmlToMarkdown: nothing warns any more -- no options, balanced, minimal', () => {
      for (const literal of ['undefined', '{}', "{ mode: 'balanced' }", "{ mode: 'minimal' }"]) {
        const warnings = runWarningCheck(literal)
        assert.equal(warnings.length, 0, `${literal}: expected no warnings, got ${JSON.stringify(warnings)}`)
      }
    })

    // 3.0: `strict`, `semantic` and `preserve` were removed. A removed name is
    // *obsolete*, not *wrong*, so it gets its own message rather than falling
    // into "unknown conversion mode" (RFC 048 §6). FromStr is the one place
    // that message comes from; Node passes it through unchanged.
    for (const removed of ['strict', 'semantic', 'preserve']) {
      const message = `conversion mode '${removed}' was removed in 3.0; it was an alias of 'balanced'. Use 'balanced'.`

      await run(`htmlToMarkdown: mode '${removed}' throws the removed message`, () => {
        assert.throws(() => htmlToMarkdown('<p>Hi</p>', { mode: removed }), { name: 'Error', message })
      })

      await run(`htmlToMarkdownAsync: mode '${removed}' rejects with the removed message`, async () => {
        await assert.rejects(htmlToMarkdownAsync('<p>Hi</p>', { mode: removed }), { name: 'Error', message })
      })
    }

    await run('htmlToMarkdownMany: a removed mode throws the removed message too', () => {
      assert.throws(
        () => htmlToMarkdownMany(['<p>Hi</p>'], { mode: 'preserve' }),
        { name: 'Error', message: "conversion mode 'preserve' was removed in 3.0; it was an alias of 'balanced'. Use 'balanced'." }
      )
    })

    await run('htmlToMarkdown: a removed mode is recognised in any case', () => {
      assert.throws(
        () => htmlToMarkdown('<p>Hi</p>', { mode: 'STRICT' }),
        { name: 'Error', message: /^conversion mode 'strict' was removed in 3\.0/ }
      )
    })

    // 3.0 (RFC 048 §4.2): napi drops object keys it has no field for, which after
    // the removals would have left `{ preserveClasses: true }` from plain
    // JavaScript with no error and no warning, while TypeScript rejects it,
    // Python raises TypeError and the CLI exits 1. An unrecognised key is now an
    // error that names it, and a key removed in 3.0 says *removed*, as a removed
    // mode name does. Checked as the argument crosses, so it holds for every
    // function that takes options -- including the async ones.
    const REMOVED_OPTIONS = {
      preserveClasses: 'Markdown has no attribute syntax, so it never changed the output',
      preserveDataAttrs: 'Markdown has no attribute syntax, so it never changed the output',
      preserveAriaAttrs: 'Markdown has no attribute syntax, so it never changed the output',
      preserveUnknownAttrs: 'Markdown has no attribute syntax, so it never changed the output',
      dropPresentationAttrs: 'Markdown has no attribute syntax, so it never changed the output',
      unwrapUnknownWrappers: 'it could not change the output',
    }
    for (const [option, why] of Object.entries(REMOVED_OPTIONS)) {
      await run(`htmlToMarkdown: removed option '${option}' throws, naming it, and says removed`, () => {
        assert.throws(
          () => htmlToMarkdown('<p>Hi</p>', { mode: 'balanced', [option]: true }),
          {
            name: 'Error',
            message: `option '${option}' was removed in 3.0; ${why}. Remove it: the output is the same without it.`
          }
        )
      })
    }

    await run('htmlToMarkdown: an unknown option throws, naming it and the valid ones', () => {
      assert.throws(
        () => htmlToMarkdown('<p>Hi</p>', { mdoe: 'minimal' }),
        { name: 'Error', message: "unknown option 'mdoe'. Valid options: mode, preserveIds, dropInteractiveShell" }
      )
    })

    await run('every options-taking function rejects a bad key, sync or async', async () => {
      const bad = { preserveClasses: true }
      const removed = /option 'preserveClasses' was removed in 3\.0/
      assert.throws(() => htmlToMarkdown('<p>Hi</p>', bad), removed)
      assert.throws(() => htmlToMarkdownMany(['<p>Hi</p>'], bad), removed)
      await assert.rejects(htmlToMarkdownAsync('<p>Hi</p>', bad), removed)
      await assert.rejects(htmlFileToMarkdown('/no/such/file.html', null, bad), removed)
      await assert.rejects(htmlFilesToMarkdown([], os.tmpdir(), bad), removed)
    })

    await run('the three valid keys, null and undefined are all accepted', () => {
      const html = '<nav>n</nav><h1 id="x">T</h1>'
      assert.strictEqual(htmlToMarkdown(html, null), htmlToMarkdown(html))
      assert.strictEqual(htmlToMarkdown(html, undefined), htmlToMarkdown(html))
      assert.strictEqual(htmlToMarkdown(html, {}), htmlToMarkdown(html))
      assert.doesNotThrow(() => htmlToMarkdown(html, { mode: 'minimal', preserveIds: false, dropInteractiveShell: true }))
    })

    // 3.0 (RFC 048 §4): Node folds `With` away -- JavaScript has optional
    // arguments, so `htmlToMarkdown(html, options?)` is the idiom. Rust and
    // Python keep their `_with` pairs on purpose.
    await run('no `…With` function is exported, and every options-taking one takes them optionally', () => {
      const exported = Object.keys(require('./index'))
      assert.deepStrictEqual(exported.filter((n) => /With/.test(n)), [], 'a …With name is still exported')
      for (const gone of ['htmlToMarkdownWith', 'htmlToMarkdownWithAsync', 'htmlFileToMarkdownWith', 'htmlFilesToMarkdownWith']) {
        assert.strictEqual(require('./index')[gone], undefined, gone)
      }
      assert.strictEqual(htmlToMarkdown('<p>Hi</p>'), htmlToMarkdown('<p>Hi</p>', { mode: 'balanced' }))
    })

    await run('htmlToMarkdownAsync: mode option respected', async () => {
      const md = await htmlToMarkdownAsync(
        '<nav>nav</nav><p>Main</p>',
        { mode: 'minimal', dropInteractiveShell: true }
      )
      assert.ok(md.includes('Main'), `main missing: ${md}`)
      assert.ok(!md.includes('nav'), `nav leaked: ${md}`)
    })

    await run('htmlFilesToMarkdown: minimal mode', async () => {
      const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-mode-'))
      const out = path.join(tmp, 'out')
      fs.mkdirSync(out)
      const f = path.join(tmp, 'test.html')
      fs.writeFileSync(f, '<nav>nav</nav><h1>Title</h1><p>Body</p>')
      const results = await htmlFilesToMarkdown([f], out, {
        mode: 'minimal', dropInteractiveShell: true
      })
      assert.equal(results.length, 1)
      const content = fs.readFileSync(results[0].dest, 'utf8')
      assert.ok(content.includes('# Title'), `title missing: ${content}`)
      fs.rmSync(tmp, { recursive: true })
    })

    await run('htmlToMarkdownMany: matches mapping htmlToMarkdown over the inputs', () => {
      const inputs = ['<h1>A</h1>', '<p>B</p>', '<em>C</em>']
      const many = htmlToMarkdownMany(inputs)
      assert.deepEqual(many, inputs.map((h) => htmlToMarkdown(h)))
    })

    await run('htmlToMarkdownMany: options apply the same as htmlToMarkdown', () => {
      const inputs = ['<nav><a href="/">Home</a></nav><main><p>Content</p></main>']
      const opts = { mode: 'minimal', dropInteractiveShell: true }
      const many = htmlToMarkdownMany(inputs, opts)
      assert.deepEqual(many, inputs.map((h) => htmlToMarkdown(h, opts)))
    })

    await run('htmlToMarkdownMany: empty input returns empty array', () => {
      assert.deepEqual(htmlToMarkdownMany([]), [])
    })

    await run('ConversionOptions: an unknown mode is a hard error, not a fallback', () => {
      assert.throws(
        () => {
          const md = htmlToMarkdown('<h1>Hi</h1>', { mode: 'nonexistent' })
        },
        {
          name: 'Error',
          message: 'unknown conversion mode: nonexistent'
        }
      );
    })

    console.log(`\n${'─'.repeat(40)}`)
    console.log(`  passed: ${passed}  failed: ${failed}`)
    if (failed > 0) { process.exit(1) }
  })()

  // ── htmlFileToMarkdown テスト ──────────────────────────────────────────────
  ; (async () => {
    const { htmlFileToMarkdown } = require('./index')
    const fs = require('fs'), path = require('path'), os = require('os')

    console.log('\n=== htmlFileToMarkdown tests ===\n')

    await run('htmlFileToMarkdown: same dir output (outDir=null)', async () => {
      const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-single-'))
      const src = path.join(tmp, 'page.html')
      fs.writeFileSync(src, '<h1>Single File</h1><p>Content</p>')

      // 3.0: a single file resolves to the destination path -- a string, not an
      // object with an `error` field that could never be set.
      const dest = await htmlFileToMarkdown(src)
      assert.equal(typeof dest, 'string', `expected the destination path, got ${JSON.stringify(dest)}`)
      const expectedDest = path.join(tmp, 'page.md')
      assert.equal(dest, expectedDest, `expected ${expectedDest}, got ${dest}`)
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

      const dest = await htmlFileToMarkdown(src, outDir)
      assert.equal(dest, path.join(outDir, 'article.md'))
      const content = fs.readFileSync(dest, 'utf8')
      assert.ok(content.includes('## Article'), `got: ${content}`)
      fs.rmSync(tmp, { recursive: true })
    })

    // 3.0: a single file fails *the call*, through JavaScript's own channel -- a
    // rejection -- not through a result object. (The bulk function reports per file.)
    await run('htmlFileToMarkdown: a nonexistent file rejects with an Error naming the IO failure', async () => {
      await assert.rejects(
        htmlFileToMarkdown('/no/such/file.html'),
        (e) => {
          assert.ok(e instanceof Error, `expected Error, got ${e}`)
          assert.match(e.message, /^IO error: /)
          return true
        }
      )
      await assert.rejects(htmlFileToMarkdown('/no/such/file.html', null, { mode: 'minimal' }), /^Error: IO error: /)
    })

    await run('htmlFileToMarkdown: mode option', async () => {
      const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-single-'))
      const src = path.join(tmp, 'spa.html')
      fs.writeFileSync(src, '<nav>nav</nav><h1>Title</h1><p>Body</p>')

      const dest = await htmlFileToMarkdown(src, null, {
        mode: 'minimal', dropInteractiveShell: true
      })
      const content = fs.readFileSync(dest, 'utf8')
      assert.ok(content.includes('# Title'), `title missing: ${content}`)
      assert.ok(!content.includes('nav'), `nav leaked: ${content}`)
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
      const d1 = await htmlFileToMarkdown(src, out1)
      const results = await htmlFilesToMarkdown([src], out2)

      const c1 = fs.readFileSync(d1, 'utf8')
      const c2 = fs.readFileSync(results[0].dest, 'utf8')
      assert.equal(c1, c2, `single vs bulk output mismatch:\n${c1}\nvs\n${c2}`)
      fs.rmSync(tmp, { recursive: true })
    })

    console.log(`\n${'─'.repeat(40)}`)
    console.log(`  passed: ${passed}  failed: ${failed}`)
    if (failed > 0) { process.exit(1) }
  })()
