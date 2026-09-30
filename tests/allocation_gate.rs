//! Allocation gate — the one performance property this project has ever
//! measured reliably.
//!
//! **Couples to RFC 049.** This file sits on commit `96c9a28` (RFC 049,
//! inline `style` emphasis), and the string baseline below already includes
//! its rewrite of the emphasis-tracking data structures. The two land
//! together, in that order — this file is not meaningful checked out on top
//! of a commit that predates RFC 049.
//!
//! **Why allocation, and not wall time.** This project tried wall-time gates
//! three times over (`rfcs/handoffs/3.0.0-bulk-regression/`,
//! `rfcs/handoffs/perf-method-calibration/`) and cancelled all of them: zero
//! performance defects in this project's history were ever caught by a timing
//! measurement, the noise floor on a shared, ordinary machine is fat-tailed,
//! and a "confirmed quiet" sitting still failed to reproduce its own finding a
//! second time. Allocation does not have this problem *once it is measured
//! correctly* — see below for what "correctly" took three attempts to reach.
//!
//! **Attempt 1, wrong: release-mode numbers under a debug-mode CI.** A
//! standalone `--release` probe showed zero variance over 60 separate
//! process invocations per workload. `ci.yaml` runs `cargo test` with no
//! `--release` flag, and debug-mode codegen allocates differently.
//!
//! **Attempt 2, wrong: two `#[test]` functions racing on one process-global
//! counter.** `cargo test` runs multiple tests in one binary concurrently, on
//! separate threads, by default; `CountingAllocator`'s counters
//! (`benches/alloc_counter.rs`) are process-global atomics, not per-thread.
//! Each test's before/after window was catching the *other* test's
//! allocations. Fixed by moving both workloads into **one** `#[test]`
//! function — correct, and still true below.
//!
//! **Attempt 3, wrong in a way review caught and attempt 2's own fix could
//! not: the bulk measurement window opened on the *first*
//! `html_files_to_markdown` call in the process, so it counted rayon's
//! entire global thread-pool construction, not just the conversion.** This
//! is a one-time, per-process cost that scales with worker-thread count —
//! roughly **7 KB per thread** — so the "baseline" was really "conversion
//! plus however many workers this machine happens to spin up." Measured
//! cold (no warm-up call before the snapshot), debug profile, this project's
//! own machine (32 logical CPUs) against `ubuntu-latest` (4 vCPUs) and every
//! point between:
//!
//! | `RAYON_NUM_THREADS` | 1 | 2 | 4 (`ubuntu-latest`) | 8 | 16 | 32 (this machine) |
//! |---|---:|---:|---:|---:|---:|---:|
//! | bulk, cold (B) | 46126 | 53190 | 67318 | 95958 | 154423 | 270615 |
//!
//! **A gate built on the cold number passes only on a machine with as many
//! threads as it happened to be measured on, and goes red on
//! `ubuntu-latest` on the first push.** The "bimodal 544 B step" an earlier
//! version of this header attributed to per-thread first-touch noise *worth
//! tolerating* was this same cause, correctly identified as thread
//! initialisation and then wrongly treated as an irreducible cost to widen
//! the tolerance around, rather than as setup to warm past.
//!
//! **The fix: one warm-up call before the snapshot, discarded**, paying the
//! pool-construction cost once, outside the measured window. A first version
//! of this fix warmed with the *same 5 files* the measured call uses, which
//! removed the 7 KB/thread scaling above but left a smaller, rarer residual:
//! about 1 run in 300 (observed once at 8 threads, once at 2, different
//! magnitudes each time — not a fixed step) still measured several KB high.
//! **Cause: 5 files may not exercise every worker thread rayon's pool spins
//! up**, so the warm-up call can touch a different, smaller subset of
//! threads than the later measured call happens to land on — the *pool* was
//! warm, but not every *thread in it*. Fixed by warming with
//! `WARMUP_FILE_COUNT` (64) files, more than any realistic thread count, so
//! the warm-up call is overwhelmingly likely to touch every thread the
//! measured call could possibly use. Same sweep, warmed with 64 files:
//!
//! | `RAYON_NUM_THREADS` | 1 | 2 | 4 | 8 | 16 | 32 |
//! |---|---:|---:|---:|---:|---:|---:|
//! | bulk, warm (B) | 34205 | 34205 | 34205 | 34205 | 34205 | 34205 |
//!
//! **Identical at every thread count — machine-independent, which is what
//! the module's opening claim requires and did not, before this, have
//! evidence for.** Re-confirmed over 300 repeated process invocations (50 at
//! each of the six thread counts above, this exact compiled test binary):
//! zero variance, zero failures, one value. `RAYON_NUM_THREADS` does not
//! affect the string workload either (checked at 1, 4, and 32 threads:
//! `10319` B every time), which never touches rayon.
//!
//! **Validated against the change this exists for, using the same warmed
//! methodology (both probes, 64-file warm-up)**, against the published
//! `2.9.0` source (`Vec<(&P, Result<PathBuf, MdkaError>)>` — a borrowed
//! return — instead of `3.0.0`'s owned `Vec<FileOutcome>`; `2.9.0`'s
//! different API shape means the literal test in this file cannot compile
//! against it, so this is the same measurement logic against a different
//! dependency, not this exact file — both sides are standalone probes, not
//! `cargo test` binaries, since a `cargo test` binary's own harness
//! overhead is a separate, fixed offset that does not change what either
//! version of the library itself allocates):
//!
//! | | `2.9.0` | current `HEAD` | diff |
//! |---|---:|---:|---:|
//! | string | 10507 B | 10319 B | 188 B |
//! | bulk, warm | 34605 B | 33845 B | **760 B** |
//!
//! **760 B over 5 files is 152 B/file — in the same range as the `+160`–
//! `167 B/file` this gate exists to catch** (first measured during the
//! `3.0.0` benchmark regeneration, independent of this file), now visible
//! cleanly because pool-construction noise no longer swamps it. The string
//! difference (`188 B`) is RFC 049's internal rewrite, unrelated to the
//! `FileOutcome` change this gate targets — both differences are real, both
//! would fail a tolerance of 0, and a baseline is a **number**, not a claim
//! about which version should be bigger.
//!
//! **Tolerances, re-derived from the warmed spread: both 0.** Neither
//! workload showed any variance once measured correctly — not "small
//! enough to tolerate," actually zero across every repeat and every thread
//! count tried. Any deviation at all is either a real change or a change in
//! the measurement itself.
//!
//! **Changing a baseline is a decision, not a fix** — the same discipline
//! `tests/output_validity/mode_identity.rs`'s P3 goldens already use. A
//! commit that changes what either workload allocates must change the
//! number below in that same commit, with the reason stated in the commit
//! message. Editing this file to make a failing run pass without that
//! reason is exactly the failure this gate exists to prevent.
//!
//! **Mechanics.** `#[global_allocator]` applies to the whole test binary, so
//! this lives in its own file rather than alongside any other integration
//! test — and only one `#[test]` function may ever live in it, per attempt 2
//! above. `benches/alloc_counter.rs` is reached via `#[path = ...]`, the
//! same `examples/`→`benches/` coupling this project already flagged
//! (`project_alloc_counter_removed_rfc012_unblocked`) and did not fix —
//! moving the allocator is a separate decision, not worth making for this
//! one include. Fixture directories are suffixed with `std::process::id()`
//! so two concurrent runs on one machine cannot delete each other's files
//! mid-test.

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

/// Measured under `cargo test`'s own debug profile: zero variance across
/// every repeat and every `RAYON_NUM_THREADS` value tried (this workload
/// never touches rayon, so thread count is not expected to matter, and
/// measurement confirmed it does not).
const STRING_BASELINE_BYTES: usize = 10319;
const STRING_TOLERANCE_BYTES: usize = 0;

/// Fixed small document for the bulk workload, written to a handful of files.
const BULK_HTML: &str = "<h2>File</h2><p>Some <b>content</b> for a bulk conversion probe.</p>";
const BULK_FILE_COUNT: usize = 5;

/// Files used only for the discarded warm-up call, not the measured one.
/// Larger than any thread count this gate is likely to run under, so every
/// worker thread rayon spins up gets touched during warm-up rather than
/// possibly for the first time during the measured call — see the module
/// doc for the residual flake this specifically fixes.
const WARMUP_FILE_COUNT: usize = 64;

/// Measured **after** a warm-up call that pays rayon's one-time pool
/// construction cost outside the snapshot window (see the module doc for
/// why this matters and the thread-count sweep that proves it): zero
/// variance across every `RAYON_NUM_THREADS` value from 1 to 32, measured
/// against this exact compiled test binary (a standalone probe crate with
/// the same logic measures 400 B lower — `cargo test`'s own harness
/// overhead, fixed and consistent, not noise; the baseline below is from
/// this file's own binary, not the probe). Without the warm-up call, this
/// number grows by roughly 7 KB per worker thread and the gate is not
/// portable across machines.
const BULK_BASELINE_BYTES: usize = 34205;
const BULK_TOLERANCE_BYTES: usize = 0;

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

    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("mdka_allocation_gate_in_{pid}"));
    let out_dir = std::env::temp_dir().join(format!("mdka_allocation_gate_out_{pid}"));
    std::fs::create_dir_all(&dir).unwrap();

    // Warm-up call over WARMUP_FILE_COUNT files, discarded: pays rayon's
    // one-time global thread-pool construction cost, and uses enough files
    // that every worker thread rayon might spin up (not just as many as the
    // measured workload itself needs) gets its own one-time per-thread
    // first-use cost paid here too — see the module doc for why 5 files
    // alone were not enough to make this reliable at every thread count.
    let warm_paths: Vec<PathBuf> = (0..WARMUP_FILE_COUNT)
        .map(|i| {
            let p = dir.join(format!("warm{i}.html"));
            std::fs::write(&p, BULK_HTML).unwrap();
            p
        })
        .collect();
    let warm = std::hint::black_box(mdka::html_files_to_markdown(&warm_paths, &out_dir));
    std::hint::black_box(&warm);
    let _ = std::fs::remove_dir_all(&out_dir);

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
