# Developer Handoff — RFC 039 Half A · Additive API parity

**Governing RFC.** [RFC 039](../../accepted/039-public-api-surface-coherence.md) — §1.1 the principle, §2 findings, §3 Half A, §5 criteria
**Milestone.** M4 · `2.4.0`
**Priority.** P1
**Prepared.** 2026-09-22
**Baseline.** `2e9af49` — **455 Rust, 39 Node, 80 Python** tests, seven CI workflows green
**Sequencing.** Independent of RFC 037. Both can be in flight; they touch different files.

---

## 0. The shape of this slice

**Everything here is additive. Nothing changes an existing signature and nothing is removed.** If you find
yourself about to alter or delete something a user can call, stop — that is Half B, which is specified but
not scheduled and needs its own acceptance.

The one exception is **A7's stderr changes**, which move *where* two things are written without changing
what is computed. Called out in A7.

**Read §1.1 of the RFC before starting.** Parity here means *the same operations, behaving the same,
expressed idiomatically in each language* — not identical signatures. Rust has no optional parameters, so
`_with` belongs there; Node takes an optional options object; Python takes keyword arguments. All three are
correct. Do not make them look alike at the cost of looking wrong in their own language.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. A1 / A3 — Python stops contradicting itself

`html_to_markdown_with` exists; `html_file_to_markdown_with` does not, so file options live as kwargs on the
plain function. A user who learns the convention from the string API concludes file conversion cannot be
configured.

**Add**, as thin wrappers over what already exists:

- `html_file_to_markdown_with(path, out_dir=None, mode=…, preserve_ids=…, …)`
- `html_files_to_markdown_with(paths, out_dir, mode=…, …)`
- `html_to_markdown_many_with(html_list, mode=…, …)`

**Keep the existing kwargs on the plain functions** — removing them is breaking. Point the documentation at
the `_with` forms as the convention, and say the plain forms still accept options.

Add all three to `__all__`.

## 2. A2 — `html_to_markdown_many` in Rust and Node

Python has it; Rust and Node have nothing. It is the project's parallel batch entry point, and it exists in
one of three languages.

- **Rust:** `html_to_markdown_many(&[impl AsRef<str>]) -> Vec<String>` and `..._many_with(…, &ConversionOptions)`.
  Parallel when the `parallel` feature is on, sequential otherwise — see A4; **the function exists either way.**
- **Node:** `htmlToMarkdownMany(htmls: Array<string>, options?: JsConversionOptions)`. **One function, not
  four.** New functions follow the target shape, not today's sync/async × with/without sprawl. Add an
  `Async` form only if you find a concrete reason, and say what it was.

**It cannot fail**, so it returns plain strings and does not touch §2.1's result-type question. Keep it that
way — that question is Half B's.

## 3. A3 — options on Python's `many`

Covered in §1. It is currently the only conversion function in the project that cannot be configured at all.

## 4. A4 — the `parallel` feature stops changing the API shape

```
$ cargo build   # with default-features = false
error[E0425]: cannot find function `html_files_to_markdown` in crate `mdka`
```

`html_files_to_markdown` and `html_files_to_markdown_with` are `#[cfg(feature = "parallel")]`. A feature
named `parallel` reads as a performance choice; losing two public functions by opting out of parallelism is
not predictable.

**Make them unconditional.** Without the feature they do the same work sequentially. This is additive —
nobody loses a function, and the default build is unchanged.

**Criterion 2 pins it:** `--no-default-features` must expose the **same public function set** as the default
build, asserted by a test or doc-test rather than by inspection, so it cannot silently regress.

Watch the collision check in `html_files_to_markdown_with` — the "first path wins, later collisions error"
rule (RFC 021) must behave identically in the sequential path. It is a correctness rule, not a scheduling
artifact.

## 5. A5 / A6 — `version()` in Rust, `py.typed` in the wheel

`version()` exists in Node and Python, not Rust. Add it.

`py.typed` is absent, so mypy reports *"module is installed, but missing library stubs or py.typed marker"*
for every typed Python user. `pyproject.toml` has `python-source = "."` and `python-packages = ["mdka"]`, so
the file goes at `python/mdka/py.typed`. **Verify it lands in the built wheel** — criterion 4 is about the
installed artifact, not the source tree. This project has been bitten before by checking the tree and not
the artifact (the 2026-08-31 audit).

## 6. A7 — the CLI, and the one judgment call in this slice

| Fix | Today |
|---|---|
| Add `--no-preserve-ids` | Unknown option. Anchors are on in 4 of 5 modes with no way off short of `--mode minimal`, which also drops the shell and unwraps wrappers |
| Correct `--help` | *"Keep id attributes"* — it does not keep attributes, it **emits `<a id=…></a>` anchors**, and the wording implies opt-in when it is already on. State the per-mode default |
| Deprecation notices to **stderr** | `mdka --preserve-classes` is silent, exit 0. Python warns on every path; Rust has `#[deprecated]`; the CLI says nothing — and it is the surface most likely to sit unread in a script |
| Progress to **stderr** | `mdka page.html > out.md` puts `page.html -> page.md` in `out.md`; the conversion goes to a sibling `page.md` |

**The judgment call.** Moving progress to stderr changes what `>` captures. Anyone parsing stdout today gets
a progress line, so parsing it is improbable and the change makes redirection do the obvious thing — but it
*is* a visible change to a shipped surface. **I am authorising it**, because a silent wrong answer is worse
than a surface change, and because the documented behaviour (`mdka page.html # → page.md`) is unaffected.
Note it in the CHANGELOG under a heading a user will read.

## 7. Acceptance criteria

RFC 039 §5, all seven. The two that are easy to under-test:

- **§5.2** — `--no-default-features` exposes the same public function set, pinned by an assertion.
- **§5.4** — mypy is satisfied by the **installed wheel**, not by a file in the source tree.

And **§5.6: no existing signature changes and nothing is removed.** The current suites pin this; if one of
them needs editing, that is a signal you have left Half A, not a test to update.

## 8. Report back

`.git-exclude/review-request/039-half-a/README.md`: the new public surface per binding, the `--no-default-features`
evidence, the wheel-mypy evidence, before/after for each CLI change, the three test counts, and anything here
I got wrong. Every package this milestone has corrected something of mine; that is the standard.
