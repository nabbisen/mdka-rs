'use strict'

// RFC 040 Part B -- tests for loader.js, the package entry point.
//
// test.js is deliberately untouched: it requires './index' directly and so
// never went through the entry point. The last test here runs it through
// loader.js instead, unchanged.

const assert = require('assert/strict')
const { spawnSync } = require('child_process')
const fs = require('fs')
const os = require('os')
const path = require('path')
const { pathToFileURL } = require('url')

let passed = 0
let failed = 0

function run(name, fn) {
  try {
    fn()
    console.log(`  ✅ ${name}`)
    passed++
  } catch (e) {
    console.error(`  ❌ ${name}`)
    console.error(`     ${e.message}`)
    failed++
  }
}

const INSTALLATION_PAGE = 'https://nabbisen.github.io/mdka-rs/getting-started/installation.html'
const PUBLISHED = [
  'linux-x64-gnu',
  'linux-x64-musl',
  'linux-arm64-gnu',
  'linux-arm64-musl',
  'darwin-arm64',
  'win32-x64-msvc',
]
// Anything that would send a reader to npm, or to a reinstall that cannot help.
const NPM_BLAME = /npm has a bug|npm\/cli|package-lock|node_modules directory|npm i\b|npm install|reinstall|try .* again/i

// A copy of the real loader.js and the real, generated index.js in a directory
// with no native binary and no @mdka/* package beside it: index.js then fails
// exactly as it does on a platform nothing was published for.
function bare(indexSource) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'mdka-loader-'))
  for (const f of ['loader.js', 'index.js', 'package.json']) {
    fs.copyFileSync(path.join(__dirname, f), path.join(dir, f))
  }
  if (indexSource !== undefined) fs.writeFileSync(path.join(dir, 'index.js'), indexSource)
  return dir
}

// Require the bare loader while pretending to be another platform; returns the
// thrown error. `libc` is 'glibc', 'musl' or undefined (not linux).
function loadAs({ platform, arch, libc }, indexSource) {
  const dir = bare(indexSource)
  const saved = {
    platform: Object.getOwnPropertyDescriptor(process, 'platform'),
    arch: Object.getOwnPropertyDescriptor(process, 'arch'),
    getReport: process.report.getReport,
  }
  try {
    Object.defineProperty(process, 'platform', { value: platform, configurable: true })
    Object.defineProperty(process, 'arch', { value: arch, configurable: true })
    process.report.getReport = () =>
      libc === 'musl'
        ? { header: {}, sharedObjects: ['/lib/ld-musl-x86_64.so.1'] }
        : { header: { glibcVersionRuntime: '2.39' }, sharedObjects: [] }
    require(path.join(dir, 'loader.js'))
  } catch (e) {
    return e
  } finally {
    Object.defineProperty(process, 'platform', saved.platform)
    Object.defineProperty(process, 'arch', saved.arch)
    process.report.getReport = saved.getReport
  }
  assert.fail(`loading as ${platform}/${arch} was expected to throw`)
}

console.log('\n=== mdka loader.js tests (RFC 040 Part B) ===\n')

// ── criteria 3 and 4: a platform with no published binary ────────────────
const unsupported = [
  ['linux riscv64 glibc', { platform: 'linux', arch: 'riscv64', libc: 'glibc' }, 'linux-riscv64-gnu'],
  ['linux arm (32-bit) musl', { platform: 'linux', arch: 'arm', libc: 'musl' }, 'linux-arm-musleabihf'],
  ['macOS Intel', { platform: 'darwin', arch: 'x64' }, 'darwin-x64'],
  ['Windows ARM', { platform: 'win32', arch: 'arm64' }, 'win32-arm64-msvc'],
]
for (const [label, env, name] of unsupported) {
  run(`unsupported platform (${label}): names it, lists the published ones, links the page`, () => {
    const e = loadAs(env)
    assert.ok(e instanceof Error)
    assert.equal(e.code, 'MDKA_UNSUPPORTED_PLATFORM')
    assert.ok(e.message.includes(`no prebuilt binary for this platform: ${name}`), e.message)
    assert.ok(e.message.includes(`process.platform=${env.platform}`), e.message)
    assert.ok(e.message.includes(`process.arch=${env.arch}`), e.message)
    for (const p of PUBLISHED) assert.ok(e.message.includes(`  ${p}\n`), `missing ${p}:\n${e.message}`)
    assert.ok(e.message.includes(INSTALLATION_PAGE), e.message)
  })

  run(`unsupported platform (${label}): does not blame npm or prescribe a reinstall`, () => {
    const e = loadAs(env)
    assert.doesNotMatch(e.message, NPM_BLAME)
  })

  run(`unsupported platform (${label}): the real load failures survive as cause, without napi's boilerplate`, () => {
    const e = loadAs(env)
    assert.ok(e.cause instanceof Error, 'no cause attached')
    // What is attached is the chain of individual failures index.js recorded,
    // not the fixed "npm has a bug" wrapper it threw around them.
    assert.match(e.cause.message, /^Cannot find module /)
    assert.equal(e.cause.code, 'MODULE_NOT_FOUND')
    assert.ok(e.cause.cause instanceof Error, 'the earlier failures in the chain were lost')
    let depth = 0
    for (let c = e.cause; c; c = c.cause) {
      assert.doesNotMatch(c.message, NPM_BLAME, 'the npm-bug text is still reachable through cause')
      depth++
    }
    assert.ok(depth >= 2, `chain has only ${depth} link(s)`)
  })
}

run('unsupported platform: libc is reported on Linux and absent elsewhere', () => {
  assert.match(loadAs({ platform: 'linux', arch: 'riscv64', libc: 'musl' }).message, /libc=musl/)
  assert.match(loadAs({ platform: 'linux', arch: 'riscv64', libc: 'glibc' }).message, /libc=glibc/)
  assert.doesNotMatch(loadAs({ platform: 'darwin', arch: 'x64' }).message, /libc=/)
})

run('an OS index.js does not know at all is reported the same way, with the original as cause', () => {
  const e = loadAs({ platform: 'sunos', arch: 'x64' })
  assert.equal(e.code, 'MDKA_UNSUPPORTED_PLATFORM')
  assert.ok(e.message.includes('sunos-x64'), e.message)
  assert.doesNotMatch(e.message, NPM_BLAME)
  assert.match(e.cause.message, /Unsupported OS: sunos/)
})

run("index.js's other throw, which has no cause at all, is attached as the cause rather than lost", () => {
  // index.js throws this bare error when it recorded no load errors.
  const e = loadAs(
    { platform: 'linux', arch: 'riscv64', libc: 'glibc' },
    "throw new Error('Failed to load native binding')",
  )
  assert.equal(e.code, 'MDKA_UNSUPPORTED_PLATFORM')
  assert.ok(e.cause instanceof Error, 'the cause was lost')
  assert.equal(e.cause.message, 'Failed to load native binding')
  assert.doesNotMatch(e.message, NPM_BLAME)
})

run('a published platform with the no-cause throw quotes it and keeps it as the cause', () => {
  const e = loadAs({ platform: 'linux', arch: 'x64', libc: 'glibc' }, "throw new Error('Failed to load native binding')")
  assert.equal(e.code, 'MDKA_BINARY_LOAD_FAILED')
  assert.match(e.message, /Underlying error: Failed to load native binding/)
  assert.equal(e.cause.message, 'Failed to load native binding')
})

// ── a published platform whose binary will not load ──────────────────────
run('a published platform that fails to load is not called unsupported', () => {
  const e = loadAs({ platform: 'linux', arch: 'x64', libc: 'musl' })
  assert.equal(e.code, 'MDKA_BINARY_LOAD_FAILED')
  assert.ok(e.message.includes('linux-x64-musl'), e.message)
  assert.doesNotMatch(e.message, /no prebuilt binary/)
  // (index.js decides glibc/musl from the real filesystem, so on a glibc test host
  // it asks for the gnu package; what matters is that the error is quoted.)
  assert.match(e.message, /Underlying error: Cannot find module '@mdka\/lib-linux-x64-/)
  assert.doesNotMatch(e.message, NPM_BLAME)
  assert.ok(e.message.includes(INSTALLATION_PAGE), e.message)
  assert.match(e.cause.message, /^Cannot find module /)
  assert.doesNotMatch(e.cause.message, NPM_BLAME)
})

// ── the lists agree with each other ──────────────────────────────────────
const ARCH = { x86_64: 'x64', aarch64: 'arm64', arm64: 'arm64' }
function nameOf(target) {
  const parts = target.split('-')
  const arch = ARCH[parts[0]]
  const last = parts[parts.length - 1]
  if (!arch) throw new Error(`unrecognised target ${target}`)
  if (last === 'gnu' || last === 'musl') return `linux-${arch}-${last}`
  if (last === 'darwin') return `darwin-${arch}`
  if (last === 'msvc') return `win32-${arch}-msvc`
  throw new Error(`unrecognised target ${target}`)
}

run('the platforms in the message are exactly napi.targets in package.json', () => {
  const pkg = require('./package.json')
  const fromTargets = pkg.napi.targets.map(nameOf).sort()
  assert.deepEqual(fromTargets, [...PUBLISHED].sort())
  // and the message really lists these, no more and no fewer
  const listed = loadAs({ platform: 'linux', arch: 'riscv64', libc: 'glibc' })
    .message.split('Platforms with a published binary:\n')[1]
    .split('\n\n')[0]
    .split('\n')
    .map((l) => l.trim())
  assert.deepEqual(listed.sort(), [...PUBLISHED].sort())
})

run('the platforms in the message are exactly the release-npm.yaml matrix', () => {
  const yaml = fs.readFileSync(path.join(__dirname, '..', '.github', 'workflows', 'release-npm.yaml'), 'utf8')
  const matrix = [...yaml.matchAll(/^\s+napiplatform:\s*(\S+)\s*$/gm)].map((m) => m[1]).sort()
  assert.deepEqual(matrix, [...PUBLISHED].sort())
})

// ── criterion 5: nothing is dropped on the way through ───────────────────
run('every export of index.js is exported by loader.js, and is the same value', () => {
  const index = require('./index')
  const loader = require('./loader')
  const a = Object.keys(index).sort()
  const b = Object.keys(loader).sort()
  assert.ok(a.length > 0)
  assert.deepEqual(b, a)
  for (const k of a) assert.strictEqual(loader[k], index[k], `${k} differs`)
})

run('every function declared in index.d.ts is exported by loader.js', () => {
  const dts = fs.readFileSync(path.join(__dirname, 'index.d.ts'), 'utf8')
  const declared = [...dts.matchAll(/^export declare function (\w+)/gm)].map((m) => m[1])
  assert.ok(declared.length >= 10, `only found ${declared.length}`)
  const loader = require('./loader')
  for (const name of declared) assert.equal(typeof loader[name], 'function', `${name} is not exported`)
})

run('the loader is a plain re-export: require("mdka") is index.js', () => {
  assert.strictEqual(require('./loader'), require('./index'))
})

run('ES module named imports still work through the entry point', () => {
  const url = pathToFileURL(path.join(__dirname, 'loader.js')).href
  const r = spawnSync(
    process.execPath,
    ['--input-type=module', '-e', `import { htmlToMarkdown, version } from ${JSON.stringify(url)}; process.stdout.write(htmlToMarkdown('<p>hi</p>').trim() + ' ' + typeof version)`],
    { encoding: 'utf8' },
  )
  assert.equal(r.status, 0, r.stderr)
  assert.equal(r.stdout, 'hi function')
})

// ── package.json ─────────────────────────────────────────────────────────
run('package.json: main is loader.js, it is published, and types still resolve', () => {
  const pkg = require('./package.json')
  assert.equal(pkg.main, 'loader.js')
  assert.ok(pkg.files.includes('loader.js'))
  assert.ok(pkg.files.includes('index.js'))
  assert.ok(fs.existsSync(path.join(__dirname, pkg.types)), `types ${pkg.types} missing`)
  assert.equal(require.resolve('.'), path.join(__dirname, 'loader.js'))
})

// ── criterion 6: the existing suite, through the wrapper ─────────────────
run('test.js passes unchanged when its require("./index") is served by loader.js', () => {
  const preload = `
    const Module = require('module')
    const path = require('path')
    const index = path.join(${JSON.stringify(__dirname)}, 'index.js')
    const loader = path.join(${JSON.stringify(__dirname)}, 'loader.js')
    const resolve = Module._resolveFilename
    Module._resolveFilename = function (request, parent, ...rest) {
      const file = resolve.call(this, request, parent, ...rest)
      // everything but loader.js itself gets the entry point in place of index.js
      return file === index && !(parent && parent.filename === loader) ? loader : file
    }
    require(${JSON.stringify(path.join(__dirname, 'test.js'))})
  `
  const r = spawnSync(process.execPath, ['-e', preload], { encoding: 'utf8', cwd: __dirname })
  assert.equal(r.status, 0, `${r.stdout}\n${r.stderr}`)
  assert.match(r.stdout, /failed: 0/)
})

console.log(`\n${'─'.repeat(40)}`)
console.log(`  passed: ${passed}  failed: ${failed}`)
if (failed > 0) process.exit(1)
