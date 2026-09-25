# Developer Handoff — RFC 048 slice 2 of 3 · the result model and Node's conventions

**RFC.** `rfcs/accepted/048-the-3.0-surface.md` §3 and §4 — accepted by the owner, 2026-09-25
**Milestone.** `3.0` (breaking). Slice 1 landed at `29d179a`; slice 3 is documentation and the migration guide.
**Priority.** P1 — the largest slice of the three, and the one users feel.
**Prepared.** 2026-09-26
**Baseline.** `125414c` — 625 Rust (`cargo test --workspace`), 47 Node, 25 loader, 88 Python; eight workflows green
**Scope.** The file-conversion result model in all three bindings, Node's function names, and two follow-ups from the slice 1 review. **String conversion is untouched. No conversion output changes.**

---

## 1. What changes, and the one sentence that explains all of it

**The distinction is fallible vs infallible, not single vs many.**

- **String conversion cannot fail.** It returns a string, or a list of strings. **No result type.** Already
  true; leave it alone.
- **One file** — the caller asked about one thing, so failing the call is right, through each language's own
  error channel. It returns **the destination path and nothing else**.
- **Many files** — one failure must not abort the batch, so each file gets its own outcome, and the outcome
  must say which file it belongs to.

### 1.1 The target

| | Rust | Node | Python |
|---|---|---|---|
| one file | `Result<PathBuf, MdkaError>` | `Promise<string>`, rejects | `str`, raises `MdkaError` |
| many files | `Vec<FileOutcome>` | `Promise<Array<FileOutcome>>` | `list[FileOutcome]` |

```rust
pub struct FileOutcome {
    pub src: PathBuf,
    pub result: Result<PathBuf, MdkaError>,
}
```

Rust expresses *exactly one of ok or error* in the type. The dynamic bindings must flatten it, and **all
three gain the same predicate**: `src`, `dest`, `error`, `ok` — where Node has **no `ok` today**.

**`ConvertResult` is removed, not renamed.** Its single-file use had one useful field (`src`, which the
caller just passed in), and in Node it declared `dest?` and `error?` — **fields the single-file function
never leaves unset or sets**. A type that lies about its own invariants is the thing this slice exists to
delete.

`BulkConvertResult` (Python) **is** renamed to `FileOutcome`, same four fields: `ConvertResult` vs
`BulkConvertResult` reads as *one vs many*, and the real difference is *cannot fail vs can*.

## 2. Node folds `With` away — §4

Node is the only binding holding two conventions at once: `htmlToMarkdownWith(html, options)` beside
`htmlToMarkdownMany(htmls, options?)`. JavaScript has optional arguments, so the second is the idiom, and
Node's own docstring already argues for it citing RFC 039 §3 A2.

| Remove | Keep, with `options?` added |
|---|---|
| `htmlToMarkdownWith` | `htmlToMarkdown(html, options?)` |
| `htmlToMarkdownWithAsync` | `htmlToMarkdownAsync(html, options?)` |
| `htmlFileToMarkdownWith` | `htmlFileToMarkdown(path, outDir?, options?)` |
| `htmlFilesToMarkdownWith` | `htmlFilesToMarkdown(paths, outDir, options?)` |
| — | `htmlToMarkdownMany(htmls, options?)` unchanged |

**Rust and Python keep their `_with` pairs.** Rust has no optional parameters, so a pair is the idiom;
Python's pair is consistent with itself since `2.4.0`. RFC 039 §1.1: *"parity means the same operations
exist and behave the same, expressed idiomatically in each language — not identical signatures."* **Do not
"harmonise" Rust or Python into Node's shape.**

**Async stays Node-only and is now a decision, not an accident.** One async string form; file functions stay
`Promise`-returning. Rust and Python gain nothing async.

## 3. Why removing live names here is legitimate, when slice 1's rule said otherwise 🛑

Slice 1 removed only names that **did nothing**, and every one had warned since `2.2.0`, `2.8.0` or `2.9.0`.
The review then refused to remove `parse_mode` for exactly that reason.

**This slice removes names that work, and none of them ever warned.** That is not a contradiction, and you
should understand why before you start:

- A **return-type change cannot be deprecated.** One function cannot return two types. There is no
  mechanism, in any of the three languages, to warn about `html_file_to_markdown` returning `PathBuf`
  instead of `ConvertResult`.
- **A major version is that mechanism**, and the migration guide is the notice. That is what `3.0` is for.

**One case where coexistence *was* possible, and is deliberately not taken:** Node's `…With` names could
have been kept as deprecated aliases beside the new signatures, since both can exist. **They are not**,
because Node callers are touched by `ConvertResult`'s removal regardless, so aliases would soften half a
break while doubling the surface for the whole of `3.x`. If you think that is wrong, say so in the report —
but do not quietly keep them.

## 4. Two follow-ups from the slice 1 review

### 4.1 Deprecate `parse_mode`

`parse_mode` is `s.parse().ok()`: it routes through `FromStr` but **discards the error**, so a removed mode
name gives `None` exactly as an unknown one does. RFC 048 §6 has been corrected accordingly.

**Deprecate it; do not remove it** — it was never warned about, which is the rule slice 1 was built on.

```rust
#[deprecated(since = "3.0.0", note = "use `str::parse`, which reports why a name was rejected; \
             `parse_mode` discards that message")]
```

It has **no callers in the repository**; only its own test and the docs mention it. Update both.

### 4.2 Node silently ignores an option it does not know

napi drops unknown fields, so from plain JavaScript `{preserveClasses: true}` neither errors nor warns after
slice 1 — while TypeScript catches it, Python raises `TypeError` and the CLI exits 1. **You are already
reshaping this object**, so close it here: reject an unrecognised key with an error that **names the key**,
and for the six options and three modes removed in slice 1, say *removed in 3.0* rather than *unknown* —
the same distinction `FromStr` now draws.

**Check the cost before committing to it:** if rejecting unknown keys means validating every call on a hot
path, measure it and report the number rather than assuming it is free.

## 5. Traps

- **Neither napi nor PyO3 can export a Rust `Result` field.** Each binding builds its own flattened object
  from `FileOutcome`; the shared type is the Rust one. That is expected — do not contort the Rust type to
  suit the bindings.
- **`FileOutcome`'s Python `repr` must be Python-idiomatic** — RFC 048 criterion 11. Today
  `BulkConvertResult`'s repr prints `error=Some("…")`, a Rust `Option` debug format, and **omits `dest` and
  `ok` entirely**. The accessors are correct; only `repr` leaks. The rename is where this gets fixed, and it
  must not be inherited.
- **`MdkaError` has one variant, `Io`.** Check what a bulk run reports when the *output directory* cannot be
  created, in all three bindings, and make sure the new shape does not lose it.
- **Node's error channel differs by arity after this slice**: single-file rejects, bulk resolves with
  `error` set per entry. That is the design; state it in `index.d.ts`'s doc comments so it is not read as an
  inconsistency.

## 6. Criteria

1. `ConvertResult` is gone from all three bindings. `FileOutcome` exists in all three with `src`, `dest`,
   `error`, `ok` — **`ok` included in Node**.
2. Single-file conversion returns the destination only and fails through the language's own channel, with a
   test per binding that the failure is observable idiomatically — a Rust `Err`, a Python raise, a Node
   rejection.
3. Bulk conversion: one failing file does not abort the batch, and the surviving entries carry their
   destinations. Tested per binding.
4. Node exports **no `…With` name**; every options-taking Node function takes them optionally. `index.d.ts`
   regenerated, and the drift check passes.
5. `parse_mode` deprecated per §4.1, with the warning demonstrated.
6. §4.2: an unrecognised Node option key is rejected by name, with *removed* distinguished from *unknown*,
   or a measured reason in the report why it is not.
7. **`FileOutcome`'s Python `repr` shows all four fields and no `Some(…)`** — paste it.
8. **`Balanced` and `Minimal` output is byte-identical to published `2.9.0`** — `mode_identity`'s P3 goldens
   still pass untouched. **If a golden needs changing, stop and report**: nothing in this slice may change a
   byte of conversion.
9. Counts reported with their commands; eight workflows green. `docs/` must not describe a removed name as
   live — minimum edits, listed; the guide is slice 3.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours. **Name who approved anything you push.**
`git commit -F <file> -- <explicit paths>`.

Report to `.git-exclude/review-request/048-slice-2-result-model/README.md`, leading with §6.1–§6.3 and §6.8 —
the new shape, that failure is observable in each language's own way, and that conversion did not move.
