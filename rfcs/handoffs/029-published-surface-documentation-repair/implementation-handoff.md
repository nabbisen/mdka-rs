# Developer Handoff — RFC 029 · Published-surface documentation repair

**Governing RFC.** [RFC 029](../../accepted/029-published-surface-documentation-repair.md)
**Source.** `2.2.2` consumer pass — RFC 027 Rule 1's first run. Disposition: `.git-exclude/reviewed/2.2.2-consumer-pass/README.md`; the raw pass is `.git-exclude/review-request/2.2.2-consumer-pass/README.md`, 26 findings.
**Milestone.** M2c → `2.2.3` (patch)
**Prepared.** 2026-09-16

---

## 0. Ready to start. No preconditions.

Independent of M3. **This ships before RFC 025/024/028**, because the README's
Node Quick Start does not parse and it renders on GitHub, crates.io, npm and
PyPI right now.

## 1. Order of work — the gate goes first

**Non-negotiable, same as RFC 020 and RFC 026.**

1. Extend the docs gate (§4). Commit alone. Push.
2. **Observe it failing** against the current README. Capture it.
3. Fix the documentation (§2, §3).
4. Observe the same gate pass.

The gate you are extending was built in the last milestone to catch broken
examples, and it could not see the most-read file in the project. Watching it go
red on that file is the proof it now can.

## 2. The P0 items

### 2.1 README Node Quick Start does not parse

```js
const { htmlToMarkdown, htmlToMarkdownWith } = require('mdka')
const md = htmlToMarkdown('<h1>Hello</h1>')
const md = await htmlToMarkdownWithAsync(html, { mode: 'minimal', dropInteractiveShell: true })
```

Four defects: `const md` twice (`SyntaxError`), `htmlToMarkdownWithAsync` used
but never imported, `html` undefined, top-level `await` in CommonJS.

**`docs/src/getting-started/usage-nodejs.md` gets all of this right.** Take its
wording rather than inventing new. It was corrected and executed under RFC 023.

### 2.2 Two documented binding options do not exist

`usage-python.md:46` says `preserve_unknown_attrs` and `drop_presentation_attrs`
are *"accepted but have no effect"*. Verified: neither exists in
`python/src/lib.rs`, and passing either raises `TypeError`. `JsConversionOptions`
has seven fields; neither is among them, so TypeScript gives `TS2353`.

**Three of the five deprecated options are reachable from the bindings. Two are
not.** Say exactly that on both pages. The current wording is worse than silence:
it tells a user their existing call still works when it raises.

## 3. The P1 items

| # | Fix |
|---|---|
| `installation.md` | *"On other platforms, run `npm run build`"* is impossible — the npm tarball has four files and no Rust source; `napi` is a devDependency. Say what is actually possible for musl, linux-arm64, macOS Intel and Windows ARM, or say plainly there is no fallback. |
| README CLI example | `mdka --mode minimal --drop-shell *.html` needs `-o` for multiple inputs. **`mdka --help` repeats the same broken example two lines below the rule forbidding it** — fix both, in `cli/src/main.rs`. |
| README paths | `![logo](/docs/src/assets/logo.png)` is root-absolute and 403s on npmjs.com and PyPI. Same for `./docs/` and `./CHANGELOG.md`, which are not in the published tarballs. |
| `--version` / unknown args | `mdka --version` → `error: IO error: No such file or directory`. Add `--version`; reject unknown `-`-prefixed arguments instead of treating them as paths. This is audit finding `A-17`, which was never scheduled — an architect omission, not a new defect. |
| README caveats | `api/modes.md` opens with *"Balanced, Strict and Preserve currently produce identical output"* and calls it the page's most important fact; the README sells five modes and "lossless archiving" without it. Same for tables, documented only in `api/elements.md`. **The honesty is already written — surface it or link pointedly to it.** |

## 4. The gate corrections

**4.1 · The gate must cover `README.md`.** `check-docs-examples.py` takes
`--root`, defaulting to `docs/src`. That is why §2.1 was invisible to it.

**4.2 · The gate must check keyword arguments, not only symbol resolution.**
§2.2 is a kwarg. The gate resolves `html_to_markdown_with` and stops, never
learning the argument is rejected. For Python, bind the documented call's kwargs
against the installed signature; for TypeScript, `tsc` already catches it if the
example is type-checked as written.

## 5. ⚠ Verify paths against a *published* artifact, not GitHub

§3's path fixes cannot be verified by looking at GitHub — that is the one
renderer where root-absolute paths work.

**Check the actual tarballs**: `npm pack mdka` and the PyPI sdist.

### ⚠ Corrected 2026-09-16 — "is the target in the package" is the wrong test

This first read *"confirm what the README references either exists inside them
or is an absolute URL."* **That produces a false pass on PyPI.** Verified: the
sdist **does** contain `CHANGELOG.md`, `ROADMAP.md` and `docs/` — so the targets
are present, and the links are still broken.

PyPI renders the README on its **project page**, where a relative path resolves
against `pypi.org`, not the tarball. npm does the same on the package page.

**The test is whether the path resolves where the README is rendered**, which
differs per registry — not whether the target is in the package. In practice:
`README.md` should carry no relative or root-absolute links at all.

Found by the implementer while following the instruction as written.

This is the same lesson as RFC 020's install gate and RFC 026's wheel gate:
check what the consumer receives.

## 6. Scope boundary, per RFC 027 Rule 2

**Derived from a search, not a file list** — that is Rule 2 as amended, and this
RFC exists because the previous two boundaries were not.

In scope: every file that renders to a user of a *published artifact* —
`README.md`, `docs/src/getting-started/`, and the CLI's own `--help`.

**Not in scope, with owners:**

- **The engine.** F-04 (link text losing spaces) → RFC 024, where it is now an
  explicit acceptance criterion. F-01/F-05/F-06 → RFC 024 and RFC 010. Tables →
  RFC 008.
- **F-23** — default `Balanced` emitting 1,190 `<a id>` anchors on a real
  Wikipedia page. A design question, shared with bekoedit's item 8. Not a defect,
  not yours.
- **F-24** — deep nesting is quadratic (300k depth, 255s). M4.
- `CHANGELOG.md`'s own content, benchmarks (RFC 012).

**Before you finish, run one search** for the class this RFC is about: a
documented call, flag or path that does not work. If you find one outside the
list above, report it — the list came from one reviewer's pass, not from
exhaustiveness.

## 7. Required verification

Per RFC 027 Rule 3, state for each whether it ran against the workspace tree or
an installed artifact.

1. **The gate failing against the pre-fix README** — captured output. Required.
2. The gate passing after.
3. Every README code block executed as written, output recorded.
4. Every README CLI example run as written, including the multi-file one.
5. `mdka --version` and an unknown flag, both shown.
6. `--help` diffed against `usage-cli.md` again — RFC 023 aligned them; do not
   let the example fix break that.
7. **Paths verified inside `npm pack` output and the PyPI sdist** — §5.
8. Binding pages: show that a documented option list matches the installed
   signature, for both Python and Node.
9. **Conversion output byte-identical**; test count unchanged; fmt, clippy,
   `mdbook build` clean.

## 8. Prohibited shortcuts

- Do not fix the README before the gate can see it.
- Do not verify paths on GitHub.
- Do not touch the engine.
- Do not invent Node wording — `usage-nodejs.md` is already correct and was
  executed under RFC 023.
- Do not let `--version` become a behaviour change beyond argument handling.

## 9. Known risks

| Risk | If it happens |
|---|---|
| The gate's kwarg check is harder than it looks for Node | TypeScript already rejects `TS2353` if the example is type-checked as written. If Python's is awkward, say so and propose what you can do. |
| Rejecting unknown `-` args breaks a real invocation | A leading-`-` filename is the edge case. Decide, test it, and say what you chose. |
| README caveats make the page defensive | Aim for one sentence and a link, not a disclaimer section. `api/modes.md`'s tone is the model. |
| The list in §3 is incomplete | Likely — §6's closing search is why. |

## 10. Acceptance checklist

- [ ] Gate covers `README.md`, **observed failing pre-fix**, output captured
- [ ] Gate checks keyword arguments, Python and Node
- [ ] Every README code block executes as written
- [ ] README CLI examples run, including multi-file; `--help` example fixed too
- [ ] Binding pages name exactly the options that exist, and say what the other two do
- [ ] `installation.md` describes a fallback that works, or says there is none
- [ ] README paths resolve **in the npm tarball and PyPI sdist**
- [ ] `mdka --version` works; unknown `-` args rejected with a usable message
- [ ] README carries the modes-identical and table caveats, or links pointedly
- [ ] `--help` still matches `usage-cli.md`
- [ ] Output byte-identical; count unchanged; fmt, clippy, `mdbook build` clean
- [ ] **`CHANGELOG.md` entry** covering the argument-handling change — added
      2026-09-16; this checklist omitted it

## 11. Escalate rather than decide

Stop and raise if: the gate cannot be made to fail on the pre-fix README; the
kwarg check needs something that looks like a framework; rejecting unknown args
would break a plausible real invocation; or your §6 search finds a defect class
this RFC does not cover.

## 12. Where this came from

A session with **no history of this project**, told to try to prove the
documentation wrong. It found in one pass what a 56-finding external audit, four
consumer-artifact gates and six documentation RFCs did not.

Worth knowing while you work: **the gate you are extending was built last
milestone to catch exactly this, and was pointed at a directory that excluded
the file.** If you notice a second surface no control covers, that is worth more
than any item in §3.
