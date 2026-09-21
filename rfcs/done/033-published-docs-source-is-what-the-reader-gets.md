# RFC 033 — Published docs: the source is what the reader gets

**Status.** Implemented (2.3.0)
**Author.** Architect
**Created.** 2026-09-16
**Milestone.** M3 — after RFC 032, before RFC 025
**Supersedes.** The two manual browser checks requested of the owner in the RFC 031 and 031b reviews (D6, and hidden-line display)
**Related.** RFC 031 §6 (a gate models the artifact the consumer meets)

---

## 1. Summary

I asked the owner for two manual browser checks. **Neither was the safe design.**
Each would have verified one page, once, in one browser, and protected nothing
afterwards. One of them would have **passed while a real defect remained**.

Both checks exist because a published page's behaviour depends on JavaScript we
do not control and cannot observe from CI. The robust fix is to **remove the
dependency**, then enforce its absence mechanically:

1. **No hidden lines in Rust examples.** What a reader sees — and copies — is
   exactly what compiles.
2. **No fragment-only links in `README.md`.** Link to the user guide, which we
   build and gate, instead of to anchors that three registries rewrite differently.
3. **Pin mdBook exactly**, so the renderer the gate models cannot change
   underneath it.

No owner action is needed, now or at future releases.

## 2. Hidden lines — the defect the browser check would have missed

### 2.1 What RFC 031 did

It fixed five non-compiling Rust examples with mdBook hidden lines:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let result = html_file_to_markdown("page.html", None::<&str>)?;
# Ok(())
# }
```

The gate compiles all of it; the page shows only the middle line. I recommended
this in the RFC 031 handoff as "the idiomatic repair".

### 2.2 What the copy button gives the reader — verified on the deployed site

From the **deployed** `book-609e4cb8.js` (mdBook 0.5.4):

```js
const clipboardSnippets = new ClipboardJS('.clip-button', {
    text: function(trigger) {
        const playground = trigger.closest('pre');
        return playground_text(playground, false);   // hidden = false
    },
});

function playground_text(playground, hidden = true) {
    ...
    } else if (hidden) {
        return code_block.textContent;
    } else {
        return code_block.innerText;                  // ← copy uses this
    }
}
```

`innerText` excludes elements that are not rendered. Hidden lines are
`display: none` (`.hide-boring .boring`). **So Copy yields the snippet without
the fallible `main`.** Pasted into `fn main()`, it fails with exactly the E0277
that RFC 031 set out to fix.

**D1 is fixed for the page and for the gate, and not for copy-paste** — the
single most common thing a reader does with an example.

**The browser check I asked for** — "hidden lines don't show, and the eye button
appears" — **would have passed.** It verified the appearance, and the appearance
was the part that was fine.

*Evidence level:* the JS is read from the deployed asset; the `innerText`
exclusion is specified behaviour. Not observed in a browser — and §2.4 makes that
unnecessary.

### 2.3 Why hidden lines cannot be made safe here

Every reader-facing channel sees something different from what compiles: the
rendered page, the copy button, a reader retyping, a search snippet. Each is
governed by mdBook's JavaScript and CSS. The gate cannot model all of them, and a
future mdBook can change any of them.

### 2.4 Design

**The visible code is the whole program.** In the five blocks, make the fallible
`main` visible:

```rust,no_run
use mdka::html_file_to_markdown;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = html_file_to_markdown("page.html", None::<&str>)?;
    println!("{} → {}", result.src.display(), result.dest.display());
    Ok(())
}
```

Three extra lines per example. In exchange, what is shown, copied, retyped and
compiled are the same text, in every browser and every mdBook version.

**Enforce it.** The docs gate **rejects any mdBook hidden line** in a runnable
`rust` block, naming the file and line and saying to make the code visible.
Detection reuses RFC 031's `mdbook_rust_source()` rules exactly — `# `, bare `#`
and `##` are hidden-line syntax; `#[` and `#!` are attributes and **must not** be
flagged. The stripping function stays: it is still how the gate knows precisely
what mdBook would treat as hidden.

Blocks marked `fragment` are exempt as before — they are labelled incomplete.

## 3. README fragment links — D6

### 3.1 The dependency

`README.md:39` and `:41` link to `(#conversion-modes)`. The README is rendered by
**four** independent renderers: GitHub, crates.io, npmjs.com and PyPI.

- **crates.io**, served HTML re-fetched today: `href="#conversion-modes"` twice,
  but the heading is `id="user-content-conversion-modes"`. It works only if
  crates.io's client JavaScript rewrites it.
- **PyPI** returns a JavaScript challenge to automated requests — its rendering
  **cannot be observed from CI at all**.
- **npm** — the rendered page is not available from the registry API.

Whether the link works is decided by three third parties' sanitisers and scripts,
which can change without notice, and **two of which we cannot even observe
automatically**. A one-time click would describe one day.

### 3.2 Design

Replace both with the user guide's page, which we build, deploy and gate:

```markdown
Five [conversion modes](https://nabbisen.github.io/mdka-rs/api/modes.html) ...
— see [Conversion Modes](https://nabbisen.github.io/mdka-rs/api/modes.html).
```

Verified: `api/modes.html` returns 200 and is the dedicated modes page.

**Trade-off, stated:** on GitHub the link now leaves the README instead of
scrolling within it. The README's own *Conversion Modes* section is unchanged and
still reachable by scrolling or GitHub's outline.

**Enforce it.** `check_readme_links()` currently **allows** `#` targets
explicitly. Change it to **reject** fragment-only targets in `README.md`, with a
message saying why. `docs/src/` is unaffected — mdBook renders it alone, and
in-page anchors there are ours.

## 4. Pin mdBook

`docs.yaml` installs `mdbook --vers "^0.5"`, which floats. The gate models 0.5.4's
wrapping and hidden-line rules. Pin exactly (`--vers "=0.5.4"`), and state the
modelled version in the gate's docstring **next to the rules it models**, so an
upgrade is a deliberate, reviewed change rather than a silent one.

After §2 there is less for an mdBook change to break, but the gate still models
mdBook's wrapping, and that must not drift unseen.

## 5. What this retires

- **D6** — closed by removal of the dependency, **not by verification**.
  Whether those anchors worked is no longer a question the project needs
  answered.
- **The hidden-line browser check** — retired the same way.
- **The owner is not a verification step** for either, now or at any future
  release.

## 6. Corrections this RFC records

- **RFC 031 handoff §4** recommended hidden lines as the idiomatic D1 repair. It
  modelled the gate and the page, and not the copy button. Mine.
- **CHANGELOG `[Unreleased]`** — my entry says the five examples "now compile".
  True as displayed-and-gated; false as copied. The implementer should correct it
  when §2 lands.
- **The 031 and 031b review outcomes** asked the owner for browser checks as the
  way to close D6 and hidden-line display. Superseded here.

## 7. Acceptance criteria

- [ ] The five blocks carry a visible fallible `main`; no hidden lines remain in any runnable `rust` block
- [ ] **Gate observed red** on a hidden line in a runnable block, with file and line in the message; green on `#[derive]` and `#![allow]`; `fragment` blocks exempt
- [ ] **Copy verified from source**: for each of the five blocks, the text between the fences compiles unchanged under a plain `fn main`-less file — i.e. the gate's compiled text equals the visible text
- [ ] `README.md` fragment links replaced with the guide URL
- [ ] **Gate observed red** on a reintroduced `](#conversion-modes)` in `README.md`; `docs/src/` fragment links still allowed
- [ ] `docs.yaml` pins `=0.5.4`; gate docstring names the modelled version
- [ ] CHANGELOG `[Unreleased]` entry corrected (§6)
- [ ] Docs gate runnable count unchanged; all six workflows green

---

## Amendment — at handoff, 2026-09-16: forbid hidden lines in **every** `rust` fence

§2.4 and §7 scope the rejection to *runnable* `rust` blocks and exempt `fragment`.
Tightened, before implementation:

**The gate rejects mdBook hidden lines in every `rust` fence** under `docs/src/`
and in `README.md` — runnable, `no_run`, `ignore`, `compile_fail` and `fragment`
alike.

**Why.** The copy-button defect in §2.2 does not depend on whether the gate
compiles a block. mdBook hides the lines in any `rust` fence, and Copy drops them
in any `rust` fence. A fragment is labelled incomplete, but a reader still
deserves to copy what they see. One unconditional rule is also simpler to state,
to enforce and to review than a rule with exemptions.

**Cost today: none.** Re-derived at handoff: the only hidden lines in any `rust`
fence are the five RFC 031 blocks, three lines each. No fragment uses them.

This is stricter than the accepted text, never looser.

## Amendment — from RFC 032 review, 2026-09-16: pin TypeScript to `latest`

RFC 032 pinned the docs gate's TypeScript to `typescript@5.9.3` and compiles with
that version's `tsc --init` options. `npm view typescript dist-tags` shows
`latest: 7.0.2` — what a reader starting a project today installs.

§4's pinning policy applies to both pins alike: **pin exactly, state what is
modelled, and re-derive the model when the pin moves.** Move `TYPESCRIPT` to
`typescript@7.0.2`.

Checked by the architect before amending: `tsc --init` in 7.0.2 writes the same
compile-relevant options as 5.9.3; the corrected `usage-nodejs.md` example
compiles under 7.0.2; the pre-fix example fails with the same two TS1484 errors.
The move loses no strictness on the one TypeScript example.
