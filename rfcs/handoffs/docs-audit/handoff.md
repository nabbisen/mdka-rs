# Developer Handoff — documentation corrections, 2026-10-15

**Source.** Owner found `mdka = "2"` in an install example and two badges reading "Failed", and asked
for a documentation audit. Audit: `.git-exclude/reviewed/docs-audit-2026-10-15/` — **read it first**;
it records what was checked and found *correct*, which is most of the work and tells you what not to
touch.
**Scope.** `README.md`, `docs/src/getting-started/installation.md`,
`docs/src/getting-started/usage-cli.md`, `docs/src/migration-3.0.md`. **Nothing else.**
**Baseline.** `origin/main` at `82f8a14`; eight workflows green; 675 Rust tests.

---

## 1. `mdka = "2"` → `"3"`, in two places

```
README.md:132
docs/src/getting-started/installation.md:9
```

These are the only dependency-version strings in the documentation. Do not go looking for others;
the audit already swept for them.

## 2. `usage-cli.md`'s "All Options" table is missing two flags 🛑

The table states *"This table mirrors `mdka --help`. If the two ever disagree, `--help` is the
truth."* It omits `--emphasis-from-style` (3.1.0) and `--backslash-hard-breaks` (3.3.0).

- [ ] Add both rows, with descriptions taken from `--help` so the two genuinely agree.
- [ ] **Verify by diffing**, not by eye: extract the long flags from `mdka --help` and from the
      table, and show the two sets are equal. Paste that comparison. The table makes a checkable
      promise, so check it.
- [ ] `--emphasis-from-style`'s description must not oversell it: it is off by default, and
      `options.md` already records that it does nothing in `Minimal` (wrapper unwrapping) and nothing
      for table cells. One clause pointing at `options.md` is enough; do not restate the trade-off
      tables.

## 3. `migration-3.0.md:275` — pin the count to `3.0`, do not just fix it

> `JsConversionOptions` has three fields, `mode`, `preserveIds` and `dropInteractiveShell`

It has five now. **Do not write "five".** That goes stale at the next option, which is exactly how
this finding came to exist. Say that the **`3.0`** surface has three and point at
`./api/options.md` for the current set. The guide documents a step from `2.x` to `3.0`; pinning it is
what keeps it true.

## 4. The badges — replace three, keep one

Currently `release-executable`, `release-npm` and `release-pypi` badges show the **default branch**,
but those workflows only ever run on tag refs, so each is frozen on an old `main` run. Two read
"failing" (npm, Executable); PyPi happens to read "passing". All three are equally meaningless.

- [ ] Replace those **three** with registry version badges, all verified rendering `v3.3.0`:
      `img.shields.io/crates/v/mdka`, `img.shields.io/npm/v/mdka`, `img.shields.io/pypi/v/mdka`.
      Link each to its registry page. Consider
      `img.shields.io/github/v/release/nabbisen/mdka-rs` for the executables, which is the published
      artifact the `Executable` badge was trying to speak for.
- [ ] 🛑 **Keep the `CI` badge exactly as it is.** `ci.yaml` runs on `main`, so it works and means
      what it says.
- [ ] **Do not use `?branch=3.3.0`.** It renders "passing" and I tested it, but it needs editing at
      every release — the same maintenance debt as finding 1. `?event=workflow_dispatch` does not
      work at all; also tested.
- [ ] After pushing, **fetch the rendered SVGs and paste their `<title>` text.** A badge is a live
      external image; the only proof is what it renders.

## 5. Not in scope — stated so it is not "tidied" in passing

- The usage pages for **Rust, Node and Python**. Their options sections are worked examples that link
  to `options.md`; they make no completeness claim and are **not** defects.
- Every "removed in 3.0" section anywhere. All correct.
- `options.md`'s "five fields, four act on the output". Correct.
- `migration-3.0.md`'s **"58 documents"**. Correct — I verified it against `mode_identity.rs`'s four
  directories (51 + 2 + 3 + 2). **Do not "fix" it to 54**; 54 was my error, not the doc's.

## 6. Criteria

1. Both `mdka = "3"`.
2. `usage-cli.md`'s flag set equals `mdka --help`'s, **shown by a diff**.
3. `migration-3.0.md` pins its count to `3.0` and points at `options.md`.
4. Three badges replaced, `CI` untouched, and the rendered `<title>` of each new badge pasted.
5. Nothing in §5 touched. `git diff --stat` confined to the four files in Scope.
6. `check-docs-examples.py` clean — `README.md` and `installation.md` both carry runnable blocks.

## 7. Report, then stop

`.git-exclude/review-request/docs-audit/README.md`, leading with criteria 2 and 4. Push when green;
no release.
