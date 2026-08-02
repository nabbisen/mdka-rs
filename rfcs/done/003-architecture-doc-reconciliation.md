# RFC 003 — Architecture documentation reconciliation

**Status.** Implemented (2.1.7)
**Tracks.** M1 · Trustworthy baseline. Corrects documentation statements that
are factually wrong about the current engine.
**Touches.** `docs/src/design/architecture.md`, `docs/src/api/elements.md`,
`docs/src/api/options.md`, `docs/src/api/modes.md`.
**Depends on.** RFC 004 — the workspace layout section describes files that RFC
004 removes.

## Summary

The published documentation describes a conversion pipeline that does not exist,
and makes several element-handling claims that observed behaviour contradicts.
This RFC corrects the statements that are wrong regardless of RFC 005's outcome,
and explicitly leaves the attribute-related claims to RFC 006.

## Motivation

`docs/src/design/architecture.md` is the page a new maintainer reads first. It
currently documents a five-step pipeline:

```
[2] Pre-process    preprocessor::preprocess(&doc, opts)
                    → filtered HTML string
[3] Re-parse       scraper::Html::parse_document(&cleaned)
```

Neither step exists. `html_to_markdown_with` (`src/lib.rs:94-97`) parses once
and traverses once, with preprocessing applied inline during traversal
(`src/traversal.rs:47-67`). The documented serialise-and-re-parse round trip was
removed from the engine but never from the documentation.

Three further claims were checked against the running engine at 2.1.6 and found
false or incomplete:

| Location | Claim | Observed |
|---|---|---|
| `elements.md:60-62` | HTML comments are retained in `Preserve` mode | Dropped in every mode, `Preserve` included. `<p>A</p><!-- c --><p>B</p>` → `"A\n\nB\n"` in both Balanced and Preserve |
| `elements.md:57-58` | Always-removed list omits `<svg>` and `<head>` | Both are removed unconditionally — `src/utils.rs:13-28` lists them |
| `options.md:104` | `drop_interactive_shell` is "Disabled by default in all modes" | Contradicts the table at `options.md:62` in the same file. `Minimal` sets it `true` (`src/options.rs:144`); the table is right and the prose is wrong |
| `modes.md:57-58` | Minimal removes shell elements "optionally… when `drop_interactive_shell` is true" | Misleading: `for_mode(Minimal)` already sets it true |
| `modes.md:96` | Preserve "retains HTML comments in the pre-processed output" | False, same error as `elements.md`. Comments are dropped in every mode. |
| `philosophy.md:38` | Non-recursion applies "both in the pre-processing pipeline and in the Markdown conversion step" | Asserts two traversals. There is one. |
| `philosophy.md:50` | Mode presets "are applied in a pre-processing pass" | Asserts a separate pass. Preprocessing is inline in the single traversal. |

The last three were found during a full `docs/src/` sweep at RFC 003 handoff
preparation (2026-08-02), after RFC 004 landed. All were verified against the
current build.

## Goals

- No statement in `docs/` contradicts observed engine behaviour, except those
  explicitly deferred below.
- The architecture page describes the pipeline that actually runs.

## Non-goals

- **Attribute semantics.** The `preserve_*` and `drop_presentation_attrs`
  descriptions in `options.md` and `modes.md` describe behaviour that does not
  currently exist but *will* after RFC 005. Rewriting them now means rewriting
  them twice. RFC 006 owns them, and they are deliberately left untouched here.
- Table support, or documenting its absence beyond what `elements.md` already
  implies. RFC 008 owns tables.
- Restructuring the mdBook navigation or adding new pages.
- Any code change.

## Proposed design

### `architecture.md` — pipeline section

Replace the five-step diagram with the three-step pipeline that exists:

```
HTML string
    │
    ▼
[1] Parse        scraper::Html::parse_document()
    │             → html5ever DOM (tolerant HTML5 parsing)
    ▼
[2] Traverse     traversal::traverse(&doc, opts)
    │             → non-recursive DFS over ego-tree, Enter/Leave events
    │             Preprocessing applied inline during traversal:
    │               · drops script/style/head/svg/… unconditionally
    │               · drops shell elements when opted in
    │               · unwraps generic wrappers when opted in
    │             Drives MarkdownRenderer
    ▼
[3] Finalise     renderer.finish()
                  → trim trailing whitespace, single trailing newline
```

State explicitly that there is **no intermediate HTML serialisation and no
second parse** — this is a deliberate design property worth recording, since the
removed round trip is what the old text described.

### `architecture.md` — workspace layout section

Remove the `tests/utils/preprocessor.rs` entry, which RFC 004 deletes. Verify
the remaining tree against `src/` as it stands after RFC 004 lands.

### `elements.md`

- Correct the comment claim: comments are removed in all modes, with no
  `Preserve` exception.
- Add `<svg>` and `<head>` to the always-removed list.

### `options.md` and `modes.md`

Correct only the `drop_interactive_shell` default statements so prose and table
agree with `src/options.rs`. Change nothing about the attribute fields.

## Compatibility

None. Documentation only. No API, behaviour, or artifact change.

## Security

Neutral. One incidental improvement: readers currently told that `Preserve` mode
retains comments might assume comment content survives conversion. It does not,
and the corrected text removes a false expectation about what reaches the output.

## Testing and verification

No automated test. Each correction is verified by executing the documented case
against the library and recording the output. The review request must include,
for every changed claim, the input HTML and the actual output.

Reference outputs already captured at 2.1.6:

```
<p>A</p><!-- secret --><p>B</p>   → "A\n\nB\n"   (Balanced and Preserve alike)
<p>X</p><svg><circle/></svg><p>Y</p> → "X\n\nY\n"
<html><head><title>T</title></head><body><p>Z</p></body></html> → "Z\n"
```

`cd docs && mdbook build` must succeed, and all internal links must resolve.

## Acceptance criteria

1. `architecture.md` describes a three-step pipeline with no preprocess or re-parse step.
2. `architecture.md` states explicitly that no intermediate serialisation occurs.
3. The workspace layout tree matches the repository after RFC 004.
4. `elements.md` **and `modes.md`** state comments are removed in all modes.
5. `elements.md` lists `<svg>` and `<head>` among always-removed elements.
6. `options.md` and `modes.md` agree with `src/options.rs` on `drop_interactive_shell` defaults.
7. `philosophy.md` no longer asserts a separate pre-processing pass or traversal.
8. No `preserve_*` or `drop_presentation_attrs` description is modified.
9. `mdbook build` succeeds with no broken links.

## Prohibited shortcuts

- Do not "fix" attribute documentation by describing what the code does today.
  RFC 005 changes it; RFC 006 documents it. Touching it here creates a third
  version of the same text.
- Do not delete the mode or option pages to resolve inconsistency.
- Do not update `docs/book/` by hand — it is generated output and gitignored.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Reviewer reads the untouched attribute docs as newly ratified | False confidence that all docs are now accurate | This RFC states the deferral explicitly; RFC 006 is tracked in the roadmap as the closing item |
| Further undocumented drift exists beyond the four items found | Documentation remains partly wrong after this RFC | The review sampled the full `docs/src/` tree, but implementer should re-verify each remaining behavioural claim while editing and report any additional findings rather than silently fixing them |

## Alternatives considered

- **Defer all documentation work to one pass after M2.** Rejected: the
  architecture page is actively misleading about the core pipeline now, and it
  is the entry point for anyone picking up RFC 005.
- **Auto-generate the element table from `src/utils.rs`.** Rejected as
  premature; the tag classification functions are not shaped for extraction, and
  the maintenance cost does not yet justify the tooling.
