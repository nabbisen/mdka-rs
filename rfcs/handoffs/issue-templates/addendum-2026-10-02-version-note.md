# Addendum 3 — issue templates: tell the reporter how to find the version

Amends `handoff.md`. The two prior addenda stay as issued; `addendum-2026-10-01b` is closed.

**Source.** Owner, after confirming the chooser and the form by hand: *"it seems better to me to add
a note about how to get the version which a user uses."* Correct — `version` is **required**, and a
reporter who does not know how to answer it either guesses or abandons the form.

**Scope.** One field's `description` in `.github/ISSUE_TEMPLATE/bug_report.yml`. Nothing else.

---

## Every surface can report its own version, and I checked all four

| Surface | Call | Verified |
|---|---|---|
| Rust crate | `mdka::version()` | `src/lib.rs:164` |
| Node.js | `require('mdka').version()` | `index.d.ts`: `export declare function version(): string`; `loader.js:132` re-exports `index.js` |
| Python | `mdka.version()`, or `mdka.__version__` | `mdka_python.pyi:58`; `__init__.py:20` sets `__version__ = version()` |
| CLI | `mdka --version` | confirmed on the published `3.2.0` binary |

## Say the runtime call, not the package manager 🛑

**Do not suggest `npm ls mdka` / `pip show mdka` / `cargo tree`.** Those report what is *installed*;
`version()` reports what is *running*, and for this project the difference is the interesting case:
the npm package loads a **platform-specific native binding**, so the wrapper and the binding can
disagree — which is exactly the shape of the `2.5.0`/RFC 040 load failures. A reporter telling us the
manifest version when the loaded binding is something else sends us the wrong way.

## The change

Replace the `version` field's `description` with one line that covers all four, for example:

```yaml
  - type: input
    id: version
    attributes:
      label: Version
      description: >-
        Ask mdka itself, so this is the version actually running: `mdka --version` on the CLI,
        or call `version()` — `mdka::version()` in Rust, `require('mdka').version()` in Node,
        `mdka.version()` in Python.
    validations:
      required: true
```

Wording is yours; the three requirements are that **all four surfaces appear**, that it says the
value comes from mdka itself rather than from a manifest, and that it stays one short paragraph —
this is a field hint, not documentation.

## Criteria

1. All four surfaces named, with the call that works for each.
2. No package-manager command suggested.
3. `bug_report.yml` still validates against `json.schemastore.org/github-issue-forms.json`.
4. Nothing else in the file changes; `feature_request.yml`, `config.yml`, `question.yml` untouched.

**No live-browser check is required this time.** The owner has confirmed the chooser and the form
render; a `description` string cannot break either, provided criterion 3 holds.

## Report

`.git-exclude/review-request/issue-templates/README.md`, appended. Push when green; no release.
