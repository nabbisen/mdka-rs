# Addendum — RFC 041 deprecation slice, 2026-09-25

**Amends.** `deprecation-slice-handoff.md`, which is frozen and unedited. Two follow-ups from the review at
`.git-exclude/reviewed/041-deprecation-slice/README.md` §3. Both ride `2.8.0`.
**Baseline.** `ed1423c`.
**Scope.** One stderr string and one documentation sentence. **No behaviour, no output, no new warning.**

---

## 1. Make the CLI's two warnings one shape — and the defect is mine

The slice produced:

```
mdka: warning: --mode strict is an alias of balanced and produces identical output; it is removed in 3.0
warning: mdka: `--preserve-classes` has no effect and is deprecated (see …). Markdown has no attribute …
```

**You followed the handoff exactly; the handoff was wrong to prescribe a shape without checking the one
already there.** You flagged it, which is why it is being fixed rather than shipped.

**Change the older one to the new shape**, not the reverse: `progname: warning: …` is the ordinary Unix form,
and `warning: mdka:` reads as though *mdka* were the warning's subject.

```
mdka: warning: `--preserve-classes` has no effect and is deprecated (see https://nabbisen.github.io/mdka-rs/api/options.html). Markdown has no attribute syntax, so this option was never expressible in the output.
```

Keep the wording; change only the prefix. **The existing test pins the old string** — update it, and say in
the report which test and what it now asserts.

**Check for others.** If any other CLI warning uses the old shape, bring it along; if this is the only one,
say so, so the next reader knows the surface is now consistent rather than merely mostly consistent.

## 2. Tell the string caller what to do

`docs/src/api/modes.md` now marks the three as deprecated and removed at `3.0`. It speaks to the caller who
**names** the variant. It does not speak to the one who writes `mode = "strict"` in a config file — and that
caller **gets no warning at all today** (`FromStr` maps the string silently; a library must not print).

Add a sentence to the deprecation section, in substance:

> The string forms — `"strict"`, `"semantic"`, `"preserve"`, as accepted by `--mode`, by the bindings and by
> `ConversionMode::from_str` — are deprecated in the same way, but **nothing can warn you about them at
> compile time**. Change them to `"balanced"` now; what they do at `3.0` is decided with the removal.

**Do not promise what `3.0` will do with them.** That is an open decision, recorded in RFC 041 §10, and the
sentence must not pre-empt it.

## 3. Criteria

1. One warning shape on the CLI, `mdka: warning: …`, and a statement in the report that you checked for
   others.
2. The pinning test updated, named in the report.
3. `modes.md` carries §2's sentence, promising nothing about `3.0`.
4. **No new warning, no message removed, no output change.** The `--preserve-classes` warning still fires in
   the same conditions.
5. 629 Rust (`cargo test --workspace`), 49 Node, 25 loader, 98 Python; eight workflows green.

## 4. One process point, not a criticism

Your report said *"approved for push and pushed."* **I had not approved it**, and the handoff says *"report
first; a green run is not approval."* The owner may have approved it directly, which is entirely theirs to
do — in which case the record should name them, so "approved" is a fact rather than something a later reader
has to infer. Nothing here needed undoing and nothing does now.

Report to `.git-exclude/review-request/041-deprecation-slice/README.md` as an appended part, not a new file —
this is the same slice.
