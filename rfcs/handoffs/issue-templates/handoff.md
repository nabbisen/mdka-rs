# Developer Handoff — issue templates that fit an HTML→Markdown converter

**Source.** Owner asked whether `.github/ISSUE_TEMPLATE/` fits the project. It does not — audit at
`.git-exclude/reviewed/issue-template-audit/`. Read it first; the findings are the design.
**Size.** Small. **Scope.** `.github/ISSUE_TEMPLATE/bug_report.yml` and `feature_request.yml` only.
**Not in scope:** `config.yml` and `question.yml`, both of which are correct as they stand.

---

## 1. The one rule this rewrite exists for 🛑

**Every field that can contain HTML or Markdown must carry `render:`.**

Without it GitHub renders the content, and for a Markdown-emitting tool that *erases the bug*: a
missing space, a stray `_`, two trailing spaces that make a hard break. Three of this repo's real
reports (#29, #41, #42) are exactly that class and survive today only because their authors wrote
their own fences. The form must not depend on a reporter's good habits.

Use `render: html` for the input, `render: markdown` for outputs.

## 2. `bug_report.yml` — the fields

Replace the body. Keep `name`, `description`, `title: "[Bug]: "`, `labels: ["bug"]`.

| id | type | required | notes |
|---|---|---|---|
| `package` | dropdown | ✅ | `Rust crate (crates.io)`, `Node.js (npm)`, `Python (PyPI)`, `CLI binary`, `Not sure` |
| `version` | input | ✅ | **required**, unlike today. "The version of the package above, e.g. `3.2.0`" |
| `input` | textarea, `render: html` | ✅ | **The HTML you converted.** Smallest fragment that still shows it |
| `actual` | textarea, `render: markdown` | ✅ | **The Markdown mdka produced** |
| `expected` | textarea, `render: markdown` | ✅ | **The Markdown you expected** |
| `mode` | dropdown | ✅ | `Balanced (default)`, `Minimal`, `Not sure` |
| `options` | checkboxes | — | `preserve_ids`, `drop_interactive_shell`, `emphasis_from_style`, each "changed from its default" |
| `environment` | textarea | — | OS, and **the runtime for the chosen package** — Node / Python / rustc version as applicable. Not Rust-only |
| `notes` | textarea | — | Anything else: a spec link, how you found it |

**Drop `steps` entirely.** It is `required: true` today and there is one step; no real report has ever
had three.

**Keep the opening `markdown` block short**, and put one sentence in it that earns its place: *a
minimal HTML fragment and the exact Markdown, as text, is the whole report* — that is what every
useful report this project has received already did.

**Do not add a mode/option note to the `question.yml` or anywhere else.** One place.

## 3. `feature_request.yml` — one sentence

Add, in the opening `markdown` block, that significant changes go through an RFC — proposal, owner
acceptance, then implementation — so a requester knows the shape and the timescale rather than
expecting a patch. **One sentence. Do not restructure the form**; its four fields are fine.

## 4. Criteria

1. Every HTML/Markdown-bearing field carries `render:`. **Paste a filled-in example of #41 into the
   new form's fields and show the rendered issue preserves the exact bytes**, including the space
   before `*_`. That is the whole point and it must be demonstrated, not asserted.
2. `package`, `version`, `input`, `actual`, `expected`, `mode` are all required; `steps` is gone.
3. `feature_request.yml` gains the RFC sentence and nothing else.
4. `config.yml` and `question.yml` **unchanged**.
5. **The chooser still renders.** A malformed issue form does not error — it *silently disappears*
   from the template chooser. After pushing, open
   `https://github.com/nabbisen/mdka-rs/issues/new/choose` and confirm **all three** templates plus
   the security link are listed. **This is the verification; a green CI proves nothing here**, since
   no workflow validates these files.

## 5. Report

`.git-exclude/review-request/issue-templates/README.md`, leading with criteria 1 and 5 — the
byte-preservation demonstration and the chooser screenshot or listing. Commit and push **only this
slice**; no release.
