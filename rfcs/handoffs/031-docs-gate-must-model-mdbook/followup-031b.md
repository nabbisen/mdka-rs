# Follow-up handoff — RFC 031 · slice `031b`

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/accepted/031-docs-gate-must-model-mdbook.md`](../../accepted/031-docs-gate-must-model-mdbook.md) — scope addendum recorded there
**Review this follows.** `.git-exclude/reviewed/031-docs-gate-must-model-mdbook/README.md` (§4, §5)
**Size.** Small. One config line, string literals in three files, two test assertions, three comments.
**Closes.** RFC 031, except D6 (owner, needs a browser).

---

## 0. Preconditions

None. Slices 031 (`e1ab3e4`…`41b6c0e`) are approved.
**This handoff is an instruction to start.**

## 1. Turn off the Run button

**Finding (new, from review).** The Rust playground offers a fixed crate set.
`https://play.rust-lang.org/meta/crates` lists 488; **`mdka` is not one**. 20 of
the 32 `rust` fences under `docs/src/` import `mdka`, so their **Run** button can
only produce `unresolved import` — including every example that is correct.

**Change.** `docs/book.toml`:

```toml
[output.html.playground]
runnable = false
```

**What I already verified** in a throwaway copy with mdBook v0.5.4:

| | default | `runnable = false` |
|---|---|---|
| `pre class="playground"` on `usage-rust.html` | 6 | 0 |
| `class="boring"` hidden-line spans | present | 6, still present |
| raw `# fn main` visible in HTML | 0 | 0 |

I checked the second and third rows because mdBook *could* have tied hidden-line
hiding to the playground class, which would have exposed every `# fn main` line
added in 031. Reading `book-*.js`, it applies `hide-boring` to every
`code.hljs` block, so it does not.

**What I did not verify — you must:** that on the **deployed** site after merge,
(a) no Run button appears on `getting-started/usage-rust`, and (b) the hidden
`# fn main` lines are still hidden. Fetch the deployed HTML and check (a) by
the absence of `class="playground"`; for (b), a browser if you have one — if not,
say so, and state that (b) rests on the static reading above.

**The docs gate must not change.** It compiles against the real crate and is
independent of `runnable`. Confirm it is still green and still reports the same
runnable count.

## 2. Remove `RFC 005` from user-facing warning text

**Scope extension to RFC 031, authorized by the architect and recorded on the
RFC** — same defect class as D4/D5: an internal ID in a string that, in Node and
Python, is the only thing the consumer receives.

**Locations — re-derived by me:**

| File | Line | Current |
|---|---|---|
| `node/src/lib.rs` | 34 | ``"mdka: `{field}` has no effect and is deprecated (see RFC 005). \`` |
| `python/src/lib.rs` | 18 | same |
| `src/options.rs` | 92, 98, 104, 110, 119, 213 | `note = "no effect: Markdown has no attribute syntax. See RFC 005."` |

Replace the RFC reference with the options page:
`https://nabbisen.github.io/mdka-rs/api/options.html` (confirm the deployed URL
form — `.html` or not — by fetching it before you write it into six places).

**Only the `see RFC 005` part changes.** Comments that mention RFC 005 are for
maintainers and stay.

### 2.1 ⚠ Do not break the suppressions you just published

Both documented suppressions from D7 **match on the message prefix**:

- Python — `usage-python.md`: `message=r"mdka: \`preserve_"`
- Node — `usage-nodejs.md:141`: `message?.startsWith('mdka: \`')`

So the message **must still begin `mdka: \`<field>\``**. Change the tail only.

**Nothing currently tests that prefix.** `node/test.js:285` asserts only
`/preserveClasses/`. So a future rewording could silently break the published
workaround. Add:

- `node/test.js` — assert the warning message **starts with** `` mdka: `preserveClasses` ``
- the Python binding's tests — the equivalent assertion on `preserve_classes`, if
  a warning test exists; if none exists, add one

Then the prefix is a tested contract rather than a coincidence.

### 2.2 Re-execute D7

Re-run the three `step5-D7-*` captures against the rebuilt artifacts — the
documented snippets, verbatim, under warnings-as-errors, with the narrowness
controls. Same EXIT results expected. This is the proof §2.1 held.

### 2.3 Compatibility

String-only. Not an API change; `#[deprecated(note)]` text is not semver-relevant.
Record it under `[Unreleased]` → **Changed** in `CHANGELOG.md`, since it is
visible to users. `node/index.d.ts` — check whether the note text is emitted
into it; if regeneration changes it, regenerate in its own commit as before.

## 3. State the two substitutions — comments only

RFC 031 §6: a gate that substitutes something more convenient must say what,
and why it cannot pass what the real artifact would fail.

### 3.1 `check_js` — local binding, not the published package

The gate copies a locally built `.node` beside `index.js`. Consumers resolve a
platform package through `optionalDependencies`. A resolution defect (RFC 020's
class) passes here. **It is covered by the `npm install gate`** — say so in the
`check_js` docstring, naming that gate.

Add the same point to the "what it still lets through" list, so the list is
complete.

### 3.2 `check_rust` — `#![allow(unused, deprecated)]`

mdBook prepends `#![allow(unused)]`. The extra `deprecated` never changes
pass/fail, but it means **the gate cannot verify any claim about deprecation
warnings** — for example, `api/options.md`'s *"This builds under
`-D warnings`"*: delete the snippet's narrow `#[allow]` and the gate stays green.

Say so in the comment where the prefix is written, and name where such claims
are verified instead (the `step5` executions, for now).

Do **not** remove `deprecated` from the prefix to "fix" this — examples that set
deprecated fields on purpose would then warn, which is harmless, but it would
not make the gate check `-D warnings` either. The comment is the change.

## 4. Out of scope

- D6 — owner.
- Python/TS example execution, and `--no-fail-fast` in `ci.yaml` — RFC 032, proposed.
- Any other `src/` change.

## 5. Acceptance checklist

- [ ] `runnable = false`; deployed page shows no playground blocks; hidden lines verified or the limit stated
- [ ] Docs gate green, same runnable count as `41b6c0e`
- [ ] `RFC 005` gone from all eight user-facing strings; URL fetched before use
- [ ] Message prefix `mdka: \`<field>\`` unchanged; **tested** in Node and Python
- [ ] D7 captures re-executed, same EXIT codes
- [ ] CHANGELOG `[Unreleased]` entry; `index.d.ts` checked
- [ ] Both substitution comments written
- [ ] Tests, fmt, clippy clean; test count stated (it will change — say by how much)

## 6. Report back

`.git-exclude/review-request/031b-run-button-and-warning-text/README.md`,
evidence under `evidence/` with exit codes inside the files, as in 031.
