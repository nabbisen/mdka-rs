# RFC 041 — The conversion surface: options that cannot act, modes that cannot differ

**Status.** Implemented — the wording half shipped in `2.6.0`, the deprecations in `2.8.0` and the removals in `3.0.0`, 2026-09-26
**Author.** Architect
**Created.** 2026-09-24
**Milestone.** Unscheduled → `3.0` (breaking). Nothing here belongs in a patch or a minor.
**Source.** Owner authorised opening this on 2026-09-24, after asking whether the `2.4.2` recommendations matched *"finally clean, safe and secure, robust and sophisticated design"* and *"APIs for users not to be confused or misunderstand"*. They did not; the measurement is recorded in the project's internal review records.
**Touches.** `src/options.rs`, `src/lib.rs`, `cli/src/main.rs`, `node/src/lib.rs`, `python/src/lib.rs`, `docs/src/api/options.md`, `docs/src/api/modes.md`, `docs/src/getting-started/usage-cli.md`, `README.md`.
**Relates to.** RFC 039 Half B — the other unscheduled `3.0` API document. These should be decided together.

---

## 1. Summary

**The API offers nine options and five modes. Two options and two behaviours are real.**

This is not a tidiness complaint. The names and documented purposes of the inert parts promise things that
cannot happen, and a user acts on that promise. `Preserve` is documented *"for archiving and auditing"*;
someone converting an archive selects it and receives `Balanced`, byte for byte, with nothing in the output
to tell them otherwise.

**No direction is proposed here.** §5 lays out three, with their costs. The decision is the owner's and it is
a `3.0` decision.

## 2. The measurement

Each option flipped alone against a ten-shape corpus, everything else at the mode default, on the published
`2.4.1` crate:

| Option | Shapes changed / 10 | |
|---|---|---|
| `preserve_ids` | 3 | real |
| `drop_interactive_shell` | 1 | real |
| `unwrap_unknown_wrappers` | 1 | **only the corruption fixed in `2.4.2`**; inert thereafter |
| `preserve_classes` | 0 | inert, deprecated `2.2.0` |
| `preserve_data_attrs` | 0 | inert, deprecated `2.2.0` |
| `preserve_aria_attrs` | 0 | inert, deprecated `2.2.0` |
| `preserve_unknown_attrs` | 0 | inert, deprecated `2.2.0` |
| `drop_presentation_attrs` | 0 | inert, deprecated `2.2.0` |

**After `2.4.2`, six of the eight booleans cannot change output.**

And the modes, read from `src/options.rs` rather than from behaviour:

| Mode | Fields differing from `Balanced` | Of those, any that can act |
|---|---|---|
| `Strict` | `preserve_classes`, `preserve_data_attrs`, `preserve_unknown_attrs`, `drop_presentation_attrs` | **none** |
| `Preserve` | the same four | **none** |
| `Semantic` | `unwrap_unknown_wrappers` | that one only — inert after `2.4.2` |
| `Minimal` | `preserve_ids`, `drop_interactive_shell`, `unwrap_unknown_wrappers` | **yes, genuinely distinct** |

## 3. Why this is a correctness problem, not a cosmetic one

### 3.1 "May diverge again" is not true of three of them

`docs/src/api/modes.md` says the four *"currently produce identical output"* and that they *"remain distinct
API, are not merged, and may diverge again"*. For `Strict` and `Preserve` there is **no mechanism by which
they could diverge**: the only fields separating them from `Balanced` are inert *permanently*, because
Markdown has no syntax for HTML attributes. That is not a coincidence waiting to end. They are aliases.

A reader takes "currently" as "today's build" and plans around a difference arriving. Nothing can arrive.

### 3.2 The mode descriptions promise fidelity that does not exist

| Mode | What we tell the user | What they get |
|---|---|---|
| `Strict` | *"Removes as few attributes as possible; for debugging and comparison"* | identical to `Balanced` |
| `Preserve` | *"Retains as much of the original as possible; for archiving and auditing"* | identical to `Balanced` |
| `Semantic` | *"Favours semantic attributes and document structure; for SPAs and accessibility"* | identical to `Balanced` (after `2.4.2`) |

Each sentence is about **attributes**, and attributes are exactly what Markdown cannot carry. The mode system
was designed around an axis the output format does not have. `Minimal` survives because its distinctness
rests on the two things that *are* expressible — dropping shell elements, and omitting `id` anchors.

### 3.3 An inert option with no test is how `2.4.2` happened

`unwrap_unknown_wrappers` was documented inert, was not asserted inert, and silently acquired an effect that
corrupted tables in two default modes. `2.4.2` adds that assertion for all six. **This RFC exists because the
assertion is a guard, not a design** — it keeps the surface honest; it does not make it small.

## 4. Constraints any answer must respect

1. **`2.4.2` ships first and independently.** Nothing here may delay a corrupt-output fix.
2. **Breaking.** Removing an option or a mode changes the Rust struct, the Python kwargs, the
   `JsConversionOptions` fields and the CLI flags. `3.0`, with a migration guide (release policy, `ROADMAP.md`).
3. **Silence is not an option.** Whatever is chosen, `modes.md` must stop implying a divergence that cannot
   occur, and the mode descriptions must stop promising attribute fidelity. That part is not breaking and
   could ship earlier if the owner prefers.
4. **Unwrapping is expressible**, unlike attribute retention. Any answer that treats
   `unwrap_unknown_wrappers` as dead-forever is wrong on the facts — it is unbuilt, not impossible.

## 5. Directions, none recommended here

### 5.1 Collapse to what is real

Remove the five permanently-inert options and the three alias modes. Two options, two modes
(`Balanced`, `Minimal`), possibly renamed to say what they do.

*For:* the surface becomes exactly the behaviour; nothing to misread. *Against:* the largest break; every
consumer pinning `ConversionMode::Preserve` must change; loses reserved seats for future axes.

### 5.2 Keep the names, make them honest

Keep five modes as documented aliases — `Strict`/`Semantic`/`Preserve` stated outright as *aliases of
`Balanced`*, not as modes that coincide — and keep the options with a machine-checked inert marker.

*For:* no break at all; could ship in a minor. *Against:* still nine options and five modes for two
behaviours; the user must read documentation to learn that three choices are one choice.

### 5.3 Build an axis the modes can actually differ on

Implement `unwrap_unknown_wrappers` properly and find other *expressible* distinctions, so the modes become
real again around what Markdown can represent.

*For:* keeps the design's intent and its names. *Against:* speculative — RFC 040 aside, no one has asked for
unwrapping; it invents scope to justify a shape rather than fitting the shape to demand. Wrong order.

## 6. Open questions for the owner

1. **Direction** — §5.1, §5.2, §5.3, or a combination.
2. **Should §4.3 (the honest wording) ship early**, in the next minor, ahead of any structural decision? It
   is not breaking, and it removes the misleading promise straight away. My inclination is yes, but it is
   listed as a question because rewording a mode's stated purpose is a product statement, not an edit.
3. **Decide with or separately from RFC 039 Half B**, the other unscheduled `3.0` API document.

## 7. Not in scope

The `2.4.2` fix and its invariants; RFC 040; any change to conversion output. **This RFC changes no
Markdown.** Every option it discusses is one that, by then, provably does nothing.

---

## 8. Recommendation — added 2026-09-24, after acceptance

§1 said no direction was proposed, and at the time that was right: choosing between §5.1, §5.2 and §5.3 is a
product decision and I had nothing to ground it in. The owner has since stated the criteria — *"finally
clean, safe and secure, robust and sophisticated design"* and *"APIs for users not to be confused or
misunderstand (and the documentation for it)"* — which is enough to derive one. §6 remains formally open;
this is an answer to it, not a substitute for the owner's.

### 8.1 Direction: §5.1, collapse to what is real

- **§5.2 is excluded by the word "finally".** It ends with nine options and five modes for two behaviours,
  permanently, and asks the user to read documentation to learn that three of their five choices are one
  choice. It makes the surface honest without making it clean.
- **§5.3 is excluded by "sophisticated".** Sophistication is the design fitting the problem, not the problem
  being enlarged to fit the design. Building an axis so that existing names stop being wrong is the wrong
  order, and §5.3 admits nobody has asked for unwrapping.
- **§5.1 is the only one that ends.** Two modes, and the options that can act.

### 8.2 The migration carries no output risk, which is unusual and decides the "safe" question

Normally collapsing an API risks changing behaviour. Here it cannot, and this is the strongest argument for
§5.1 rather than against it:

- The five attribute options are **provably inert** — 0/10 shapes each, and inert *permanently*, since
  Markdown has no attribute syntax.
- `Strict` and `Preserve` differ from `Balanced` **only** in those fields; `Semantic` only in
  `unwrap_unknown_wrappers`, inert after `2.4.2`.

So **mapping the three alias modes onto `Balanced` changes no user's output at all** — they already produce
identical bytes, by construction and not by coincidence. The entire cost of this change is names and
compilation. Nothing a user converts comes out differently.

That is the difference between a risky collapse and a safe one, and it will not be true later: the longer
three aliases sit there being documented as potentially divergent, the more likely someone gives one of them
a real effect and the free migration is gone.

### 8.3 Sequence

**Next minor — non-breaking, and it does the user-facing work immediately:**

1. §6.2's honest wording: `modes.md` stops saying *"may diverge again"* of modes that cannot, and `Strict`,
   `Semantic` and `Preserve` stop being described as *"for debugging and comparison"* / *"for SPAs and
   accessibility"* / *"for archiving and auditing"* — three purposes built on attribute fidelity that
   Markdown cannot carry. They are documented as **aliases of `Balanced`**.
2. Mark the three mode variants `#[deprecated]`, so the **compiler** tells users, not only the documentation.
   **Precedent: this project already deprecated the five attribute options in a minor, `2.2.0`**, so this is
   consistent rather than novel. One caveat to weigh, not hide: a downstream build running `-D warnings`
   turns a new deprecation into a failure. That was equally true in `2.2.0` and was accepted then.

**`3.0` — the removal**, with the migration guide the release policy requires. By then the deprecation has
been visible for at least one release and the guide says "delete the argument; your output does not change",
which is a true and unusually easy migration note.

### 8.4 §6.3: decide with RFC 039 Half B, not separately

Both rewrite the same public surface at `3.0`. Deciding them apart risks two migrations for the same users,
or two designs that each assume the other did not happen. **They should be one decision and, when the time
comes, one migration guide** — even if they remain two documents.

### 8.5 What is still the owner's

Everything above is a recommendation. The decision, the scheduling of `3.0`, and whether §8.3's first step
goes into the next minor or waits are unchanged as owner calls; §6 stays the list of record.

---

## 9. §6 answered — owner decision, 2026-09-24

| Question | Decision |
|---|---|
| **§6.1 direction** | **§5.1 — collapse to what is real.** Deprecate the alias modes and the permanently-inert options in a minor (precedent: the attribute options in `2.2.0`), remove at `3.0` |
| **§6.2 ship the honest wording early** | **Yes.** Non-breaking, and the inaccuracy is published on crates.io, on PyPI and in the README, not only on the docs site |
| **§6.3 decide with RFC 039 Half B** | **Yes.** Both rewrite the same surface at `3.0`; deciding apart risks two migrations or two designs each assuming the other did not happen |

**§6.2 is separable and goes first.** It is documentation only and corrects text that is untrue in effect
today, so it does not wait for the structural work. Handoff:
`rfcs/handoffs/041-conversion-surface-honesty/wording-slice-handoff.md`.

**The structural half stays unscheduled** and is now bound to RFC 039 Half B. When it is scheduled, one
migration guide covers both.

Also folded in, from the documentation audit: **`mdka_python` leaks into the Python package's public
namespace** — `dir(mdka)` exposes 14 names where 13 are documented. It belongs to this RFC's surface work,
not to a patch.

---

## 10. A constraint on the `3.0` removal, found by the `2.8.0` deprecation slice

**When the alias variants are removed, `parse_mode` / `FromStr` must do something deliberate with the
strings, and it must not be a silent behaviour change.**

The deprecation warns the caller who *names* `ConversionMode::Strict` — a compile warning now, a compile
error at `3.0`, impossible to miss. It does **not** warn the caller who reads `mode = "strict"` from a
config file: `FromStr::from_str` carries an internal `#[allow(deprecated)]` and maps the string to the
variant silently. A library must not print, and adding a logging dependency for one notice is out of
proportion, so `2.8.0` correctly left it.

**That makes the string caller the worst-served one:** no warning at all today, and at `3.0` whatever
`parse_mode` then does, at runtime, in production.

So `3.0` must choose, and record the choice in the migration guide:

- **Accept and map** — `"strict"` continues to parse as `Balanced`. Nothing breaks; the names live on in
  string form, which partly defeats the collapse.
- **Reject with a message that names the replacement** — `Err("conversion mode 'strict' was removed in 3.0;
  it was an alias of 'balanced'")`. Honest, and a runtime break for callers who were never warned.

**Returning a bare `Err` with the existing `unknown conversion mode: strict` text is the one unacceptable
option**: it tells a user their config is wrong rather than that it is obsolete.

The same question applies to the CLI's `--mode` and to both bindings, which parse strings too.

