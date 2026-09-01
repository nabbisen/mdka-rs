# mdka — Roadmap

**Status.** Active — planning baseline approved by the project owner on 2026-08-02.
**Current version.** 2.2.1 (released 2026-09-01)
**Milestone progress.** M1, M1b and M2 complete. **M2b in progress** — remediation
of the independent audit of 2026-08-31. `2.2.1` shipped RFC 020; `2.2.2` carries
the rest. Then M3.
**Governance.** RFC lifecycle follows [RFC 000](./rfcs/done/000-rfc-lifecycle-policy.md).

This document is the planning baseline from which the RFC portfolio is derived.
It records milestones, sequencing, and release policy. It does not record design
decisions — those live in individual RFCs.

---

## Release policy

| Release type | Trigger | Contents | Deliverable |
|---|---|---|---|
| **Patch** `X.Y.Z+1` | Ad hoc | Docs, CI, dependency bumps, bug fixes with no API change | Tarball + CHANGELOG entry |
| **Minor** `X.Y+1.0` | One milestone completed | Additive API, new element support | Tarball + CHANGELOG + RFC dispositions |
| **Major** `X+1.0.0` | Project owner decision only | Compatibility break | Migration guide required |

### Merge policy

**Decided by the project owner, 2026-08-02: commits go directly to `main`.** No
pull-request requirement, no branch protection.

Rationale: with a single committer, pull requests buy enforcement rather than
coordination, and the friction is recurring. Detection is unaffected — CI runs
on push and reports within minutes; recovery is a `git revert` that blocks
nobody.

The accepted consequence is that **CI on `main` is advisory**. Release-time
enforcement is provided separately by RFC 014, which prevents publication from
any commit whose CI did not pass. That is where the exposure that reaches users
actually lives.

### Release mechanics

- Tag format is `X.Y.Z` — no `v` prefix (Rust crate convention).
- Release archives currently wrap their contents in a directory named after the
  asset, and carry platform and version in both that name and the filename
  (e.g. `mdka@Linux-x64-gnu-2.1.8.tar.gz` → `mdka@Linux-x64-gnu-2.1.8/mdka`).
  This **contradicts** the packaging rule in
  `.git-exclude/rules/project-instructions-rust-cli.md:57`, which forbids an
  intermediate parent directory. Recorded, not resolved: `release-executable.yaml`
  is stale and slated for replacement by the CI workflows, so the layout will be
  decided deliberately then. Until that lands, this bullet describes what the
  archives actually do. See
  `.git-exclude/reviewed/archive-layout-decision/README.md`.
- Scheduling is **sequence-based**, not date-bound. Releases are cut at logical
  breaking points — normally when a milestone's RFCs are all resolved.
- One milestone maps to one release unless the owner directs otherwise.

### Major version position

No major version transition is planned on this roadmap. RFC 005 resolves the
`ConversionOptions` defect additively, within the 2.x compatibility line. Any
future major-version decision is reserved to the project owner and is not
implied by completion of any milestone below.

### Work that is blocked on a major version — recorded, not scheduled

Raised by the audit of 2026-08-31 and deliberately **not** scheduled, because
each requires a compatibility break this roadmap does not plan.

| Item | Why it is breaking |
|---|---|
| `C-06` — give `MdkaError` path context (`Read`/`Write`/`CreateDir` variants) | Removes the `Io(#[from])` variant; downstream `match` stops compiling |
| `C-10(a)` — mark `MdkaError` `#[non_exhaustive]` | **Also breaking.** Rust classifies adding the marker to an existing enum as major (`cargo-semver-checks: enum_marked_non_exhaustive`) — an exhaustive downstream match loses its exhaustiveness |

The audit recommends the marker as free future-proofing that "costs nothing and
unblocks all future error work". That is its one substantive error: the marker
carries the same compatibility cost as the variants it was meant to enable, so
it cannot be the escape hatch from the constraint.

**The consequence matters more than the error.** The audit bundles `S-02`'s
collision reporting into this cluster as a single medium-term change, which would
park a live silent-data-loss defect behind a major version. RFC 021 therefore
fixes `S-02` **within** the existing error type, using
`io::ErrorKind::AlreadyExists`. No new variant, no marker, ships in `2.2.2`.

If a 3.0 is ever opened, these are its first candidates. Until then the error
type stays as it is, and the limitation is documented rather than worked around.

---

## Milestones

### M1 · Trustworthy baseline → `2.1.7` (patch) — ✅ COMPLETE

**Released 2026-08-02.** All five RFCs implemented, reviewed, and approved; all
four exit criteria met. Post-release evaluation and root-cause analysis of the
`verify-ci` failure are in
`.git-exclude/reviewed/release-2.1.7-complete/README.md`.

No behaviour change. Establishes the quality gate that every later milestone
lands through, and closes the documentation claims that are wrong regardless of
any pending design decision.

Sequencing within M1: **001 → 002 → 004 → 003**. RFC 003 rewrites the workspace
layout section of the architecture docs, so it must follow RFC 004's disposition
of the orphaned preprocessor.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 001 | CI quality gates | P0 | S |
| 002 | Governance artifacts | P0 | S |
| 004 | Orphaned preprocessor disposition | P0 | S |
| 003 | Architecture documentation reconciliation | P0 | S |
| 014 | Release-time CI verification | P1 | S |

RFC 014 was added mid-milestone, after the merge-policy decision above made
explicit that CI on `main` is advisory. It depends only on RFC 001 and runs
independently of 002/003/004.

**Exit criteria.** CI green on `main` with `-D warnings` enforced; no release
can publish from a commit whose CI did not pass; CHANGELOG covers every
published version; no documentation statement contradicts observed engine
behaviour except those explicitly owned by RFC 005/006.

### M1b · Release tooling completion → no release of its own

**Added 2026-08-02 by owner decision, after the `2.1.7` release.** Not part of
the originally agreed roadmap.

Four defects in release tooling surfaced during `2.1.7`: crates.io publishes
outside RFC 014's guard, GitHub release creation is manual, `version.sh` silently
misses `[workspace.dependencies]` (drifted across three releases), and the
binding crates reach crates.io by accident of scripting rather than by decision.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 015 | Release tooling completion | P1 | M |

**No release is cut for this milestone.** Workflow changes take effect at the
next release; `version.sh` takes effect immediately. Its work is exercised when
`2.2.0` ships at the end of M2.

Placed before M2 because the context is current and because every release made
without it repeats the manual sequence and the `version.sh` trap.

**Exit criteria — revised twice on 2026-08-08.** See RFC 015's two revisions
for the reasoning behind each change.

| Criterion | State |
|---|---|
| A version bump that half-applies fails loudly | ✅ Met |
| Binding-crate presence on crates.io is a recorded decision | ✅ Met |
| No registry publishes from a commit whose CI did not pass | ✅ **Met once the Trusted Publisher registrations are in place** — Slice 1 reversed, then restored |
| Cutting a release is "push a tag, then watch" | **Abandoned by decision** — Slice 2 withdrawn |

Three of four met. The remaining shortfall is a recorded decision, not a gap.

### Future candidates arising from M1b

Recorded as candidates, not plan. Neither is scheduled; both would need a fresh
RFC and owner agreement.

| Candidate | Why it was not done | What it would need |
|---|---|---|
| **Automate GitHub release creation** | `GITHUB_TOKEN`-created releases do not trigger other workflows, so the design would have published nothing. Escaping that needs a PAT (expires annually) or a GitHub App. Buys one saved command per release. | Either a non-`GITHUB_TOKEN` identity, **or** the tag-push restructure noted in RFC 015 — triggering the publishing workflows on tag push instead of release creation, which needs no credential but must resolve asset-upload ordering |
| **Release precondition checker** | Not previously considered. Automates *checking* rather than *acting* — see below. | A script or workflow asserting CI green on the commit, versions consistent across all manifests, a `CHANGELOG.md` entry for this version, and tag matching the manifest version |

**Automating crates.io publishing is no longer on this list.** It was here
briefly after Slice 1 was reversed; the owner then determined how to configure
Trusted Publishers, and RFC 015's second revision restores it. See that revision
for the sequence.

The two remaining candidates interact with each other only loosely; the
precondition checker is independent and is the one most likely worth doing.

#### When to revisit — and why frequency is the wrong trigger

Discussed with the project owner 2026-08-08. Recorded because this reasoning is
easy to lose and easy to get backwards.

**Low release frequency cuts both ways.** It is usually cited against
automation — too few repetitions to amortise setup. But it is equally an
argument *for* it: a process run twice a year is one you have forgotten by the
next time, whereas frequent releases build muscle memory.

The sharper consideration points the other way: **rarely-used automation is
untrustworthy automation.** A workflow exercised twice a year has every run as
effectively a first run. M1b produced two consecutive data points — `verify-ci`
broke on first real use, and `create-release` would have published nothing on
its first real use. Neither was caught by review; both were written carefully.

So low frequency makes manual steps less reliable *and* automated steps less
reliable. It does not cleanly favour either, and should not be the trigger.

**Better triggers, roughly in order of strength:**

1. **A second person needs to be able to release.** Automation's real value is
   encoding a process that currently lives in one person's head.
2. **A release goes wrong because of a forgotten manual step.** One occurrence
   of the empirical signal outweighs any amount of speculation.
3. **The manual checklist outgrows what fits comfortably in your head.**
   Currently: check CI, bump, tag, create release, run publish script. If later
   milestones add steps, reassess.
4. **Releases become frequent enough that the automation would be exercised
   enough to trust.** Frequency matters here — but for this reason, not because
   manual effort becomes intolerable.

**The candidate most likely to be worth doing is the third one in the table
above.** A precondition checker automates verification while leaving the
irreversible `cargo publish` manual. It captures most of the safety benefit with
none of the irreversibility risk, and it fails in the honest direction: a broken
checker is visibly broken, rather than silently approving something.

The project already has one instance of that pattern working well —
`version.sh`'s post-update assertion, which does not perform the release but
refuses to let a half-applied bump pass quietly (RFC 015 Slice 3). Extending
that shape is lower-risk than extending the publish-automation shape.

### M2 · Truth in the API surface → `2.2.0` (minor) — ✅ COMPLETE

Closes the gap between what `ConversionOptions`, the CLI, the bindings, and the
documentation promise, and what the engine actually does. Six of the eight
option fields are currently inert.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 005 | `ConversionOptions` semantics — implement attribute handling | P0 | L | ✅ 2.2.0 |
| 006 | Option documentation + binding parity realignment | P1 | M | ✅ 2.2.0 |
| 007 | English-only public surface | P1 | M | → M2b |

**Shipped 2026-08-12.** RFC 007 did not make 2.2.0 and moves to M2b.

**One exit criterion was met only inside `docs/src/api/`.** The external audit of
2026-08-31 found `docs/src/getting-started/usage-cli.md:49-51` still documenting
the three deprecated no-op flags as working, and never mentioning
`--unwrap-wrappers`. RFC 006's scope, which I wrote, named `docs/src/api/` and
never swept `getting-started/`. The defect class M2 existed to eliminate survived
one directory away. Repaired in RFC 023.

**Exit criteria.** Every public option field demonstrably changes output, with a
test per field per surface; Rust, CLI, Node, and Python expose the same option
set; no Japanese text in any artifact published to crates.io, npm, or PyPI.

#### Carried-forward finding for RFC 006

Found during RFC 003 implementation and deliberately not fixed there, since
RFC 003's scope was a fixed list of eight enumerated corrections.

| Finding | Evidence |
|---|---|
| `docs/src/api/elements.md`'s Block Elements table groups `<div>`, `<article>`, `<section>`, `<main>`, `<figure>`, `<figcaption>` into one row claiming all six are "unwrapped in Minimal/Semantic". **False for `<figure>` and `<figcaption>`** — they are never unwrapped in any mode. | `src/utils.rs::is_wrapper_tag` is `span\|div\|section\|article\|main` and excludes both; `is_structural_tag` explicitly *includes* both, which blocks unwrapping even if they were wrapper-eligible. Two disjoint source-level lists, verified at RFC 003 review. |

RFC 006 owns `unwrap_unknown_wrappers` documentation, so this row belongs to it
rather than to a standalone RFC. Note that black-box confirmation is
inconclusive here — `figcaption` triggers its own block spacing regardless of
unwrap status, so output alone cannot distinguish the two cases. The source-level
evidence is what settles it.


### M2b · Audit remediation → `2.2.1` + `2.2.2` (patches) — ⏳ IN PROGRESS

Arising from the independent audit of 2026-08-31
(`.git-exclude/reviewed/audit-2026-08-31/`, architect response in
`ARCHITECT-RESPONSE.md`). **This milestone is live user harm only.** Nothing here
is an improvement; every item is something that is currently wrong for someone
who has installed the package.

| RFC | Title | Priority | Size | Release |
|---|---|---|---|---|
| 020 | npm distribution repair + published-artifact install gate | **P0** | S | **`2.2.1`** |
| 021 | Bulk conversion output-collision safety | **P0** | S | `2.2.2` |
| 022 | Remove the counting allocator from the shipped CLI; settle `jemalloc` | P1 | S | `2.2.2` |
| 023 | Getting-started documentation reconciliation | P1 | S | `2.2.2` |
| 026 | Consumer-artifact verification gates | **P0** | M | `2.2.2` |
| 027 | Verification discipline: the consumer pass | P1 | S | `2.2.2` |
| 007 | English-only public surface (carried from M2) | P2 | M | `2.2.2` |

**Carried into `2.2.2`, added 2026-09-01 during review** — small items with no
RFC number of their own:

| Item | Source |
|---|---|
| Delete the orphaned `node/<platform>/` directories. `napi create-npm-dirs` has written to `node/npm/<platform>/` since commit `e231e1a` (2026-04-16), which dropped `--cwd .`; `version.sh`'s generic scan has been version-bumping dead files ever since. Deleting them is the fix — the scan exists for a live reason and should not be narrowed. | RFC 020 review |
| Duplicate inputs to bulk conversion are reported as a collision: `mdka -o out/ a.html a.html` errors with a message naming the same path twice, and exits 1. No data is lost and the behaviour follows RFC 021's rule exactly, but a benign idempotent input should not fail a script. Needs the source paths canonicalized to tell a duplicate from a true collision — the one place canonicalization *is* required, which RFC 021's review correctly found unnecessary for comparing destinations. Whether it should warn-and-continue rather than error is a behaviour question to settle in that slice. | RFC 021 review |

**Split into two releases, 2026-09-01.** M2b originally targeted a single
`2.2.1`. RFC 020's implementation established that the npm fix **cannot be
verified except by releasing** — no local or CI check can exercise a registry
round-trip. Shipping it alongside four unrelated changes would spend that one
observation on a noisy sample, and would keep 100% of npm users broken while
unrelated fixes travelled with it.

`2.2.1` is RFC 020 alone. `2.2.2` carries the rest.

**026 and 027 are the point of this milestone.** The other four repair what the
audit found; these two change why we did not find it. Without them M2b buys one
round of fixes and leaves the control gap that produced them — an audit is not a
process. RFC 026 gates on the artifact a user installs; RFC 027 puts a reviewer in
the consumer's position before each release.

**Why a patch and not a minor.** Every change is a defect repair. RFC 020 and 021
add no API. RFC 022 removes an allocator that was never a documented feature.
RFC 023 is documentation. Nothing here is additive, so patch releases are correct.

**Sequencing constraint.** RFC 020's install gate lands **before** its fix, so
the gate is observed failing against the broken package and passing after. A gate
that has only ever been seen green proves nothing — the lesson from `verify-ci`
in M1b, now applied to the artifact rather than the pipeline.

**Exit criteria.** `npm install mdka@2.2.1 && node -e "require('mdka')"` succeeds
in a clean directory on every published platform — this one closes with `2.2.1`;
the rest close with `2.2.2`: a CI job performs exactly that
against the packed tarball and fails if it cannot; converting two files with
colliding output stems reports an error for the loser instead of silently
discarding it; no getting-started page documents a no-op as working; **every
published artifact — npm tarball, PyPI wheel, packaged crate — is installed from
outside the workspace and exercised by CI**; and **a consumer pass has been
performed against the released `2.2.2` and recorded**.

The last two are the ones that matter beyond this release. The first four would
leave us exactly where we were on 2026-08-30: correct, and unable to tell.

### M3 · Conversion fidelity → `2.3.0` (minor)

Purely additive element coverage. Tables are the largest known gap against the
project's GFM positioning; today `<table>` content is emitted as an unstructured
text run.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 025 | Markdown output-validity harness | **P0** | M |
| 024 | Inline composition: route every writer through the output sink | **P0** | M |
| 010 | Escaping & text-processing correctness audit | P0 | M |
| 008 | GFM table support | P1 | L |
| 009 | Element coverage extension (`dl`/`dt`/`dd`, `del`/`s`, `sup`/`sub`) | P2 | M |

**Reordered by the 2026-08-31 audit.** Tables were the largest *known* gap; the
audit found the larger *unknown* one. `mdka` produces invalid Markdown for
several everyday constructs — a linked image, bold inside a link, a bare `<pre>`,
a code span containing `_`. Emitting a correct table matters less than emitting
correct output for HTML that is already in scope, so 024/025/010 precede 008.

**RFC 025 lands first, and is the reason the rest are findable.** 136 tests were
green while all of this shipped, because no test parses mdka's output as
Markdown — every renderer assertion compares against a string we wrote ourselves.
A suite authored by the same hand as the renderer cannot discover that the
renderer's output is not Markdown.

**RFC 010 is now populated** by the audit: `A-03` (escaping inside code spans),
`A-04` (unescaped destinations and titles), `A-05` (fixed-width fences), `A-09`
(line-leading digits), `A-10`, `A-11`, and `D-05`. It no longer needs to start
from a blank survey.

**Exit criteria.** Every construct in the composition matrix round-trips through
a CommonMark parser to the structure mdka intended; tables round-trip to GFM pipe
syntax including alignment and header rows; each rule in
`docs/src/api/text-processing.md` is confirmed or corrected against a test.

### M4 · Durability → `2.4.0` (minor)

| RFC | Title | Priority | Size |
|---|---|---|---|
| 011 | Robustness: fuzzing + `MdkaError::Io` error-path tests | P2 | M |
| 012 | Benchmark hardening + regenerate published performance claims | P2 | M |
| 013 | Internal comment migration to English | P2 | L |

RFC 013 is a large, purely mechanical diff. It is scheduled into a quiet release
deliberately, so it does not bury substantive changes in `git blame`.

#### Carried-forward review findings

Deferred here by owner decision on 2026-08-02 rather than reopening completed
RFCs. None is a correctness issue; all are hygiene. Recorded so they are not
lost — a deferred item tracked nowhere is an abandoned item.

| ID | Finding | Source | Suggested home |
|---|---|---|---|
| R-01 | `mdka-cli`, `mdka-node`, `mdka-python` declare no `rust-version`. Only the root `mdka` package does. Already covered transitively — `mdka-cli` depends on `mdka`, whose declaration propagates through the resolver — so this is tooling and crates.io display hygiene. Cleanest fix is `[workspace.package] rust-version` plus `rust-version.workspace = true` in each member. | RFC 001 review, round 2 | New slice, or fold into RFC 012 |
| R-02 | **Promoted 2026-08-12 from hygiene to a shipped defect.** The `node` CI job checks `node/index.d.ts` for drift but not `node/index.js`, though `package.json` publishes both and `npm run build` regenerates both. **The drift is now confirmed and live:** tracked `node/index.js` hardcodes `expected 2.0.2` in four napi-rs binding version checks while `package.json` is `2.1.8` — pinned six releases back and shipping to npm. Inert by default (gated on `NAPI_RS_ENFORCE_VERSION_CHECK`), but any consumer who sets that variable gets `mdka-node` throwing *"expected 2.0.2 but got 2.1.8"* on load, with a reinstall suggestion that cannot help. Fix needs its own slice: regenerate deliberately, review the full diff (output depends on the local napi-rs), add `node/index.js` to `version.sh`'s post-update assertions, and extend the CI drift check to cover it. | RFC 001 review round 2; confirmed in RFC 005 Slice B1 round 2 | **RFC 006 Slice C** — adding a Node option forces `npm run build`, which regenerates `node/index.js` anyway, so the fix is done there deliberately rather than as a separate slice. Extending the CI drift check and `version.sh` assertions stays in M4. |
| R-04 | **`release-npm.yaml`'s internal tag check — (a) and (c) FIXED 2026-08-12, (b) deferred.** (a) ~~Its rc pattern lacked a `+` on the patch component, so any release candidate with a two-digit patch silently skipped the npm publish~~ — fixed; `2.1.10-rc.1` now passes, verified by running the extracted step. (c) ~~The rc asymmetry with `create-release.yaml`'s filter was an accident~~ — now a recorded decision: **tag-push automation is for final releases only; release candidates are cut by creating the GitHub release by hand**, which still fans out via `release: created`. Documented in a comment naming both checks. (b) **Still open:** the intentional skip path is `exit 1`, so a deliberate skip and a genuine npm failure are both red. Fixing it properly means gating the `publish` job's remaining six steps behind a step output, or splitting the check into its own job — a structural change to the workflow that publishes to npm, deliberately not made immediately before the 2.2.0 release. The skip now prints an unmistakable `::warning::` and an explicit "this is not an npm publishing failure", so the log resolves it in one click. | RFC 019 dry run, 2026-08-12 | (b): small slice, after 2.2.0 |
| R-03 | `node/test.js` terminates the whole process on first failure — three concurrently-started async IIFEs share counters, and whichever finishes first with `failed > 0` calls `process.exit(1)`. Demonstrated: a single broken assertion left 5 of 35 tests unrun. The gate's exit code is correct, so CI still fails; diagnosis is what degrades. | RFC 001 review, round 2 | RFC 011 |

R-01 and R-02 originate in the architect's specification, not in implementation
work.

**Exit criteria.** Fuzz target runs clean for a defined budget; every performance
figure in `docs/src/design/` regenerated from a current benchmark run; no
Japanese text remains in `src/`, `cli/`, `node/`, or `python/`.

---

## Portfolio at a glance

| RFC | Title | Milestone | Priority | Size | Depends on |
|---|---|---|---|---|---|
| 001 | CI quality gates | M1 | P0 | S | — |
| 002 | Governance artifacts | M1 | P0 | S | — |
| 003 | Architecture documentation reconciliation | M1 | P0 | S | 004 |
| 004 | Orphaned preprocessor disposition | M1 | P0 | S | 001 |
| 005 | `ConversionOptions` semantics | M2 | P0 | L | 001, 004 |
| 006 | Option docs + binding parity | M2 | P1 | M | 005 |
| 007 | English-only public surface | M2 | P1 | M | — |
| 008 | GFM table support | M3 | P0 | L | 001 |
| 009 | Element coverage extension | M3 | P2 | M | 008 |
| 010 | Escaping & text-processing audit | M3 | P1 | M | 001 |
| 011 | Robustness: fuzzing + I/O error paths | M4 | P2 | M | 001 |
| 012 | Benchmark hardening | M4 | P2 | M | 008 |
| 013 | Internal comment migration to English | M4 | P2 | L | 007 |
| 014 | Release-time CI verification | M1 | P1 | S | 001 |
| 015 | Release tooling completion | M1b | P1 | M | 014 |

Numbers are permanent and never reused, per RFC 000. Numbers 005–013 are
reserved; those RFCs are drafted at the start of their milestone rather than up
front, so their design reflects what the preceding milestone actually shipped.

---

## Roadmap maintenance

At every RFC disposition point (implemented, withdrawn, superseded, deferred)
the following is reviewed and any material change reported to the project owner:

- RFC portfolio and milestone progress
- Whether roadmap assumptions still hold
- Dependency and risk changes
- Whether new RFCs are required, or existing ones should be split or merged

When all milestones are resolved, or no substantial development theme remains,
that state is reported to the project owner and joint replanning resumes. The
roadmap is not extended unilaterally.
