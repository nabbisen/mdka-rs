# Developer Handoff — RFC 041 · deprecate the alias modes (the `3.0` runway)

**RFC.** `rfcs/accepted/041-conversion-surface-honesty.md` — §9, owner decision 2026-09-24: *"Deprecate the alias modes and the permanently-inert options in a minor … remove at `3.0`."*
**Milestone.** `2.8.0` (minor). **Nothing here is breaking**; every name keeps working and keeps its output.
**Priority.** P1 for the runway — `3.0` cannot remove what was never deprecated.
**Prepared.** 2026-09-25
**Baseline.** `1b35924` — 621 Rust (`cargo test --workspace`; plain `cargo test` gives 594 and is the wrong command), 42 Node, 25 loader, 91 Python; eight workflows green
**Scope.** Deprecation notices only, across four surfaces. **No conversion change. No option or mode removed. No signature changed.**

---

## 1. Why this exists, and what it is not

`Strict`, `Semantic` and `Preserve` are **aliases of `Balanced`** — byte for byte, asserted by
`mode_identity.rs` P2 — with no mechanism by which they could ever diverge. Their documented purposes promise
attribute fidelity Markdown cannot carry. The owner chose §5.1, *collapse to what is real*: **deprecate now,
remove at `3.0`.**

**This slice buys the right to remove them.** A `3.0` that deletes a public name nobody was warned about is
the kind of break this project does not do.

**It is not a cleanup of the option surface.** Five inert options were already deprecated in `2.2.0` and stay
exactly as they are. See §3 for the sixth, which stays *undeprecated* on purpose.

## 2. The modes 🛑

Deprecate **`ConversionMode::Strict`, `Semantic` and `Preserve`.** `Balanced` and `Minimal` are untouched:
`Minimal` is the only mode that genuinely converts differently.

**Four surfaces, because `#[deprecated]` does not cross FFI.** Each already has a precedent in this
repository — follow it rather than inventing a second mechanism:

| Surface | Mechanism | Precedent |
|---|---|---|
| **Rust** | `#[deprecated(since = "2.8.0", note = …)]` on the three variants | `src/options.rs:101–128`, the five inert fields |
| **Node** | `process.emitWarning(…, "DeprecationWarning")` | `node/src/lib.rs`, `warn_deprecated_field` |
| **Python** | `warnings.warn(…, DeprecationWarning)` | `python/src/lib.rs`, `warn_deprecated_field` |
| **CLI** | A warning on stderr; **`--mode strict` still works** | new — see §4 |

**Warn only when the caller asked for it**, never for a default. Both binding helpers already say why:
*"warning on every call regardless of intent would just get the warning suppressed wholesale."* The same rule
applies here — a user who never names a mode must see nothing.

**The note must say what to do, not only that something is wrong.** *"`Strict` is an alias of `Balanced` and
produces identical output; use `Balanced`. It is removed in 3.0."* Point at the modes page.

**Expect `#[deprecated]` to fire inside our own code** — `parse_mode`, `as_str`, `for_mode`, the test
harness's `MODES`. `for_mode` already carries `#[allow(deprecated)]`. Add it where it is genuinely internal,
**and nowhere else**: an `#[allow(deprecated)]` that silences a real caller is how a deprecation becomes
invisible.

## 3. The sixth inert option stays undeprecated — do not "finish the job"

`unwrap_unknown_wrappers` is inert and **carries no `#[deprecated]`, deliberately.** `src/options.rs:139–143`
states the distinction and it is the right one:

- The five deprecated options are inert because **Markdown has no attribute syntax.** That is permanent.
- `unwrap_unknown_wrappers` is inert because **unwrapping currently leaves nothing observable** — a fact
  about today's renderer, not about Markdown.

RFC 041 §9 says *"permanently-inert"*. Only the first group qualifies. **Leave it exactly as it is**, and if
you think that is wrong, say so in the report rather than changing it.

## 4. The CLI

`--mode strict|semantic|preserve` **keeps working and keeps its output.** Print one line to **stderr**:

```
mdka: warning: --mode strict is an alias of balanced and produces identical output; it is removed in 3.0
```

**stderr, not stdout** — the CLI's stdout is converted Markdown and a warning there would corrupt a pipe.
`cli/tests/rfc039_a7_cli_surface.rs` already asserts on `--help`; extend the help text to mark the three as
deprecated, and assert both the help text and the stderr line.

## 5. Criteria

1. The three variants carry `#[deprecated(since = "2.8.0", …)]`; `Balanced` and `Minimal` do not.
2. **Each of the four surfaces warns, demonstrated with its actual output pasted** — a `rustc` warning, the
   Node `DeprecationWarning`, the Python `DeprecationWarning`, the CLI's stderr line.
3. **Silence when not asked for**: a Node or Python call that passes no mode, and `mdka` with no `--mode`,
   emit nothing. Asserted, not just observed.
4. **Output is byte-identical to `2.7.0` in all five modes**, over the `mode_identity.rs` corpus. A
   deprecation that changes output is not a deprecation.
5. `mode_identity.rs` P1 and P2 still pass unchanged.
6. **A test that fails if a variant is removed without this warning ever existing** — i.e. assert the
   deprecation is present, so `3.0` cannot quietly skip the runway.
7. Every `#[allow(deprecated)]` added is listed in the report with one line on why it is internal.
8. Documentation: `docs/src/api/modes.md` and the options page say deprecated-and-removed-at-`3.0`; the
   README's mode table likewise if it lists them.
9. 621 Rust (`cargo test --workspace`) plus new cells, 42 Node, 25 loader, 91 Python; eight workflows green.

## 6. Not in this slice

- **Removing anything.** That is `3.0`, with RFC 039 Half B, and one migration guide covering both.
- RFC 039 Half B's result-type unification, and the `mdka_python` namespace leak (RFC 041 §9 folds it in, but
  it is surface work, not a deprecation).
- `unwrap_unknown_wrappers` — §3.
- Any conversion behaviour.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.

**Use `git commit -F - -- <paths>`, not `git add <paths> && git commit`** — the second form commits the whole
index in a shared tree.

**Two things to know rather than act on.** `release artifact gate` is red on a prep commit that changes the
README's Quick Start, the artifact contract or the build matrix; §5.8 may touch the README's mode table, so
if it goes red at release prep for that reason, that is the standing explanation in `RELEASE-CHECKLIST.md`
§1 and not a finding. And our one known consumer pins `= 2.5.1`, uses `Minimal`, and is therefore unaffected
by every line of this slice.

Report to `.git-exclude/review-request/041-deprecation-slice/README.md`, leading with §5.2 and §5.3 — the
four warnings and the four silences. Those are the deliverable; the attribute itself is one line.
