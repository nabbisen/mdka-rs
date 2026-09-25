# Developer Handoff — RFC 048 slice 3 of 3 · the migration guide and the documentation sweep

**RFC.** `rfcs/accepted/048-the-3.0-surface.md` §8.7, §8.8, §8.10 — accepted by the owner, 2026-09-25
**Milestone.** `3.0`. Slices 1 (`29d179a`) and 2 (`d665aea`, closed at `e9ef8c4`) have landed. **This is the last slice before release prep.**
**Priority.** P1 — `3.0` is the first breaking release this project has made, and this is the document people will judge it by.
**Prepared.** 2026-09-26
**Baseline.** `d1e0f7d` — 629 Rust (`cargo test --workspace`), 59 Node, 25 loader, 91 Python; eight workflows green
**Scope.** `docs/`, `README.md`, and one workflow comment. **No code. No test. No conversion change.**

---

## 1. The migration guide is the deliverable

A new page, in `docs/src/` and in `SUMMARY.md`. **Nothing like it exists** — `SUMMARY.md` has no migration
entry, because this project has never made a breaking release.

**Write it for someone upgrading from `2.9.0`, not for someone who read the RFCs.** They have not read RFC
048 and should not have to.

### 1.1 What it must open with

**`3.0` changes no conversion output.** Not one byte, in either surviving mode, and it is asserted against
published `2.9.0` by `mode_identity.rs`'s P3 goldens. **Say that first**, because it decides how a reader
spends their afternoon: there are no fixtures to re-verify, no output to diff. Everything below is
compile-and-rename.

### 1.2 Every break, per binding, before and after

Derive them from the code, not from this list — but nothing here may be missing:

**All bindings**
- Modes `Strict`, `Semantic`, `Preserve` → use `Balanced`. Byte-identical; that is why they went.
- Options `preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`, **and in Rust only**
  `preserve_unknown_attrs` and `drop_presentation_attrs` → remove them; output is unchanged.
- **`unwrap_unknown_wrappers` → removed.** §2 below: this one gets its own treatment.
- String mode names: `"strict"` and friends now give *"was removed in 3.0; it was an alias of 'balanced'"*,
  distinct from *unknown conversion mode*. **Tell config-file readers to change the string now.**

**Rust**
- `html_file_to_markdown*`: `Result<ConvertResult, MdkaError>` → **`Result<PathBuf, MdkaError>`**. `src` is
  gone from the return because the caller passed it.
- `html_files_to_markdown*`: `Vec<(&P, Result<PathBuf, MdkaError>)>` → **`Vec<FileOutcome>`**, and the
  **`'a` lifetime parameter is gone** — outcomes own their `src`.
- **`FileOutcome` is not `Clone`** (`MdkaError` wraps `std::io::Error`). A caller who cloned the old tuple's
  parts must restructure. **Say this explicitly**; it is the one break that is not a rename.
- `parse_mode` is **deprecated**, not removed → use `str::parse`, which reports *why* a name was rejected.

**Node.js — the longest section, and say so**
- **`ConvertResult` is gone.** It served single *and* bulk, so **every file-converting Node caller is
  touched.**
- `htmlFileToMarkdown` → `Promise<string>`, the destination path. Failure **rejects**.
- `htmlFilesToMarkdown` → `Promise<Array<FileOutcome>>`, with **`ok`** — a field Node did not have.
- **`htmlToMarkdownWith`, `htmlToMarkdownWithAsync`, `htmlFileToMarkdownWith`, `htmlFilesToMarkdownWith` are
  removed.** Options are now an optional last argument on the base names.
- **An unrecognised option key is now rejected by name**, where it was silently ignored. A caller passing a
  misspelt or removed key gets an error where they previously got nothing — **a behaviour change for code
  that was already wrong**, and worth its own line.

**Python**
- `html_file_to_markdown*` → **`str`**, raising `MdkaError`, instead of `ConvertResult`.
- `BulkConvertResult` → **`FileOutcome`**, same four fields; **it is frozen** (immutable).

**CLI**
- `--mode strict|semantic|preserve`, `--preserve-classes`, `--preserve-data`, `--preserve-aria`,
  `--unwrap-wrappers` all removed, each with its own message on stderr. Quote them; they are good messages
  and a user who meets one should recognise the guide.

### 1.3 `unwrap_unknown_wrappers` gets its own paragraph — RFC 048 §8.10 🛑

**Write it assuming the reader never saw the deprecation warning.** It was deprecated in `2.9.0` and removed
in `3.0`, which shipped the day after: almost nobody ran a version that warned.

Say what it was, that it **could not change the output**, that removing the call changes nothing, and that
if wrapper handling ever becomes expressible it returns as a **new** option rather than this one coming back.

### 1.4 What did *not* change

Worth a section. `html_to_markdown`, `html_to_markdown_with`, `html_to_markdown_many` and their per-language
forms; `Balanced` and `Minimal`; `preserve_ids` and `drop_interactive_shell`; `version()`; every element and
escaping rule. **A migration guide that only lists losses reads like a bigger break than it is.**

## 2. The documentation sweep

`docs/` currently describes the `3.0` surface for everything slices 1 and 2 touched — they kept it true as
they went. What is left is **the backlog the pre-`3.0` audit found and two slices deliberately did not fold
in**:

| | |
|---|---|
| `docs/src/design/architecture.md` | The workspace layout omits **`src/table.rs`** — 637 lines, RFC 008, shipped in `2.4.0`. Add it. Check the rest of the tree against the diagram while you are there |
| `docs/src/getting-started/usage-rust.md:167`, `usage-nodejs.md:188` | `version()` → `// e.g. "2.3.0"`. Six minors stale, in the one example whose subject is the current version |
| `docs/src/getting-started/usage-python.md` | **Never mentions `version()`**, though Python exports it |
| `.github/workflows/scripts/check-docs-examples.py:487` | A comment still names `preserve_unknown_attrs`. The only non-`docs/` file in this slice |

## 3. The performance page — label it, do not regenerate it

`docs/src/design/performance-characteristics.md` states its numbers were measured at **`2.3.0` (`main` @
`c9cbbb8`)**. That is six minors and two conversion milestones ago — RFC 008's tables, RFC 009's element
coverage and the `2.6.0`/`2.7.0` inline work have all landed since, and **nobody has re-measured.**

**Do not regenerate it in this slice.** It is a measurement job with its own discipline (RFC 012), it is not
documentation, and bundling it here would put a benchmark run on the critical path of a release.

**Do make the staleness unmissable**: a note at the top of the page saying which version these figures
describe and that they have not been re-measured since. The page is currently honest but easy to read as
current, and `3.0` will bring readers to it.

**Recorded separately** for the owner to schedule.

## 4. Criteria

1. A migration guide exists, is in `SUMMARY.md`, and covers every break in §1.2 with before and after.
2. It **opens** with "no conversion output changes", and says the claim is asserted, not hoped.
3. **Node's section is the longest**, and the guide says why.
4. §1.3's paragraph is written for a reader who never saw the warning.
5. §1.4 exists — what did not change.
6. **Every code sample in the guide runs.** `docs example gate` compiles them, so this is enforced, not
   promised — but check locally first, because a guide is a bad place to find out.
7. The four sweep items in §2 are done.
8. The performance page carries §3's note; **no benchmark is run.**
9. **A completeness check, scripted**: every public name in all three bindings either appears in the guide or
   is unchanged from `2.9.0`. Paste the script and its output. Do not check by reading — the audit's own
   first pass produced a false negative that way.
10. No change to `src/`, `cli/`, `node/`, `python/` or any test. Eight workflows green.

## 5. Not in this slice

- Release preparation — the version bump, `CHANGELOG.md`, RFC moves. **That is the next handoff**, and it is
  where `3.0.0` is chosen as a number.
- Regenerating benchmarks (§3).
- Anything in `src/`. If you find a defect, **report it — do not fix it here**; `3.0`'s code is frozen and a
  documentation slice is the wrong place to unfreeze it.

## 6. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours. **Name who approved anything you push.**
`git commit -F <file> -- <explicit paths>`.

Report to `.git-exclude/review-request/048-slice-3-migration-guide/README.md`, leading with §4.9's
completeness check and the guide's opening. **After this, `3.0` is ready to prepare.**
