# Implementation handoff — RFC 031 · Docs example gate must compile what mdBook publishes

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/accepted/031-docs-gate-must-model-mdbook.md`](../../accepted/031-docs-gate-must-model-mdbook.md) — **Accepted 2026-09-16 by the owner**
**Source.** 2.2.3 consumer pass — `.git-exclude/reviewed/2.2.3-consumer-pass/` (`report.md` is the performer's full report; `README.md` is my disposition)
**Milestone.** M3. **Order: 030 ✅ → 031 → 025 → 024 → 028.**
**Target.** Docs fixes publish on merge to `main` (`docs.yaml`). Only §6 (PyPI metadata) waits for `2.3.0`.

---

## 0. Preconditions

None. RFC 030 is approved. **This handoff is an instruction to start.**

## 1. Why

The docs example gate wraps a `?`-using Rust block in
`fn main() -> Result<…>`. mdBook, which publishes the page, wraps it in a plain
`fn main() {`. So the gate compiles a more forgiving program than the one behind
the **Run** button, and two examples on `usage-rust.md` fail with E0277 on the
published page while the gate is green.

I confirmed this from the built book, not from the report. The published code
for "Converting a Single File" is literally:

```rust
#![allow(unused)]
fn main() {
use mdka::html_file_to_markdown;
let result = html_file_to_markdown("page.html", None::<&str>)?;
...
}
```

## 2. Order of work — gate first, and see it red

**Do not fix the docs first.** If you do, you never see the corrected gate catch
the defect it exists for, and RFC 026 says a gate not seen failing is not
trusted.

1. **§3** — fix the Rust wrapping. Run the gate. **It must go red on the two
   `usage-rust.md` blocks** (`Converting a Single File`, `Bulk Parallel
   Conversion`). Capture it. If it stays green, stop — the wrapper is still
   wrong.
2. **§4** — add hidden-line handling. Prove it row by row.
3. **§5** — fix the documentation. Gate green.

## 3. Rust wrapping — mirror mdBook exactly

`check-docs-examples.py`, `check_rust`, around line 160. For a block without
`fn main`:

```
#![allow(unused)]
fn main() {
<block>
}
```

- **Remove the `if "?" in code` branch entirely.** No `Result`.
- **Replace the "Mirror rustdoc" comment** with one naming mdBook as the model,
  and saying why the gate must not be more permissive than the renderer. That
  comment is where the defect came from; the replacement is what stops it
  coming back.
- The file currently prepends `#![allow(unused, deprecated)]`. Keep `deprecated`
  allowed — examples reference deprecated fields on purpose — but make sure the
  body wrapping is otherwise mdBook's.

## 4. ⚠ Hidden lines — the part that will go wrong

The idiomatic fix for D1 uses mdBook's hidden lines:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use mdka::html_file_to_markdown;
let result = html_file_to_markdown("page.html", None::<&str>)?;
# Ok(())
# }
```

**The gate has no hidden-line handling today** — I checked; nothing strips a
`#` marker, and no page under `docs/src/` uses one yet. Without it, a correct
doc fix breaks the gate: it sees `fn main` inside `# fn main`, skips wrapping,
and hands `# fn main() -> …` to rustc, which is not Rust.

Implement mdBook's rule. Match on the line **after leading whitespace**:

| Line starts with | Do |
|---|---|
| `# ` | strip the `# `, compile the rest |
| exactly `#` | compile an empty line |
| `##` | compile with **one** `#` removed (escape) |
| `#!` or `#[` | **leave untouched** — these are attributes |

**Strip before the `fn main` detection**, so detection sees what mdBook
compiles.

**The trap is row four.** `startswith("#")` strips `#[derive(Debug)]` and
`#![allow(...)]` from every example and the gate will fail in confusing ways —
or worse, pass because an attribute that mattered vanished.

### 4.1 Prove each row

Scratch fixtures (not committed pages), one per row, each run through the gate:

- `# use std::fmt;` then code using `fmt` → compiles (hidden line honoured)
- a bare `#` line → compiles
- `## ` line inside a raw string literal, e.g. `let s = r"
## heading
";` → compiles, and the string contains `# heading`
- `#[derive(Debug)] struct S;` then `println!("{:?}", S);` → compiles (**attribute preserved** — this fails if you stripped it)
- `#![allow(unused)]` at the top of a block → compiles

Capture the output. Delete the fixtures.

## 5. Documentation fixes

Every finding below is **re-derived by me against the tree**, with locations.

| ID | Location | Fix |
|---|---|---|
| **D1** | `docs/src/getting-started/usage-rust.md:68–77`, `:81–95` | Hidden fallible `main` per §4. Both read files that won't exist in the playground — **decide `no_run` or not, and say which in the review request.** The gate only builds, so `no_run` does not weaken it. |
| D2 | `docs/src/getting-started/usage-nodejs.md:30–37` | `require()` + top-level `await` → `ERR_AMBIGUOUS_MODULE_SYNTAX`; `html` and `pages` undefined. Every other async block on the page wraps in `async function main()` — match that, and define the inputs. |
| D3 | `docs/src/api/elements.md:53–59` | Page says `H1H2ab`. The markup **as printed** (newline before `<tbody>`) gives **`H1H2 ab`** — I reproduced it. Fix the stated output or put the markup on one line. |
| D4 | `docs/src/api/elements.md:49` | Remove `A-13` — an internal audit ID, unlinked. |
| D5 | `docs/src/api/elements.md:45–48` | RFC 008 and RFC 009 **do not exist** as files. Point at `ROADMAP.md`. |
| **D7** | `docs/src/api/options.md:~127–135` ("simply do nothing"); `docs/src/getting-started/usage-python.md:47–48` ("kept so existing calls keep working"); `usage-nodejs.md` | See §5.1. |
| D8 | `docs/src/getting-started/installation.md` | Add the prebuilt-binary path. **Link to the README's platform table; do not copy it**, or the two drift. |

### 5.1 D7

All three bindings warn when a deprecated field is set (`node/src/lib.rs:38`
emits `DeprecationWarning`; the Python binding does the same). No page says so,
and two pages promise the opposite. Under `python -W error`, pytest
`filterwarnings = error`, or Node treating warnings as errors, the
"kept so existing calls keep working" calls **fail**.

On each of the three pages, state: that the warning is emitted; that
warnings-as-errors turns it into a failure; how to suppress it narrowly while
migrating. **Do not remove or soften the warning.**

**Execute the suppression snippet you document**, under `-W error` for Python.
A documented workaround nobody ran is how this defect class started.

### 5.2 D6 — look before changing

`#conversion-modes` in the README resolves to `id="user-content-conversion-modes"`
in crates.io's served HTML. That prefix is normal sanitisation, usually rewritten
client-side. **Click it in a real browser on crates.io, npmjs.com and PyPI.**
Record the result per registry. Change nothing unless one actually fails. If you
have no browser, say so and leave it open — do not change anchors on the served
HTML alone.

## 6. D9 — PyPI metadata (lands with `2.3.0`)

`python/pyproject.toml` has a `[project]` table (`:5`) and no `[project.urls]`,
so PyPI's sidebar is empty. Add Homepage (docs site), Source, Documentation,
Changelog.

**Verify against a built artifact**, not the TOML: build the wheel, read
`METADATA`, confirm the `Project-URL:` lines. Per RFC 027 Rule 3, label it.

Crate `homepage` fields — your judgement; say which way and why.

## 7. The JS path substitutes too — answer this, don't fix it blind

Pre-derived so you don't have to rediscover it:

- D2's block is fenced ` ```js `, so it **is** in the gate's runnable set.
- `check_js` runs **`node --check`** on it. That is a **syntax** check. The
  block failed for a module-format reason at load time and for undefined
  variables at run time — neither is a syntax error — so it passed.

That is the same pattern as §3: a convenient stand-in (parse) for what the
consumer does (run). RFC 031 §6's rule applies.

**I am not prescribing execution.** Running examples needs inputs and a built
binding, and some examples legitimately depend on files. In the review request:

1. Confirm why `--check` passed D2 — by running it, not by reasoning.
2. Propose what the JS path should do, and **state explicitly what it would
   still let through that a consumer would hit.** That sentence is RFC 031 §6's
   operative requirement; I want it written for this path.
3. Implement it if it is small. If not, say so and it becomes its own RFC.

Same question, briefly, for Python and TS: does each path check the program the
consumer runs? One line each is enough if the answer is yes.

## 8. Out of scope

- D10, D11 — no action.
- The other three gates.
- `src/`. **No behaviour change.** If a doc fix seems to need one, stop and raise.
- The consumer-pass mechanics in the release checklist.

## 9. Acceptance checklist

- [ ] §3 Rust wrapping mirrors mdBook; `?` branch removed; comment replaced
- [ ] §2.1 **gate observed red on both `usage-rust.md` blocks** before the doc fix — captured
- [ ] §4 hidden-line rule, all four rows; stripping precedes `fn main` detection
- [ ] §4.1 each row demonstrated, attribute preservation included — captured
- [ ] §5 D1, D2, D3, D4, D5, D7, D8 fixed; gate green
- [ ] §5.1 documented suppression executed under warnings-as-errors — captured
- [ ] §5.2 D6 checked per registry in a browser, or explicitly left open
- [ ] §6 D9 `[project.urls]` added, verified from a built wheel's `METADATA`
- [ ] §7 answered for JS, with the "what it still lets through" sentence; one line each for Python and TS
- [ ] `mdbook build` clean; tests, fmt, clippy unchanged

## 10. Report back

`.git-exclude/review-request/031-docs-gate-must-model-mdbook/README.md`, with
captures under `evidence/` — **including exit codes in the capture files
themselves** (`; echo "EXIT: $?"`), not only in the prose. That was the one gap
in RFC 030's otherwise excellent evidence.
