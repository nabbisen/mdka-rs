# RFC 031 — Docs example gate must compile what mdBook publishes

**Status.** Accepted (2026-09-16, owner)
**Author.** Architect
**Created.** 2026-09-16
**Milestone.** M3 — control repair, **before the engine work**, alongside RFC 030
**Repairs.** RFC 026 §4.3 (the docs example gate)
**Source.** 2.2.3 consumer pass — `.git-exclude/reviewed/2.2.3-consumer-pass/`
**Related.** RFC 020 and RFC 029 §5 — the two earlier instances of the pattern in §6

---

## 1. Summary

The docs example gate compiles Rust examples differently from mdBook, which is
what publishes them. The gate is **more permissive** than the renderer, so it
passes code that fails on the published page. The 2.2.3 consumer pass found two
such blocks behind a **Run** button.

Make the gate compile exactly what mdBook compiles. Then fix what it reports,
plus seven further documentation findings from the same pass.

## 2. The defect

`check-docs-examples.py:162`, for a Rust block with no `fn main`:

```python
# Mirror rustdoc: an example using `?` is written as if inside a
# fallible function, so wrap it in one ...
if "?" in code:
    code = "fn main() -> Result<(), Box<dyn std::error::Error>> {\n...Ok(())\n}"
else:
    code = "fn main() {\n...\n}"
```

**mdBook does not do this.** Built from `docs/` and read from the published
HTML, this is the exact code behind the Run button on `usage-rust`:

```rust
#![allow(unused)]
fn main() {
use mdka::html_file_to_markdown;
let result = html_file_to_markdown("page.html", None::<&str>)?;   // E0277
}
```

Plain `fn main()`, always. `?` does not compile inside it.

**The comment names the wrong renderer.** These pages are published by mdBook,
not rustdoc. The gate modelled a renderer we do not use, and therefore checked a
more forgiving program than the one we ship.

## 3. The gate fix

### 3.1 Wrap exactly as mdBook does

For a `rust` block without `fn main`: prepend `#![allow(unused)]`, wrap in plain
`fn main() {` … `}`. **No `?`-sensitivity. No `Result`.** A block using `?`
without its own fallible main must fail, because it fails on the page.

### 3.2 ⚠ Honour mdBook's hidden lines — or the fix breaks the gate

The idiomatic repair for D1 is mdBook's hidden-line syntax, which keeps the page
readable while making the code compile:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use mdka::html_file_to_markdown;
let result = html_file_to_markdown("page.html", None::<&str>)?;
# Ok(())
# }
```

mdBook **compiles** `#`-prefixed lines and **hides** them on the page.

**The gate currently has no notion of hidden lines.** I checked:
`check-docs-examples.py` never strips a `#` marker, and no page under
`docs/src/` uses one yet. So today it is latent — but the moment D1 is fixed
the idiomatic way, the gate will:

1. see `fn main` in the raw text (inside `# fn main`) and **skip wrapping**, then
2. hand `# fn main() -> ...` to rustc, which is **not valid Rust**, and fail.

A correct documentation fix would turn the gate red. Someone would then "fix"
the gate by loosening it again. **This is the step that makes §3.1 safe to
ship.**

The rule to implement is mdBook's (shared with rustdoc):

| Line (after leading whitespace) | Treatment |
|---|---|
| `# ` followed by anything | Hidden: **strip `# `**, compile the rest |
| exactly `#` | Hidden: compile as an empty line |
| `##…` | Escape: compile as a literal `#…` (drop one `#`) |
| `#!…` or `#[…` | **Not hidden.** Real Rust — attributes. Compile unchanged. |

Strip hidden markers **before** the `fn main` detection, so detection sees the
program mdBook compiles, not the Markdown source.

The `#!`/`#[` row is the one that will be got wrong. A naive `startswith("#")`
strips every attribute in every example.

### 3.3 Prove the gate, both directions

Per RFC 026, a gate is not trusted until seen doing its job:

- **Red for the real defect.** With §3.1 in place and D1 unfixed, the gate
  **fails on `usage-rust.md`'s two `?` blocks** with E0277. Capture the output.
  If it does not fail on them, §3.1 is wrong and nothing downstream counts.
- **Green for the correct fix.** After D1 is fixed with hidden lines, green.
- **Hidden-line table, each row.** A scratch block per row of §3.2's table,
  including an attribute (`#[derive(Debug)]`) that must survive and a `##` escape.
  Show each compiles as mdBook would compile it.

## 4. Documentation fixes

From the consumer pass. `docs.yaml` deploys on push to `main`, so **these reach
users when they land — no release needed.**

| ID | Page | Fix |
|---|---|---|
| **D1** | `getting-started/usage-rust.md` — "Converting a Single File", "Bulk Parallel Conversion" | Hidden fallible `main` per §3.2. These read files that will not exist in the playground, so also consider `no_run` — **your call, stated in the review request.** |
| D2 | `getting-started/usage-nodejs.md` — "Async Conversion" | Mixes `require()` with top-level `await` → `ERR_AMBIGUOUS_MODULE_SYNTAX`; also uses undefined `html`, `pages`. Match the page's own pattern: every other async block wraps in `async function main()`. |
| D3 | `api/elements.md:59` | Says `H1H2ab`; the markup **as printed** gives `H1H2 ab` — the page's own newline before `<tbody>` becomes a space. Correct the stated output **or** put the markup on one line. The example exists to show degradation; its output must be exact. |
| D4 | `api/elements.md:49` | `A-13` is an internal audit ID, unlinked, meaningless to a consumer. Remove. |
| D5 | `api/elements.md:45–48` | Cites RFC 008 / 009, **which do not exist**. Point at `ROADMAP.md` instead. (`api/options` links RFC 005 correctly — that one exists.) |
| **D7** | `api/options.md`, `getting-started/usage-python.md`, `getting-started/usage-nodejs.md` | See §4.1. |
| D8 | `getting-started/installation.md` | Omits the prebuilt-binary path the README leads with. Add it. **Do not copy the README's platform table** — link to it, or the two will drift. |

### 4.1 D7 — the promise is wrong, not merely incomplete

`api/options`: the dead fields *"simply do nothing."* Python page: *"They are
kept so existing calls keep working."*

All three bindings emit a deprecation warning, and **no page says so.** Under
`python -W error`, pytest `filterwarnings = error`, or Node with warnings as
errors, **existing calls stop working** — the exact opposite of the documented
promise, in the configuration careful consumers run.

Document: that the warning is emitted; that `-W error` and equivalents turn it
into a failure; and how to suppress it narrowly for anyone who cannot remove the
field yet. **Keep the warning.** It is correct; the documentation is not.

### 4.2 D6 — investigate before touching anything

`#conversion-modes` resolves to `id="user-content-conversion-modes"` in the HTML
crates.io serves. This prefixing is normal sanitisation and is usually rewritten
client-side. **Check the link in a real browser on crates.io, npm and PyPI
first.** If it works everywhere, record that and change nothing. Changing
anchors on a hunch risks breaking the renderers where they currently work.

## 5. Packaging metadata — rides `2.3.0`

**D9.** `python/pyproject.toml` sets no `project_urls`, so PyPI's sidebar is
empty. Add Homepage (the docs site), Source (GitHub), Documentation and
Changelog. This ships in wheel metadata, so it takes effect at the next release,
not on merge.

While there, consider `homepage` in the crate manifests pointing at the docs
site — crates.io currently links docs.rs only. **Your judgement; say which.**

## 6. The rule this RFC records

Third instance of one pattern:

| Where | The gate inspected | The consumer met |
|---|---|---|
| RFC 020 | the locally packed tarball | the package installed from npm |
| RFC 029 §5 | "does the target exist in the tarball" | the link as rendered on PyPI |
| **RFC 031** | code wrapped in a `Result`-returning main | code wrapped in plain `fn main()` by mdBook |

Each was a convenient stand-in that happened to be more forgiving than the real
artifact. Each produced a false pass. Each was approved by me.

**Rule: a gate models the artifact the consumer meets, produced the way it is
actually produced. Where a gate substitutes anything more convenient, it must
state what it substitutes and why that substitution cannot pass something the
real artifact would fail.**

The second sentence is the operative one. "Mirror rustdoc" was a substitution
nobody was obliged to justify, and it could not have been justified.

## 7. Out of scope

- **D10** — `mdka-cli` shows the library README on crates.io. Shared-README
  consequence; no action.
- **D11** — one trailing newline, npm vs PyPI. No action.
- Python and JS/TS gate paths. Nothing in the pass indicates they mis-model
  their renderers. **But check the analogous question for each** — does the JS
  check run examples the way Node would load them? D2 is a module-format
  failure; if the gate did not catch it, say why.

## 8. Acceptance criteria

- [ ] Gate wraps Rust exactly as mdBook does — plain `fn main()`, `#![allow(unused)]`, no `?`-sensitivity
- [ ] Gate honours all four rows of §3.2's hidden-line table, **attributes preserved**
- [ ] Hidden markers stripped **before** `fn main` detection
- [ ] The misleading "Mirror rustdoc" comment replaced with one naming mdBook as the model
- [ ] §3.3: gate **observed red** on D1's two blocks before the doc fix, output captured
- [ ] §3.3: gate **observed green** after, hidden-line table demonstrated row by row
- [ ] D1, D2, D3, D4, D5, D7, D8 fixed
- [ ] D6 checked in a browser on all three registries; outcome recorded either way
- [ ] D9 `project_urls` added
- [ ] §7: stated whether the JS path should have caught D2, and why it did not
