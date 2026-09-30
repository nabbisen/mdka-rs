//! Allocation gate — the one performance property this project has ever
//! measured reliably.
//!
//! **Why allocation, and not wall time.** This project tried wall-time gates
//! three times over (`rfcs/handoffs/3.0.0-bulk-regression/`,
//! `rfcs/handoffs/perf-method-calibration/`) and cancelled all of them: zero
//! performance defects in this project's history were ever caught by a timing
//! measurement, the noise floor on a shared, ordinary machine is fat-tailed,
//! and a "confirmed quiet" sitting still failed to reproduce its own finding a
//! second time. Allocation does not have this problem — it is a property of
//! the code path taken, not of the CPU scheduler, and it does not vary with
//! machine, load, or which process happens to share a core.
//!
//! **Measured, not assumed — twice, because the first measurement was wrong.**
//! A standalone `--release` probe showed zero variance over 60 separate
//! process invocations per workload, and that number was committed here
//! first. It was wrong for *this* file: `ci.yaml` runs `cargo test` without
//! `--release` (debug profile), and debug-mode codegen allocates a different
//! byte count for the bulk workload than release mode does (the release-mode
//! parallel-dispatch machinery inlines away some bookkeeping that debug mode
//! does not). Worse, the first version of this file ran both workloads as two
//! separate `#[test]` functions, which `cargo test` runs concurrently on
//! separate threads **by default** — and `CountingAllocator`'s counters
//! (`benches/alloc_counter.rs`) are process-global, not per-thread, so each
//! test's "before/after" snapshot window was catching allocations made by the
//! *other* test running at the same time on another thread. That produced
//! wildly non-deterministic numbers (string conversion swung between roughly
//! 110 KB and 157 KB across five consecutive runs) that looked exactly like
//! "the bulk workload's variance can't be separated from the signal" — the
//! outcome §3 of the handoff explicitly allows for — right up until running
//! with `--test-threads=1` made both workloads perfectly stable, which is
//! what exposed the real cause. **The fix is structural, not a flag this
//! project cannot pass from outside `cargo test --workspace`:** both
//! workloads are measured inside **one** `#[test]` function, sequentially, so
//! there is no second test in this binary to race against and no thread-count
//! flag for CI to remember.
//!
//! **Re-measured correctly, under the real conditions** (debug profile,
//! single test, `cargo test --test allocation_gate` run repeatedly as
//! separate process invocations, matching what a CI job actually does):
//!
//! - **String conversion: zero variance.** One value, every one of 260+
//!   repeats. Tolerance **0** — any deviation at all is either a real change
//!   or a change in the measurement itself.
//! - **Bulk file conversion: a small, bounded, bimodal spread, not
//!   unbounded noise.** 260+ repeats produced exactly **two** distinct
//!   values, 544 B apart — the common one 148/150 times in the largest
//!   single batch (~98.7%), the lower one the rest — never a third value,
//!   never a wider spread. This is consistent with a discrete, one-time
//!   cost (plausibly a per-thread initialization allocation inside rayon's
//!   pool that only happens when a given worker thread is used for the
//!   first time in a process, which depends on how the 5-file workload
//!   happens to get scheduled across it) rather than with genuinely
//!   unbounded scheduling noise. **Tolerance 600 B**: comfortably above the
//!   observed 544 B step with margin, and comfortably below the 1764 B real
//!   signal measured against `2.9.0` (below) — a ~2.9× separation between
//!   tolerance and signal.
//!
//! **Validated against the change this exists for.** The same two workloads,
//! same allocator, same `cargo build` (debug) profile, run against the
//! published `2.9.0` source (`Vec<(&P, Result<PathBuf, MdkaError>)>` — a
//! borrowed return — instead of `3.0.0`'s owned `Vec<FileOutcome>`), from a
//! standalone probe crate (`2.9.0`'s different API shape means the literal
//! test in this file cannot compile against it, so this is the same
//! measurement logic against a different dependency, not this exact file):
//!
//! | | `2.9.0` | current `HEAD` | diff | vs. tolerance |
//! |---|---:|---:|---:|---|
//! | string | 10507 B | 10319 B | 188 B | tolerance is 0 — **would fail** |
//! | bulk | 271633 B | 273397 B | 1764 B | tolerance is 600 — **would fail** |
//!
//! Both workloads separate `2.9.0` from current `HEAD` well outside their
//! tolerances — the string difference `188 B` is itself `2.9.0`'s
//! allocation being *smaller* than current `HEAD`'s common value even though
//! the gate's own contribution here is RFC 049's internal rewrite, not the
//! `FileOutcome` change this gate was proposed for (that change is isolated
//! to the bulk path only). A baseline is a **number**, not a claim about
//! which version should be bigger.
//!
//! **Changing a baseline is a decision, not a fix** — the same discipline
//! `tests/output_validity/mode_identity.rs`'s P3 goldens already use. A commit
//! that changes what either workload allocates must change the number below
//! in that same commit, with the reason stated in the commit message. Editing
//! this file to make a failing run pass without that reason is exactly the
//! failure this gate exists to prevent.
//!
//! **Mechanics.** `#[global_allocator]` applies to the whole test binary, so
//! this lives in its own file rather than alongside any other integration
//! test — and, per the above, only one `#[test]` function may ever live in
//! it. `benches/alloc_counter.rs` is reached via `#[path = ...]`, the same
//! `examples/`→`benches/` coupling this project already flagged
//! (`project_alloc_counter_removed_rfc012_unblocked`) and did not fix —
//! moving the allocator is a separate decision, not worth making for this
//! one include.

#[path = "../benches/alloc_counter.rs"]
mod alloc_counter;
use alloc_counter::{AllocSnapshot, CountingAllocator};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

use mdka::options::{ConversionMode, ConversionOptions};
use std::path::PathBuf;

/// Fixed document for the string-conversion workload: headings, inline
/// emphasis, a link, and a list — enough surface to be a real measurement,
/// small enough that the byte count is a value, not a benchmark.
const STRING_HTML: &str = "<h1>Title</h1><p>A <b>bold</b> and <i>italic</i> paragraph with a <a href=\"https://example.com\">link</a>.</p><ul><li>one</li><li>two</li></ul>";

/// Measured under `cargo test`'s own debug profile, single test in this
/// binary, repeated `cargo test --test allocation_gate` invocations: zero
/// variance. (Coincides exactly with the release-mode probe's number for
/// this workload; the two profiles diverge on the bulk workload below, not
/// this one.)
const STRING_BASELINE_BYTES: usize = 10319;
const STRING_TOLERANCE_BYTES: usize = 0;

/// Fixed small document for the bulk workload, written to a handful of files.
const BULK_HTML: &str = "<h2>File</h2><p>Some <b>content</b> for a bulk conversion probe.</p>";
const BULK_FILE_COUNT: usize = 5;

/// Measured under `cargo test`'s own debug profile: the common value across
/// 260+ repeated `cargo test --test allocation_gate` invocations (~98.7% of
/// runs; see the module doc for the rare second value and why 600 B below
/// covers it). Differs from a `--release` probe's 270853 B by 2544 B —
/// debug-mode codegen for the parallel dispatch path, not measurement noise.
const BULK_BASELINE_BYTES: usize = 273397;
const BULK_TOLERANCE_BYTES: usize = 600;

fn assert_within_tolerance(
    errors: &mut Vec<String>,
    workload: &str,
    measured: usize,
    baseline: usize,
    tolerance: usize,
) {
    let diff = measured.abs_diff(baseline);
    if diff > tolerance {
        errors.push(format!(
            "{workload} allocation moved: measured {measured} B, baseline {baseline} B, \
             diff {diff} B (tolerance {tolerance} B). If this change is intentional, update \
             the baseline in this file in the same commit, with the reason in the commit message."
        ));
    }
}

/// Both workloads, measured in one test function so they cannot be scheduled
/// onto separate threads of the same process — see the module doc for why
/// that matters here specifically.
#[test]
fn allocation_is_unchanged() {
    let mut errors = Vec::new();

    let opts = ConversionOptions::for_mode(ConversionMode::Balanced);
    let before = AllocSnapshot::now();
    let md = std::hint::black_box(mdka::html_to_markdown_with(
        std::hint::black_box(STRING_HTML),
        &opts,
    ));
    let after = AllocSnapshot::now();
    std::hint::black_box(&md);
    let string_measured = after.delta_since(&before).allocated_bytes;
    assert_within_tolerance(
        &mut errors,
        "string conversion",
        string_measured,
        STRING_BASELINE_BYTES,
        STRING_TOLERANCE_BYTES,
    );

    let dir = std::env::temp_dir().join("mdka_allocation_gate_in");
    let out_dir = std::env::temp_dir().join("mdka_allocation_gate_out");
    std::fs::create_dir_all(&dir).unwrap();
    let paths: Vec<PathBuf> = (0..BULK_FILE_COUNT)
        .map(|i| {
            let p = dir.join(format!("f{i}.html"));
            std::fs::write(&p, BULK_HTML).unwrap();
            p
        })
        .collect();
    let _ = std::fs::remove_dir_all(&out_dir);

    let before = AllocSnapshot::now();
    let results = std::hint::black_box(mdka::html_files_to_markdown(&paths, &out_dir));
    let after = AllocSnapshot::now();
    std::hint::black_box(&results);
    assert_eq!(results.len(), BULK_FILE_COUNT);
    assert!(results.iter().all(|o| o.result.is_ok()));
    let bulk_measured = after.delta_since(&before).allocated_bytes;

    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&out_dir).unwrap();

    assert_within_tolerance(
        &mut errors,
        "bulk file conversion",
        bulk_measured,
        BULK_BASELINE_BYTES,
        BULK_TOLERANCE_BYTES,
    );

    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}
