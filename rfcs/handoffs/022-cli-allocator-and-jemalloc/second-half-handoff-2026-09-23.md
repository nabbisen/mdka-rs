# Developer Handoff — RFC 022 second half · Remove `mdka::alloc_counter`

**Governing.** RFC 022's second half — no RFC number of its own; the owner decision and the gate are recorded in `ROADMAP.md` under *"Scheduled: remove `mdka::alloc_counter` — with a required gate"*
**Gate.** ✅ **PASSED 2026-09-23** — `.git-exclude/reviewed/alloc-counter-removal-gate/README.md`
**Milestone.** M4 · `2.4.0`
**Priority.** P1 — **it blocks RFC 012**
**Prepared.** 2026-09-23
**Baseline.** `d2c5448` — seven workflows green; 566 Rust / 42 Node / 90 Python

---

## 0. The gate is already run. Do not re-run it; do not skip reading why.

Deprecated in `2.2.2`; removed here. **You do not need to run the reverse-dependency check** — I ran it
today and it passed: ten dependents on crates.io, every published `.crate` downloaded and scanned, **zero
references** to `alloc_counter`, `CountingAllocator` or `AllocSnapshot`.

Worth knowing: the previous ruling recorded **six** dependents; there are now **ten**. Four appeared in
three weeks. That is why the roadmap said *"re-run, not cite"* — and why, **if this slice slips past another
release, the gate must be run again** rather than citing mine.

**Once handed over, this file is frozen**; changes arrive as dated addenda.

## 1. What to remove

- `pub mod alloc_counter;` from `src/lib.rs`, and the module.
- The **three** `#[allow(deprecated)]` use sites in the benchmarks and examples that exist only to silence
  it — the roadmap names them explicitly as part of this change.
- Anything in `benches/memory.rs` that depends on it.

**`benches/memory.rs` is the interesting one.** It uses `CountingAllocator` to produce the memory table on
the performance page. Removing the counter without replacing the measurement leaves RFC 012 with no memory
numbers to publish.

**So decide, and say which you chose:** either the benchmark measures peak RSS another way, or the memory
table is produced by a different method that RFC 012 can document. **If the honest answer is "the memory
table cannot be reproduced without it", stop and tell me** — that changes RFC 012, and it is better said now
than discovered when the table is due.

## 2. Criteria

1. `alloc_counter` gone from the public API. Verify with `cargo public-api` against `d2c5448` — the diff
   should show **removals only**, and nothing else removed.
2. `cargo build --no-default-features` still compiles and exposes the same function set (RFC 039 §5.2's
   existing CI step covers this — check it stays green).
3. No `#[allow(deprecated)]` left that was only there for this.
4. `benches/memory.rs` either still produces a memory measurement, or §1's escalation.
5. **This is a breaking change to the public API.** It is sanctioned — deprecated two releases ago, gate
   passed — but the CHANGELOG entry must say so plainly, under a heading a reader upgrading to `2.4.0` will
   see.
6. All three suites reported separately; `fmt`/`clippy`; counts against 566 / 42 / 90.

## 3. Then RFC 012

`rfcs/accepted/012-benchmark-hardening.md` is accepted and waits on this. Its handoff is
`rfcs/handoffs/012-benchmark-hardening/implementation-handoff.md`. **Do not start it until this lands** —
its whole point is that every published figure comes from one sitting against code that is not about to
change.

## 4. Report back

`.git-exclude/review-request/022-alloc-counter-removal/README.md`: the `cargo public-api` diff, your answer
on `benches/memory.rs`, the CHANGELOG wording, the three test counts.
