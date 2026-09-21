# Implementation handoff — RFC 033 · Published docs: the source is what the reader gets

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/done/033-published-docs-source-is-what-the-reader-gets.md`](../../done/033-published-docs-source-is-what-the-reader-gets.md) — **Accepted 2026-09-16 by the owner**, with the handoff-time amendment at its end
**Milestone.** M3. **Order: 030 ✅ → 031 ✅ → 032 → 033 → 025 → 024 → 028.**
**Size.** Small. Five doc blocks, two README links, two gate rules, one version pin.

---

## 0. Preconditions — met

~~Do not start until RFC 032 is approved.~~ **RFC 032 approved 2026-09-16**
(`.git-exclude/reviewed/032-gates-report-everything/README.md`).
**This handoff is now an instruction to start.**

RFC 032 changed `check-docs-examples.py` substantially (+328/−109). **Every line
number below is from `d73b1be`, before that change — re-derive all of them.**

## 1. Why

RFC 031 fixed five non-compiling examples with mdBook hidden lines. The deployed
mdBook 0.5.4 copy button returns `innerText`, which omits `display: none`
elements — so **Copy yields the example without its hidden fallible `main`**, and
pasted code fails with E0277. Details and the deployed JS: RFC 033 §2.2.

The design: what is shown is exactly what compiles, and the gate enforces it.

## 2. Part A — hidden lines

### A.1 The five blocks — re-derived at `d73b1be`

Every `rust` fence under `docs/src/` and `README.md`, scanned with mdBook's
hidden-line rules:

| Block | Info string | Hidden lines |
|---|---|---|
| `docs/src/getting-started/usage-rust.md:68` | `rust,no_run` | 3 |
| `docs/src/getting-started/usage-rust.md:84` | `rust,no_run` | 3 |
| `docs/src/api/core.md:64` | `rust,no_run` | 3 |
| `docs/src/api/core.md:109` | `rust,no_run` | 3 |
| `docs/src/api/errors.md:40` | `rust,no_run` | 3 |

No other `rust` fence has any. **Re-run the scan when you start**, and work to
what it finds, not to this table.

### A.2 Rewrite each as a visible program

```rust,no_run
use mdka::html_file_to_markdown;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = html_file_to_markdown("page.html", None::<&str>)?;
    println!("{} → {}", result.src.display(), result.dest.display());
    Ok(())
}
```

- `use` lines **outside** `main`, at the top — that is what a reader would write.
- Keep `no_run`: these read files that do not exist in a sandbox.
- **Do not change what the example demonstrates.** Only the scaffolding becomes
  visible. Diff each block and say so.

### A.3 Gate rule — reject hidden lines in **every** `rust` fence

Note the amendment at the end of RFC 033: **all** `rust` fences, not only runnable
ones. Fragments, `ignore` and `compile_fail` included.

That means the check runs **before** the runnable filter, over every block whose
language is `rust` — including blocks `SKIP_MARKERS` would skip. Put it where it
cannot be bypassed by a marker.

Detection must use **the same rules as `mdbook_rust_source()`** — ideally by
calling a shared helper, so the two can never disagree:

| Line, after leading whitespace | Hidden? |
|---|---|
| `# ` + anything | yes |
| exactly `#` | yes |
| `##` + anything | yes — an escape is still hidden-line syntax |
| `#!` or `#[` | **no** — attribute |

Message: file, line of the offending line (not only the fence start), and one
sentence — *make the code visible; mdBook's copy button omits hidden lines.*

Keep `mdbook_rust_source()`. After this change it should be a no-op on every
passing block — say that in its docstring, and say why it is kept: it is the
definition the rejection rule shares.

### A.4 Prove it

- **Red:** a hidden line in (i) a runnable block, (ii) a `fragment` block, (iii) a
  `rust,ignore` block — each reported with file and line.
- **Green:** `#[derive(Debug)]` and `#![allow(unused)]` at line start — not
  flagged.
- **Invariant:** for every `rust` fence, `mdbook_rust_source(code) == code`.
  Assert it in the gate after the rejection, as a cheap guard that the two agree.

## 3. Part B — README fragment links

### B.1 Change

`README.md:39` and `:41` — both `(#conversion-modes)` →
`(https://nabbisen.github.io/mdka-rs/api/modes.html)`.

Fetch that URL before writing it (it returned 200 at `d73b1be`; the
extensionless form also works — use `.html`, as 031b did).

**Do not** remove or rename the README's own `## Conversion Modes` section.

### B.2 Gate rule

`check_readme_links()` currently **allows** `#` targets
(`check-docs-examples.py:126` at `d73b1be`). Change it to **reject** a
fragment-only target in `README.md`, with a message: *the README is rendered by
GitHub, crates.io, npm and PyPI, which rewrite heading ids differently; link to
the user guide instead.*

Leave the rest of the function's behaviour alone. It remains scoped to
`README.md` — `docs/src/` in-page anchors are rendered by mdBook alone and stay
allowed.

### B.3 Prove it

- **Red:** reintroduce `](#conversion-modes)` → reported with line.
- **Green:** a `docs/src/` page with an in-page anchor is unaffected.
- Restore, confirm clean.

## 4. Part C — pins: mdBook and TypeScript

`.github/workflows/docs.yaml:35`: `--vers "^0.5"` → `--vers "=0.5.4"`.

First **confirm 0.5.4 is what deployed today** — the live `book-*.js` hash
`609e4cb8` came from it; check the most recent Docs run's install log for the
resolved version rather than assuming. If CI resolved a different 0.5.x, pin that
one and say so.

In the gate's module docstring, next to the wrapping and hidden-line rules, state
the modelled version and that an mdBook upgrade must re-check both rules and the
copy-button behaviour (RFC 033 §2.2) before the pin moves.

### C.2 TypeScript — amendment from the RFC 032 review

`TYPESCRIPT = "typescript@5.9.3"` → **`"typescript@7.0.2"`** — `npm view
typescript dist-tags` shows it as `latest`.

1. Run `npx --yes --package typescript@7.0.2 tsc --init` in a scratch directory
   and **diff its compile-relevant options against `TSC_INIT_OPTIONS`**. The
   architect found them identical; confirm rather than trust. If they differ,
   take 7.0.2's.
2. Update the comment above `TSC_INIT_OPTIONS` so it names 7.0.2.
3. **Prove strictness is kept:** the pre-fix `usage-nodejs.md:166` example
   (value imports of `JsConversionOptions`, `ConvertResult`) must fail TS1484
   under the new pin; the current example must pass.
4. State the upgrade policy in the comment, in the same words as mdBook's:
   pin exactly, re-derive `tsc --init` when the pin moves.

## 5. CHANGELOG

`CHANGELOG.md:43–46`, in `[Unreleased]` → **Fixed**, currently says the five
examples *"now compile"* — written by the architect, true as displayed, false as
copied. Rewrite so it is true: the examples now show complete programs, and copy
as code that compiles. Mention the README links in the same section.

## 6. Out of scope

- `src/`. Any Python, JS or TS example.
- Any change to what an example demonstrates.
- mdBook upgrade.

## 7. Acceptance checklist

- [ ] §0 — line numbers re-derived against the post-032 script
- [ ] A.1 scan re-run at start; result stated
- [ ] A.2 every hidden-line block rewritten as a visible program; each diff scaffolding-only
- [ ] A.3 rejection over **every** `rust` fence, before skip markers apply; shared detection helper
- [ ] A.4 red ×3 (runnable, fragment, ignore) with file:line; attributes green; invariant asserted
- [ ] B.1 links replaced; URL fetched first
- [ ] B.2/B.3 fragment rejection red on reintroduction; `docs/src/` anchors unaffected
- [ ] C mdBook pinned to the version CI actually resolved; modelled version in the docstring
- [ ] C.2 TypeScript pinned to 7.0.2; `tsc --init` options re-derived; pre-fix example still fails TS1484
- [ ] §5 CHANGELOG corrected
- [ ] Docs gate runnable count unchanged; all six workflows green
- [ ] **Deployed-site check, no browser needed:** after merge, fetch each of the five pages and confirm `class="boring"` count is **0**

## 8. Report back

`.git-exclude/review-request/033-published-docs-source/README.md`, evidence under
`evidence/`, exit codes inside the files.
