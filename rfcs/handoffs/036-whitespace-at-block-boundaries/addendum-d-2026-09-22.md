# Addendum D — RFC 036 slice `036d`, 2026-09-22

**Amends.** `slice-d-handoff.md` (frozen; this addendum is the only correct channel)
**Issued after.** the `036d` review, `.git-exclude/reviewed/036d-div-separator/README.md`
**Status.** The slice is approved on substance. **Three fixes before it commits**, one of them mine.

---

## 1. 🛑 Required — two CI tests still assert the old behaviour

You updated `cli/tests/unwrap_wrappers_flag.rs`, with a good note. Its two siblings — same fixture, same
comment, same intent — were not, and both run in CI:

- `node/test.js:249` (`ci.yaml:135`)
- `python/test_mdka.py:436` (`ci.yaml:198–212`)

Each asserts the outputs **differ** and that the unwrapped one is `"BeforeinnerAfter\n"`. Verified against
your tree: both are now `"Before\n\ninner\n\nAfter\n"`, so all three assertions in each fail. **Committing
as staged turns CI red.**

Update both to assert inertness, carrying the same explanation the Rust test now has.

**The lesson is the general one, not the two files.** `cargo test` does not reach `node/test.js` or
`pytest`. This field is exposed in all three bindings, so a change to its meaning has to be verified in all
three. **Run `cargo test`, `node test.js` and `pytest` before committing** — and treat "which suites can
even see this change?" as the first question whenever a slice touches a field that crosses the FFI boundary.

**Worth knowing:** both comments record that this fixture was chosen because *"RFC 005 Slice A found
block-element fixtures cannot discriminate this field at all."* It is the project's only known
discriminating fixture, and it has stopped discriminating. That is the strongest evidence yet for option C's
premise, and it came from the one direction nobody pointed at it.

## 2. Required — `src/options.rs:128`

```rust
/// Whether to unwrap wrapper elements that carry no meaning.
pub unwrap_unknown_wrappers: bool,
```

The book, the CLI help, the README and `usage-cli.md` all say *"no effect today"*. **docs.rs does not** —
and for a Rust consumer that is the primary documentation. Half two's point is that documentation must not
claim an effect the code lacks; this is the last place it does.

One line, in the style of `preserve_classes`'s *"Inert. …"* three lines above. **Still no `#[deprecated]`,
still no warning.**

## 3. Required — `docs/src/api/elements.md:52`, and this one is mine

```
`<div>` counts in Balanced, Strict and Preserve, but not in Minimal and Semantic, which unwrap it
```

False now — a `<div>` counts as a block in all five modes — and it is load-bearing, because it sits inside
the loose/tight list rule.

`slice-d-handoff.md`'s docs table said *"`elements.md:19` — the row is already correct, verify it, do not
edit it."* You verified line 19 and reported it unchanged, which is exactly what I asked. **I scoped that
instruction to one line of a file with two live claims.** My miss; please fix it in the same commit.

## 4. Optional — your call

`docs/src/design/architecture.md:41`, the pipeline diagram: *"unwraps generic wrappers when opted in"*.
Still literally true. But in a diagram describing what each stage *does*, a stage with no observable effect
is worth a second look. Not required.

## 5. Then commit

RFC 036 closes with this slice, except `036c` (the setext collision), scoped in `addendum-b-2026-09-22.md`
and not yet handed off. Report the re-run of all three suites in the commit message or a short follow-up
note; no new review package needed unless something surprises you.
