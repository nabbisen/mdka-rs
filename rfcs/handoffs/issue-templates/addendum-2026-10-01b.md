> **CLOSED 2026-10-01 — do not action.** Criteria 1 and 5 need a signed-in browser session, which is
> unavailable to both the dev team and the architect. There is **no API route**: GraphQL's
> `issueTemplates` covers Markdown templates only, not YAML forms (verified against `cli/cli`,
> `rust-lang/rust` and `vercel/next.js`) — my claim otherwise in the review was wrong. The slice is
> accepted on the schema validation; the remaining check is a one-minute task left to the owner. See
> `.git-exclude/reviewed/issue-templates/README.md`.

# Addendum 2 — issue templates, 2026-10-01 (b)

Amends `handoff.md` and `addendum-2026-10-01.md`, both of which stay as issued. Review:
`.git-exclude/reviewed/issue-templates/README.md`.

**The files are accepted and live.** `logs` is restored correctly and all three templates re-validate
against GitHub's schema. Nothing needs reverting.

**Criterion 1 needs redoing**, because issue #82 does not show what the report says it shows.

## Why

The rendered HTML you quoted proves a **fenced code block in an issue body** preserves bytes. That
was never in doubt. What criterion 1 is for is proving that **the form puts the fence there** — that
is what `render:` does and the only thing protecting a reporter's trailing space.

#82 carries **no labels**, while `bug_report.yml` declares `labels: ["bug"]`. The `bug` label exists
in this repo, and `/issues/82/events` has a single `closed` event — no `labeled`, no `unlabeled`, so
it never had one. An issue created through the form would have been labelled. The body is a faithful
reproduction, `### Logs` and `_No response_` included, but a reproduction is not a submission.

## The redo — one pass, both criteria

1. Open `https://github.com/nabbisen/mdka-rs/issues/new/choose`.
2. **Criterion 5:** record whether **Bug report**, **Feature request** and **Question** are all
   listed, plus the security link.
3. Open **Bug report**, fill `input` / `actual` / `expected` with #41's values, submit **without
   editing the title prefix**.
4. **Criterion 1:** the issue arrives titled `[Bug]: …` **and labelled `bug`** — that pair is only
   reachable through the form — and the trailing space before the closing `*` survives in the
   rendered body.
5. Close it with a comment saying why it existed, and quote the resulting title, label and rendered
   block in the report.

## One thing to carry forward

This is the fourth time in this sequence that real evidence has not described what it appeared to.
Each time the work was sound and the framing was not. **When reporting a demonstration, say what
produced the artifact** — "submitted through the web form at /issues/new/choose", not "observed on a
real rendered page". The mechanism is the claim; the artifact is only the trace.

## Scope

No file change expected. If the redo turns something up, report it rather than fixing it.
