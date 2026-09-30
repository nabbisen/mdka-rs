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
//! **Attempt 4, wrong, and this is the one that shipped and went red on
//! CI: `std::process::id()` was added to the fixture directory names to
//! fix a real, different bug (two concurrent runs on one machine deleting
//! each other's files) — but it made the *measured call's own paths*
//! longer or shorter depending on how many digits the pid happened to
//! have, and `FileOutcome::src` clones that path.** This project's own CI
//! runner had a 4-digit pid where the machine this was measured on had a
//! 6-digit one. Swept by overriding the suffix length directly:
//!
//! | suffix chars | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
//! |---|---:|---:|---:|---:|---:|---:|---:|
//! | bulk measured (B) | 34005 | 34045 | 34085 | 34125 | 34165 | 34205 | 34245 |
//!
//! **Exactly +40 B per character** (confirmed independently on this
//! machine too, same slope, different absolute baseline). Not noise, not
//! the runner, not the compiler: `PathBuf`'s own allocation size tracks the
//! string length of the path it holds, and that path included the fixture
//! root's full name.
//!
//! **The fix: enter the fixture root with `std::env::set_current_dir`
//! before building any path the measured call will see, and use short,
//! fixed-length relative paths (`"in/f0.html"`, `"out"`) for the
//! conversion itself.** The root's own absolute path (still pid-suffixed,
//! still solving the original concurrent-run collision) is only ever used
//! to create and later remove it — never passed to
//! `html_files_to_markdown`, so its length cannot reach the measurement.
//! Same suffix-length sweep, fixed:
//!
//! | suffix chars | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
//! |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
//! | bulk measured (B) | 32905 | 32905 | 32905 | 32905 | 32905 | 32905 | 32905 | 32905 | 32905 |
//!
//! **Constant at every length tried**, and re-confirmed varying `TMPDIR`
//! itself (4 lengths from empty to 20 characters, crossed with
//! `RAYON_NUM_THREADS` 1/4/32, 10 repeats each — 120 runs, zero failures,
//! one value), since a longer `TMPDIR` would have reached the measurement
//! exactly the way a longer pid did.
//!
//! **Three environment dependences found in three rounds — thread count,
//! toolchain, path length — each found only by running somewhere new, never
//! by reasoning about the code in advance.** A zero-tolerance allocation
//! baseline is a claim about an environment, not just about a library
//! version, and the only real proof of "machine-independent" is running it
//! in a genuinely different one. Every sweep in this file is evidence of
//! exactly one such attempt; there is no reason to believe this is the last
//! one a sufficiently different environment could find, only that these four
//! are now closed.
//!
//! **Validated against the change this exists for, using the same warmed,
//! fixed-length-path methodology (both probes, 64-file warm-up, relative
//! paths from a `set_current_dir` root)**, against the published `2.9.0`
//! source (`Vec<(&P, Result<PathBuf, MdkaError>)>` — a borrowed return —
//! instead of `3.0.0`'s owned `Vec<FileOutcome>`; `2.9.0`'s different API
//! shape means the literal test in this file cannot compile against it, so
//! this is the same measurement logic against a different dependency, not
//! this exact file — both sides are standalone probes, not `cargo test`
//! binaries, since a `cargo test` binary's own harness overhead is a
//! separate, fixed offset that does not change what either version of the
//! library itself allocates):
//!
//! | | `2.9.0` | current `HEAD` | diff |
//! |---|---:|---:|---:|
//! | string | 10507 B | 10319 B | 188 B |
//! | bulk, warm, fixed-length paths | 33715 B | 32905 B | **810 B** |
//!
//! **810 B over 5 files is 162 B/file — squarely inside the `+160`–
//! `167 B/file` this gate exists to catch** (first measured during the
//! `3.0.0` benchmark regeneration, independent of this file), now visible
//! cleanly because neither pool-construction noise nor path-length noise
//! swamps it. The string difference (`188 B`) is RFC 049's internal
//! rewrite, unrelated to the `FileOutcome` change this gate targets — both
//! differences are real, both would fail a tolerance of 0, and a baseline
//! is a **number**, not a claim about which version should be bigger.
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
//! one include. The fixture *root's* directory name is suffixed with
//! `std::process::id()` so two concurrent runs on one machine cannot delete
//! each other's files mid-test — but nothing under it, and nothing passed to
//! the measured call, carries that suffix or the root's own absolute path
//! (see attempt 4 above for why that distinction is load-bearing, not
//! stylistic).

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
/// construction cost outside the snapshot window, **and** after entering a
/// fixture root via `set_current_dir` so every path the measured call sees
/// is a short, fixed-length relative path (see the module doc for both
/// fixes and the sweeps that prove each one): zero variance across every
/// `RAYON_NUM_THREADS` value from 1 to 32 and every fixture-path length
/// tried, measured against this exact compiled test binary.
const BULK_BASELINE_BYTES: usize = 32905;
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

    // A fixed-length root, entered via set_current_dir, so every path this
    // process's own measured call sees is a short, constant-length relative
    // path ("in/f0.html", "out") regardless of this root's own absolute
    // length (which varies with the pid and with `TMPDIR` — see the module
    // doc for why that varied the measurement by +40 B/character before this
    // fix). The root's own absolute path is never passed to the measured
    // call, only used to create/remove it.
    let pid = std::process::id();
    let root = std::env::temp_dir().join(format!("mdka_allocation_gate_{pid}"));
    let original_cwd = std::env::current_dir().unwrap();
    std::fs::create_dir_all(root.join("in")).unwrap();
    std::env::set_current_dir(&root).unwrap();
    let out_dir = std::path::Path::new("out");

    // Warm-up call over WARMUP_FILE_COUNT files, discarded: pays rayon's
    // one-time global thread-pool construction cost, and uses enough files
    // that every worker thread rayon might spin up (not just as many as the
    // measured workload itself needs) gets its own one-time per-thread
    // first-use cost paid here too — see the module doc for why 5 files
    // alone were not enough to make this reliable at every thread count.
    let warm_paths: Vec<PathBuf> = (0..WARMUP_FILE_COUNT)
        .map(|i| {
            let p = std::path::Path::new("in").join(format!("warm{i}.html"));
            std::fs::write(&p, BULK_HTML).unwrap();
            p
        })
        .collect();
    let warm = std::hint::black_box(mdka::html_files_to_markdown(&warm_paths, out_dir));
    std::hint::black_box(&warm);
    let _ = std::fs::remove_dir_all(out_dir);

    let paths: Vec<PathBuf> = (0..BULK_FILE_COUNT)
        .map(|i| {
            let p = std::path::Path::new("in").join(format!("f{i}.html"));
            std::fs::write(&p, BULK_HTML).unwrap();
            p
        })
        .collect();
    let _ = std::fs::remove_dir_all(out_dir);

    let before = AllocSnapshot::now();
    let results = std::hint::black_box(mdka::html_files_to_markdown(&paths, out_dir));
    let after = AllocSnapshot::now();
    std::hint::black_box(&results);
    assert_eq!(results.len(), BULK_FILE_COUNT);
    assert!(results.iter().all(|o| o.result.is_ok()));
    let bulk_measured = after.delta_since(&before).allocated_bytes;

    std::env::set_current_dir(&original_cwd).unwrap();
    std::fs::remove_dir_all(&root).unwrap();

    assert_within_tolerance(
        &mut errors,
        "bulk file conversion",
        bulk_measured,
        BULK_BASELINE_BYTES,
        BULK_TOLERANCE_BYTES,
    );

    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}
