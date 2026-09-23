# Developer Handoff — `2.4.1` · Four fixes from the 2.4.0 consumer pass

**Source.** `.git-exclude/reviewed/2.4.0-consumer-pass/README.md` — five findings, all reproduced by me
against the released binary
**Authorised.** Owner, 2026-09-23 — cut a `2.4.1`
**Milestone.** M4 · `2.4.1` (patch)
**Priority.** P1
**Prepared.** 2026-09-23
**Baseline.** `2f72f5b`, tag `2.4.0` — 566 Rust / 42 Node / 90 Python, seven workflows green

---

## 0. Why a patch at all

The engine is fine: 43 of 43 documented examples pass, the Quick Start is byte-exact on five surfaces, ~18
real-world tables parse with zero ragged rows.

**Two false statements are sitting on three registry pages**, and a registry README can only be corrected by
publishing. That is the reason for the release; §2 and §3 ride along because they are small and land in the
same place.

**Two findings are deliberately NOT in scope** — the `<sup>` rule and `py.typed` without stubs. Both need an
owner decision on direction and neither is a regression. I am raising them separately; do not touch them.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. 🛑 `Semantic` drops `id` anchors on unwrapped wrappers — **fix the code, not the docs**

```
<main id="x"><p>hi</p></main>
  Balanced / Strict / Preserve  ->  <a id="x"></a>\n\nhi
  Semantic                      ->  hi                      <-- wrong
```

Also `<div id>`, `<section id>`, `<article id>`, `<span id>`. Reproduces through the CLI, Rust, Node and
Python.

**This is `preserve_ids` being violated, not a documented difference.** `api/options` defines it without
qualification — *"emit an anchor for elements carrying a non-empty `id` attribute"* — and `Semantic` honours
it for `<h2 id>` and `<p id>` while silently dropping it for wrappers. `Minimal` correctly emits nothing
because `preserve_ids` is **off** there, which is the control that shows the rule is fine and its application
is not.

It is also exactly RFC 036 option C's own principle, which I wrote and then failed to apply twice:
**unwrapping removes the tag, not what it stood for.** The paragraph break was kept for that reason; the
anchor is the same case.

**So: an unwrapped wrapper carrying an `id` emits its anchor when `preserve_ids` is on.** That makes the
published four-modes claim *true* rather than requiring four documents to retract it.

**Criteria.** The four shapes above byte-identical across `Balanced`, `Strict`, `Semantic`, `Preserve`;
`Minimal` still emits no anchor; wrappers **without** an `id` unchanged; non-wrapper elements unchanged;
non-table, non-wrapper output byte-identical to `2f72f5b`.

### 1.1 And fix the fixtures, which is the part that let this ship

`api/modes` cites two characterisation tests as proof, asserting their fixtures were *"specifically chosen to
discriminate a difference if one existed, rather than inferring identity from fixtures that happen not to
distinguish them."*

**Neither fixture puts an `id` on a wrapper.** The stated methodology is right; the fixtures do not implement
it. Three independent checks were blind the same way — those tests, the harness cells and batteries, and
**my own 177-shape probe**, which is what I based RFC 036 option C on.

Add `id` attributes to a wrapper in the discriminating fixtures, so the test does what its own comment
claims. **If you can see any other axis those fixtures fail to discriminate, add it and say so** — that
sentence should be earned, not asserted.

## 2. 🛑 A blank line in a code block inside a table cell corrupts the cell

```
cell containing:  a / blank / b / blank / c
  emits  | `a`<br>``<br>`b`<br>``<br>`c` |
  parses TableCell(`a`, <br>, `<br>`b`<br>`, <br>, `c`)
```

A blank source line emits ` `` `, which is **not** an empty code span but a stray delimiter run. It pairs
with the next run, so `b` lands inside a code span it was never in and the `<br>`s become literal text.

Not synthetic: Wikipedia's *Markdown* article — whose syntax-reference table is the reason the page exists —
is mangled **24 times**.

**Criteria.** The three-line shape above round-trips with `a`, `b` and `c` each in their own code span and
the blank lines represented without a stray run. Single-line and no-blank-line code cells **byte-identical**
to `2f72f5b`. Cells for both.

## 3. 🛑 The README denies this release's headline feature

`README.md:187` — live on crates.io, npm and PyPI right now:

> **Tables are not yet converted.** `<table>` cell text is emitted without structure or separators, so a
> table becomes a run of joined text.

It describes precisely the defect `2.4.0` fixed. The docs site does not carry it. **Delete it and put the
truth there**, including the four shapes that fall back — the CHANGELOG's `2.4.0` entry has wording you can
reuse.

While you are in the README: sweep for any other claim `2.4.0` falsified.

## 4. 🛑 `mdka page.html > out.md` leaves an empty file

Verified: `redirect.md` **0 bytes**, the conversion in `page.md`. A file argument writes a sibling and prints
nothing to stdout.

Claimed in **three** places — `docs/src/getting-started/usage-cli.md:73`, `cli/src/main.rs:66` (`--help`) and
the source comment at `:26`. The companion example in the same sentence,
`echo '<h1>Hi</h1>' | mdka --preserve-classes > out.md`, **is** correct, which is how it survived review.

**Mine: I wrote that claim into the RFC 039 handoff and never ran it.**

Fix the wording in all three. The stdin example is the one that demonstrates the stream split; the file
example demonstrates something else and should say what it actually does.

## 5. Criteria for the slice

1. §1–§4 all fixed, each verified by **parsing** where it produces Markdown.
2. **Non-table, non-wrapper output byte-identical to `2f72f5b`.**
3. Cells for §1 and §2, all five modes.
4. `--help` and the docs agree with observed behaviour — run the commands, do not read them.
5. All three suites reported separately; `fmt`/`clippy`; counts against 566 / 42 / 90.
6. CHANGELOG `[Unreleased]` → a `2.4.1` section. **It is the release notes** — `create-release.yaml` links
   straight to it. Say plainly that `2.4.0`'s README misstated table support and that `Semantic` did not
   match the other three.

## 6. Report back

`.git-exclude/review-request/2.4.1-consumer-pass-fixes/README.md`: what moved, before/after parses, the
byte-identical set, the fixture change and what it now discriminates, the three test counts, and anything
here I got wrong.
