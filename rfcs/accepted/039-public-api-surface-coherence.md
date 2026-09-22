# RFC 039 — Public API surface coherence

**Status.** Accepted (owner, 2026-09-22) — **Half A scheduled for `2.4.0`; Half B specified, unscheduled**
**Author.** Architect
**Created.** 2026-09-22
**Milestone.** M4 · Coverage and durability → `2.4.0` (Half A); `3.0` (Half B)
**Source.** A function-by-function survey of all three bindings and the CLI, 2026-09-22, after the owner added *"interfaces of APIs should be intuitive and easy to understand to users"*. Request: `.git-exclude/review-request/m4-open-decisions-v3/README.md`.
**Touches.** `src/lib.rs`, `src/options.rs`, `node/src/lib.rs`, `python/src/lib.rs`, `python/mdka/`, `python/pyproject.toml`, `cli/src/main.rs`, `docs/src/`.

---

## 1. Summary

> The configuration surface advertises five modes and eight options. It delivers **two** behaviours and
> **two** working options — and the same operation returns **three different result shapes** depending on
> which language you call it from.

Nothing here is a defect in output. Every item is a defect in the **interface**: a user cannot predict what
a function returns, whether an option exists, or whether a convention holds, without reading the
implementation.

### 1.1 The principle this RFC applies

**Parity means the same operations exist and behave the same, expressed idiomatically in each language — not
identical signatures.** Rust has no optional parameters, so `_with` is right there; Node has optional object
arguments; Python has keyword arguments. All three are fine. What is not fine is a binding that is
**inconsistent with itself** (§2.2) or an operation that exists in only one language (§2.3).

## 2. Findings

### 2.1 One operation, three result shapes

| | single file | bulk |
|---|---|---|
| Rust | `Result<ConvertResult, MdkaError>`, `ConvertResult { src, dest }` | `Vec<(&P, Result<PathBuf, MdkaError>)>` — tuples with a borrowed key; **`ConvertResult` is not used** |
| Python | `ConvertResult { src, dest }` | `BulkConvertResult { src, dest?, error? }` — **a second type** |
| Node | `ConvertResult { src, dest?, error? }` | `Array<ConvertResult>` — same type; `error` documented *"(bulk conversion only)"* |

**Node's single-file type advertises a field it never sets.** `htmlFileToMarkdown` returns a type whose
`dest` is optional and which carries `error`, while the function guarantees both are determined. A type that
lies about its own invariants is the least intuitive thing in this surface.

**Python's two type names describe the wrong distinction** — `ConvertResult` vs `BulkConvertResult` reads as
"one vs many"; the real difference is "cannot fail vs can".

### 2.2 Python contradicts its own convention

`html_to_markdown_with` exists. **`html_file_to_markdown_with` does not** — file options are keyword
arguments on the plain function. A user who learns the convention from the string API will look for it, not
find it, and conclude file conversion cannot be configured.

This is the gap most likely to be hit, because it is *within* one language rather than between two.

### 2.3 The parallel batch path exists in one binding and takes no options

`html_to_markdown_many(html_list)` — Python only, `par_iter()` with the GIL released. **It is the only
conversion function in the project that cannot be configured at all.** Rust and Node have no equivalent.

### 2.4 A feature flag changes the API shape

`html_files_to_markdown` and `html_files_to_markdown_with` are `#[cfg(feature = "parallel")]`. With
`default-features = false` they **disappear** — verified: `cannot find function html_files_to_markdown in
crate mdka`.

A feature named `parallel` reads as a performance choice, not a surface choice. Losing two public functions
by opting out of parallelism is not something a user will predict.

### 2.5 Smaller asymmetries

- `version()` exists in Node and Python, **not in Rust**.
- Async exists only in Node, as four string functions — and async paths **cannot emit deprecation warnings**
  (napi-rs: `Env` is not `Send`), so the variants the README's Quick Start recommends are the least
  instrumented. Carried from M2.
- Eight option fields, **two effective**; five modes, **two behaviours**. Honestly documented since RFC 036,
  but a newcomer reading `ConversionOptions` still sees eight knobs and cannot tell which two are real
  without leaving the type.
- No `py.typed`, so mypy fails for every typed Python user.
- The CLI's five confusions — see the request, §6.

## 3. Half A — additive, `2.4.0`, breaks nothing

| # | Change | Closes |
|---|---|---|
| A1 | Python: add `html_file_to_markdown_with`, `html_files_to_markdown_with`. The existing kwargs on the plain functions stay, for compatibility; the docs point at `_with` | §2.2 |
| A2 | Add `html_to_markdown_many` to Rust (`+ _with`) and Node (`(htmls, options?)`) — idiomatic per language, §1.1 | §2.3 |
| A3 | Python: add `html_to_markdown_many_with` | §2.3 |
| A4 | Make `html_files_to_markdown*` **unconditional**; `parallel` changes only whether the work is parallel, never which functions exist | §2.4 |
| A5 | Add `version()` to Rust | §2.5 |
| A6 | Ship `py.typed` in the wheel | §2.5 |
| A7 | CLI: add `--no-preserve-ids`; correct `--help` for `--preserve-ids`; emit deprecation notices on **stderr**; send per-file progress to **stderr** | §2.5 |

**New functions follow Half B's target, not today's shape** — `many` cannot fail, so it returns `Vec<String>`
and touches none of §2.1's question.

## 4. Half B — specified now, shipped at `3.0`, **not implemented**

Recorded so every future addition checks itself against a target instead of against its nearest neighbour.

1. **One result type across all three bindings**, with a single answer to *"can this fail"*. A fallible
   result is fallible in the type; an infallible one does not carry an `error` field.
2. **Rust's bulk conversion uses it** instead of `Vec<(&P, Result<…>)>`.
3. **An option surface that shows only what works**, with a documented path back if attribute or wrapper
   preservation is ever implemented — the modes and fields kept today for exactly that reason.

Half B is a breaking change and needs its own acceptance, scoping and migration note when it is scheduled.
**Nothing in Half A forecloses any of it.**

## 5. Acceptance criteria — Half A

1. Every function in §3 exists, is exported, is documented, and appears in the bindings' own test suites.
2. `cargo build --no-default-features` compiles a crate exposing the **same public function set** as the
   default build. A test or doc assertion pins this, so A4 cannot silently regress.
3. `html_to_markdown_many` in Rust and Node returns the same values as mapping `html_to_markdown` over the
   inputs, for the same inputs, and its `_with`/options form matches `html_to_markdown_with`.
4. `pip install` of the built wheel satisfies mypy — no *"missing library stubs or py.typed marker"*.
5. CLI: `--no-preserve-ids` turns anchors off in every mode; `--help` describes `--preserve-ids` accurately
   and states its per-mode default; `mdka --preserve-classes` prints a notice on **stderr** and nothing on
   stdout; `mdka page.html > out.md` leaves `out.md` containing **the conversion**, with progress on stderr.
6. **No existing signature changes and nothing is removed.** Every current call still compiles and behaves
   identically — pinned by the existing suites.
7. All three suites run and reported separately; `fmt`/`clippy` clean.

## 6. Not in scope

Output behaviour of any kind. The async-warning limitation (a napi-rs constraint, documented at
`node/src/lib.rs:42–49`). Adding async to Python or Rust. Retiring any mode or option — Half B.
