# mdka — Roadmap

**Status.** Active — planning baseline approved by the project owner on 2026-08-02.
**Current version.** 2.3.0 (prepared 2026-09-22; awaiting the pre-tag checkpoint)
**Current version note.** `2.2.1` shipped RFC 020; `2.2.2` shipped RFC 007, 021,
022, 023, 026 and 027; `2.2.3` shipped RFC 029; **`2.3.0` ships RFC 010, 024, 025,
028, 030–035** — output validity, and the control repairs that made it measurable.
**Milestone progress.** M1, M1b, M2, M2b and M2c complete. **M3 complete** — all ten RFCs implemented and approved, prep done, version bumped; awaiting the pre-tag checkpoint. **M4 (`2.4.0`) is next**, starting with RFC 012.
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

### Scheduled surface removal inside v2 — owner decision, 2026-09-01

**`mdka::alloc_counter` is deprecated in `2.2.2` and removed in `2.4.0`.**

Removing public API in a 2.x release is formally a major change. The owner owns
the version contract and ruled deliberately: deferring to a 3.0 that is not
planned risks meaning "never", and a dated removal is better governance than an
open-ended deferral. Recorded here as a scheduled break, not an oversight.

What makes it responsible rather than merely decided:

- **Two releases and one full milestone of deprecation notice**, with a
  CHANGELOG entry at both ends.
- **Zero known consumers, measured not estimated.** All six crates.io dependents
  (`htm_md`, `bigquery-functions`, `elvish-core`, `threadcat`, `htmlmd-core`,
  `zapmyco-tools`) were downloaded and grepped: none references
  `alloc_counter`, `CountingAllocator` or `AllocSnapshot`. Every use of `mdka`
  across all six is `from_html`, `html_to_markdown`, `html_to_markdown_with` or
  `options`.
- **A required gate at removal time** — see M4.

This is the only scheduled break in the 2.x line. It is not a precedent for
removing documented API; `alloc_counter` was never in the API reference and
exists only to serve this repository's own benchmarks.

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


### M2b · Audit remediation → `2.2.1` + `2.2.2` (patches) — ✅ COMPLETE

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
| GitHub Release notes link to the release's `CHANGELOG.md` section. They previously carried only `Full Changelog: 2.2.0...2.2.1`. The anchor is derived from the heading, since GitHub's anchor includes the release date (`## [2.2.1] - 2026-09-01` → `#221---2026-09-01`) and no version-only anchor exists; a missing heading degrades to a plain file link. Landed 2026-09-01; **first exercised at `2.2.2`**, since `create-release.yaml` only runs at a release. | Owner request |
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

**M2b closed 2026-09-16 with `2.2.2`.** All seven RFCs shipped. **One item is
deferred by construction:** RFC 027's first consumer pass runs *after*
publication, by someone who did not implement the release — tracked in the
`2.2.2` release record.

`2.2.2` was the first release cut against
`.git-exclude/release/RELEASE-CHECKLIST.md`.

The consumer pass runs against **`2.2.2`**, not `2.2.1`: by the time it could
run, `2.2.2` had replaced `usage-cli.md`, the CLI `--help` and the
type-annotation claim, so a pass against `2.2.1` would review text that no
longer exists.

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

#### Known limitation of the consumer-artifact gates — recorded 2026-09-16

**The four gates prove the Linux artifacts install. They do not prove the macOS
or Windows ones do.** Stated because RFC 026 §6 requires the gap to be written
down rather than left implicit in a green checkmark.

| Gate | Covers | Blind to |
|---|---|---|
| `npm install gate` | the published package, on `ubuntu-latest` | macOS and Windows per-platform packages — they exist only after publication, so nothing can install them beforehand |
| `pypi wheel gate` | the wheel built on `ubuntu-latest` | macOS and Windows wheels, built only by the release workflow |
| `crates package gate` | all four crates, built from their packaged form | platform-specific build failures; the crates are pure Rust, so this is the smallest of the three gaps |
| `docs example gate` | every runnable example in `docs/src/` | whether an example is *correct advice*, as opposed to syntactically and referentially valid |

The npm gate additionally reports on the **last published release**, not the
working tree — `optionalDependencies` are injected at publish time, so no local
artifact carries them. That lag is inherent and is documented in the workflow
itself.

**Mitigation: release-time verification.** The consumer pass (RFC 027 Rule 1)
installs from every registry on a real machine after publication, which is the
only position from which the macOS and Windows paths can be checked at all.
These gates narrow what the consumer pass has to catch; they do not replace it.

**Do not describe this set as complete coverage.** Four green checkmarks mean
the Linux artifacts install and the documented examples resolve — nothing more.

### M2c · Published-surface repair → `2.2.3` (patch) — ✅ COMPLETE

From the **first consumer pass** (RFC 027 Rule 1), run against published `2.2.2`
by a session with no history of this project. Disposition:
`.git-exclude/reviewed/2.2.2-consumer-pass/README.md`.

| RFC | Title | Priority | Size |
|---|---|---|---|
| 029 | Published-surface documentation repair | **P0** | S |

**Why a patch of its own.** The README's Node Quick Start does not parse, and it
renders on GitHub, crates.io, npm and PyPI. `usage-python.md` documents a keyword
argument that raises `TypeError`. Both are live; neither should wait behind M3's
renderer work.

#### What the consumer pass exposed about our controls

**`README.md` has never been in scope for any documentation RFC.** RFC 023's
boundary was `docs/src/getting-started/`; RFC 007's was six source files. The
most-read file in the project sat outside every one — **the third scope-boundary
miss this milestone**, after RFC 022's `examples/` consumers and RFC 007's count
table.

RFC 027 Rule 2 now requires boundaries be derived from a search. **That is
necessary and was not sufficient**: a search only covers what you think to
search for.

**The docs-example gate could not see it either** — `--root` defaults to
`docs/src`. And it checks symbol resolution, not keyword arguments, so the
Python defect was invisible twice over. Both recorded as corrections on RFC 026;
both fixed by RFC 029.

**One finding went to M3, not here.** F-04 — link text silently losing spaces
inside `<a>` — shares RFC 024's root cause but has a worse symptom: `****[b](/x)`
is visibly wrong, `[Readmore now]` reads as prose with a word gone. Added to
RFC 024 as an explicit acceptance criterion so it cannot be fixed by accident.

**Recorded, not scheduled:** F-24, deep nesting is quadratic — 25k depth 0.72s,
100k 10.5s, 300k 255s. The README's "no stack overflow at any depth" claim holds;
the failure mode moved from crash to hang. M4. And F-23, default `Balanced`
emitting 1,190 `<a id>` anchors on a real Wikipedia page — a design question
shared with bekoedit's item 8, not a defect.
**Further evidence, RFC 028 review, 2026-09-17:** 2,000-deep nested `<b>` around a fixed payload is about 5× slower than
no wrapper, in 2.2.3's renderer and RFC 028's alike — pre-existing, likely html5ever's formatting-element handling. Not
investigated.

### M3 · Output validity → `2.3.0` (minor) — ✅ COMPLETE, awaiting the tag

**All ten RFCs implemented and approved, 2026-09-22.** The owner accepted RFC 010's criterion 7 the same day: ship the
correctness work, recover the speed in RFC 012. What remains is release prep — the list below — then the pre-tag checkpoint.

**Reshaped by owner decision, 2026-09-16.** M3 was "conversion fidelity", with tables
(RFC 008) and element coverage (RFC 009) in it. The RFC 025 harness then measured the
output: **104 cells, 64 known defects**, several of them content-destroying, on HTML
mdka already claims to handle. `2.3.0` is now **validity**: every known defect in
existing output has an owner and ships together — RFC 025, 024, 028, 010. Tables and
element coverage move to **M4 / `2.4.0`**, where they build on valid output and on the
GFM parsing the harness gains in `025c`. Reasoning:
`.git-exclude/reviewed/025-output-validity-harness/README.md` §5–§6.

| RFC | Title | Priority | Size | Order |
|---|---|---|---|---|
| 030 | Crates package gate: verify the workspace, not the registry | P1 | S | ✅ implemented & approved |
| 031 | Docs example gate must compile what mdBook publishes | P1 | M | ✅ implemented & approved; D6 → RFC 033 |
| 032 | Gates report every failure; execute Python/TS examples | P2 | S | ✅ implemented & approved |
| 033 | Published docs: the source is what the reader gets | P1 | S | ✅ implemented & approved |
| 034 | PyPI: declared wheel matrix, checked where published | P1 | M | ✅ implemented & approved (034, 034b) |
| 025 | Markdown output-validity harness | **P0** | M | ✅ harness and `025c` approved (`d5d64cd`: 115 cells, 69 known defects, CommonMark + GFM); corpus slice `025b` unscheduled |
| 024 | Inline composition: route every writer through the output sink | **P0** | M | ✅ implemented & approved (`1de7f2c`, `467ebf1`, `8d03b0c`); slices `024b`–`024e` ✅ (`e6d39ff`) — code holds text, fence at line start, blocks and `<br>` in code are text |
| 028 | Inline elements around block content; emphasis negated by its own style | **P0** | M | ✅ implemented & approved (`b91aafb`, `cb5e351`) — tree-query pre-pass, style negation, links distributed over blocks |
| 035 | Block structure inside containers — loose list items, ordered nesting, blockquote continuity (A-06/07/08) | **P0** | M | ✅ implemented & approved (`a90307d`) — container prefix stack; loose/tight rule |
| 010 | Escaping and text round-trip | **P0** | L | ✅ implemented & approved (`a8f5c7e`); criterion 7 — **cost accepted by the owner 2026-09-22** (~+9–14% text-heavy, cumulative vs 2.2.3) |

**Handoff hygiene rules, recorded 2026-09-16.**

1. **Before dispatch:** an amendment to an RFC that has an undispatched handoff
   updates that handoff in the same commit. (RFC 025 was amended to require both
   matrix directions while its handoff still specified one.)
2. **After dispatch — supersedes rule 1 for dispatched handoffs:** a handoff named to
   the owner as ready is **frozen**. Later changes go in a **new dated addendum file**
   beside it, named to the owner as a separate handoff; the original gains only a
   pointer line. The RFC 025 handoff was edited in place three times after it was
   ready, and the dev team built from an earlier version — a silent edit to a file
   someone is working from is not a notification.

**Before `2.3.0` is cut** — prep items recorded during M3 so they are not
rediscovered at the checkpoint:

**Dispatched to the dev team 2026-09-22:** `.git-exclude/release/2.3.0/prep-handoff.md` — items 1 to 5 below. The architect
holds the RFC moves, the index, this file and the version bump.

- Remove internal RFC IDs from two docs.rs-visible doc comments:
  `src/options.rs:87` (`pub preserve_ids`, "RFC 005 Slice B1") and `src/lib.rs:177`
  (`pub fn html_files_to_markdown_with`, "RFC 021"). Confirmed 2026-09-22 as the only two on
  **public** items; `src/lib.rs:246` is a private fn, `alloc_counter.rs` is `#[doc(hidden)]` and
  removed at 2.4.0, and the private modules are not rendered.
- **CHANGELOG: add the performance entry** — cumulative vs 2.2.3, about +9% to +14% on text-heavy
  input, no per-RFC attribution, with what it buys and that RFC 012 recovers it.
- Review the `[Unreleased]` CHANGELOG entries for RFC 031, written by the
  architect during review rather than by the implementer.
- ~~Move RFC 030, 031, 032, 033 and 034 to `done/`~~ — **done 2026-09-22**: all ten M3 RFCs moved, statuses set to *Implemented (2.3.0)*, `rfcs/README.md` updated and 19 files' references swept.
- **Performance claims** (under option A): one dated note on `docs/src/design/performance-characteristics.md` covering all three stale
  statements — the 2.0.0 table, the small-input claim, the malformed-input claim — and the owner's decision on `README.md:19`
  (*"without sacrificing speed or memory"*), which ships to crates.io, npm and PyPI.
- **bekoedit reply:** rewrite its corpus section before sending — the corpus was requested separately on 2026-09-16, and bekoedit replied that it **does not exist yet**. Confirm the nine vendored reproductions are Apache-2.0.
- ~~RFC 034 must be implemented before the cut~~ — done.
- ~~**Version bump to 2.3.0**~~ — **done 2026-09-22**: all four crates at 2.3.0, `version.sh` verified no manifest retains 2.2.3, `Cargo.lock` refreshed, CHANGELOG section dated. Next: the pre-tag checkpoint.
- ~~**Release gate: do not cut `2.3.0` with RFC 024 and without RFC 028.**~~ **Satisfied 2026-09-17** — RFC 028 landed (`b91aafb`). RFC 024's sink turned `<a><p>x</p><p>y</p></a>` from a link with joined words into no link at all — both known defects, fixed by RFC 028, but the interim state must not reach users.
- Docs gate: give `##` escape lines their own rejection message, and tell authors of multi-line strings containing `# ` lines to use a single-line string with `\n` (RFC 033 review §3).
- ~~D6 and the hidden-lines browser check (owner)~~ — superseded by RFC 033, which removes both dependencies instead of verifying them once.

**Reordered by the 2026-08-31 audit.** Tables were the largest *known* gap; the
audit found the larger *unknown* one. `mdka` produces invalid Markdown for
several everyday constructs — a linked image, bold inside a link, a bare `<pre>`,
a code span containing `_`. Emitting a correct table matters less than emitting
correct output for HTML that is already in scope, so 024/025/010 precede 008.
**Carried further on 2026-09-16:** 008 and 009 left M3 entirely (see the note at the top of this milestone).

**RFC 030 (implemented) and RFC 031 go first, ahead of the engine work, though
neither is the most important item here.** RFC 031 came from the `2.2.3`
consumer pass: the docs example gate wrapped `?`-using Rust in a
`Result`-returning main while mdBook wraps in a plain one, so two examples
failed behind a Run button under a green gate — the third gate in this project
found checking a more forgiving stand-in than the consumer's artifact.

**On RFC 030:** It is CI-only and small, and it repairs the instrument
that will be watching everything else in M3. The crates package gate has been
red at every release commit since RFC 026 created it, and its red *skips* `mdka-node`
and `mdka-python` — so for two releases nothing has verified that those two
crates package standalone. Worse, now that `2.2.3` is published the gate will
read **green on `main` with no code change at all**, and flip red again at the
next version bump: its colour tracks the release calendar rather than the tree.
Fix the instrument before taking the measurements.

**RFC 025 lands first among the engine work, and is the reason the rest are
findable.** 136 tests were
green while all of this shipped, because no test parses mdka's output as
Markdown — every renderer assertion compares against a string we wrote ourselves.
A suite authored by the same hand as the renderer cannot discover that the
renderer's output is not Markdown.

**RFC 010 is now populated** by the audit: `A-03` (escaping inside code spans),
`A-04` (unescaped destinations and titles), `A-05` (fixed-width fences), `A-09`
(line-leading digits), `A-10`, `A-11`, and `D-05`. It no longer needs to start
from a blank survey.

**Exit criteria.** Every construct in the composition matrix round-trips through
a CommonMark parser to the structure mdka intended, **in both directions —
inline-inside-container and block-inside-inline**; tables round-trip to GFM pipe
syntax including alignment and header rows; each rule in
`docs/src/api/text-processing.md` is confirmed or corrected against a test.

#### Field report from bekoedit, 2026-09-16

A downstream consumer building paste-as-Markdown reported nine measured gaps.
**All nine reproduced exactly**; assessment in
`.git-exclude/reviewed/upstream-bekoedit-2026-09-16/README.md`, reply drafted in
`.git-exclude/upstream/bekoedit/send/draft/`.

Three were already scheduled (tables → RFC 008; the `\1.` escape → RFC 010;
strikethrough → RFC 009). The rest are new:

| Item | Disposition |
|---|---|
| Emphasis wrapping block content emits stray `**` | **RFC 028**, above |
| Task-list checkboxes dropped | added to RFC 009's scope |
| Inline-`style` emphasis ignored; `data:` URI option; `emit_id_anchors` independent of mode; backslash hard-break option | **candidate options RFC** — see below |

**Two things this report changed beyond its own items.**

RFC 025's composition matrix had only one direction. Every cell put an inline
construct inside a container; none put a block inside an inline, which is where
RFC 028's defect lives. **The 56-finding external audit missed it too.** The
matrix is amended and the exit criterion above now names both directions.

And bekoedit offered a corpus of real clipboard HTML — browsers, Google Docs,
Word, LibreOffice. **Accept it.** It is the input this project cannot generate
for itself: our fixtures are composed from our model of what HTML looks like, and
RFC 028 is the proof that the model has gaps. Recorded as a required input to
RFC 025.

**Reply held until after the release cut** — owner decision, 2026-09-16. The
draft is written and complete at
`.git-exclude/upstream/bekoedit/send/draft/2026-09-16-reply-conversion-gaps.md`,
marked HELD with its trigger.

Taken as the **next** cut, `2.2.2`. Noted there and here because `2.2.2` carries
none of bekoedit's items — they land in `2.3.0` — so the reply will still be
promising rather than reporting. If the owner would rather it carry shipped
fixes, the trigger moves to the `2.3.0` cut.

**Sending it is a release-checklist item**, not something to remember. RFC 027's
checklist gains: *"send any correspondence held for this release."* A draft with
a trigger and no owner is a draft that never goes out.

**Noted, not acted on:** bekoedit ranks tables the highest-impact item, as the
audit did. That is now two independent voices. M3 still sequences correctness
ahead of tables — wrong output for HTML we already claim to handle is worse than
missing support for HTML we do not — but the owner should see the ranking rather
than have it buried in a sequencing decision the architect made alone.

#### Dispatch discipline — added 2026-09-16

RFC 028's handoff was written into `rfcs/handoffs/` while both its preconditions
were unmet, carrying its gate in a header metadata line. The implementer stopped
and escalated correctly
(`.git-exclude/reviewed/028-sequencing-conflict/README.md`).

**A handoff present in `rfcs/handoffs/` is an instruction to start.** A
sequencing clause inside it is not a substitute for not dispatching it — it
depends on the reader noticing a line that contradicts the document's existence.

**Rule: a handoff whose preconditions are unmet is not written into
`rfcs/handoffs/` until they are near.** Where one already exists in that state,
its gate goes at the very top as a stop block, not in the header table. RFC 028's
handoff has been rewritten that way.

This is the architect's discipline, not the implementer's.

#### Candidate — conversion options for real-world HTML

**Not scheduled. Needs owner appetite before an RFC is drafted.**

Four additive options requested by bekoedit, listed so they are not lost:

| Option | Why |
|---|---|
| `emit_id_anchors`, independent of mode | `preserve_ids` conflates keeping `id` information with emitting raw HTML to carry it. A caller wanting Balanced's other choices without raw HTML in the output has no way to say so. **A gap RFC 005 created.** |
| Drop or alt-only `data:` URI images | A pasted screenshot puts megabytes of base64 into the output |
| Read inline `style` for emphasis (opt-in) | Google Docs and some editors express bold/italic only through `style` |
| Backslash hard-break instead of two trailing spaces | Editors that strip trailing whitespace silently remove the break |

**Harness limitations recorded, 2026-09-17 (RFC 024 review of `024d`):**

- `known_defect` is strict in **every** mode, so a defect present in only some modes (e.g. one a mode's unwrapping hides) cannot be
  marked; such a cell must stay unmarked until fixed. A per-mode marker is a candidate if this recurs.
- The `[text]` word check splits words at HTML block tags regardless of `unwrap_unknown_wrappers`, so it disagrees with Minimal and
  Semantic joining unwrapped `<div>`s (documented behaviour).

**Evidence for `emit_id_anchors`, 2026-09-16:** in Balanced mode every Google Docs
paste emits Google's internal clipboard GUID as a raw anchor —
`<a id="docs-internal-guid-…"></a>` — because `preserve_ids` is tied to the mode.
Found while verifying RFC 028's premise.

RFC 005 and RFC 006 spent a milestone making the option surface honest. **Adding
four options needs deliberate appetite, not accumulation** — which is why this is
a candidate rather than a plan.

### M4 · Coverage and durability → `2.4.0` (minor)

| RFC | Title | Priority | Size |
|---|---|---|---|
| 008 | GFM table support — **moved from M3, 2026-09-16** | P1 | L |
| 009 | Element coverage extension (`dl`/`dt`/`dd`, `del`/`s`, `sup`/`sub`, task-list checkboxes) — **moved from M3 with 008** | P2 | M |
| 011 | Robustness: fuzzing + `MdkaError::Io` error-path tests | P2 | M |
| 012 | Benchmark hardening + regenerate published performance claims | **P1** | M |
| 013 | Internal comment migration to English | P2 | L |

**RFC 012's target, set 2026-09-22.** `2.3.0` converts text-heavy HTML about **9–14% slower than 2.2.3** (small +13.7%, medium +13.0%,
large +10.9%, flat +8.8%, deep_nest −1.1%; three independent runs). The recovery target is **cumulative against 2.2.3**, not per-RFC:
roughly a third to a half of the cost is RFC 024's single-writer bookkeeping, RFC 028's and RFC 035's pre-passes and container stack —
they are **not** "already paid for". **Where the cost sits, measured 2026-09-22 by input shape:** the container
prefix stack and list bookkeeping dominate on list-heavy input (+13.2% for 024/028/035 alone on 100k list items), per-character
escaping dominates on punctuation-dense prose (+7.9% for RFC 010), and plain prose in few elements costs least (+4.4% total).
Recovery routes: bulk-scan runs of ordinary characters; cache each container depth's prefix string; skip the pre-scan for documents
with no inline wrapper and no list. **Allocation evidence, 2026-09-22 (prep review):** peak memory is unchanged (+0.8% to +1.5%,
re-measured by the architect) but the **allocation count rose 59% to 91%** on the benchmark documents, +200% on 15,000 nested quotes,
and 56 → 30,058 on 30,000 list items — about **one extra allocation per line inside a list or quote**. That is the prefix being rebuilt
as a fresh `String` per line, and it makes the "cache the prefix" route measurable rather than speculative. RFC 012 also regenerates the published performance page, whose table is from 2.0.0 and whose
small-input and malformed-input claims no longer hold. Raised to P1: the page ships stale claims until it lands.

**RFC 013's scope is measured, 2026-09-16.** The published `2.2.1` sdist carries
Japanese in 22 files, ~304 lines — the private modules in `src/`, plus `tests/`,
`benches/`, `examples/`, `python/test_mdka.py`, `version.sh` and
`python/pyproject.toml`. RFC 007 cleared everything *rendered* to a user, so the
**wheel** `pip install` fetches now carries zero non-compliant Japanese. What
remains is source-visible only and all of it belongs here.

RFC 013 is a large, purely mechanical diff. It is scheduled into a quiet release
deliberately, so it does not bury substantive changes in `git blame`.

#### Scheduled: remove `mdka::alloc_counter` — with a required gate

Deprecated in `2.2.2`, removed here, per the owner decision recorded above. No
RFC number; it is the second half of RFC 022.

> **Gate — before the removal lands, re-run the crates.io reverse-dependency
> check** and confirm no dependent references `alloc_counter`,
> `CountingAllocator` or `AllocSnapshot`.

At the 2026-09-01 ruling all six dependents referenced none of them. **That
check must be re-run, not cited.** New dependents appear between releases, and a
two-release-old check is exactly the stale claim this project keeps finding. If
a dependent then uses it, the removal waits and becomes 3.0 work.

Also remove the `#[allow(deprecated)]` at the three benchmark and example use
sites in the same change.

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
| 008 | GFM table support | M4 (moved from M3, 2026-09-16) | P1 | L | 001, 025 |
| 009 | Element coverage extension | M4 (moved from M3, 2026-09-16) | P2 | M | 008 |
| 010 | Escaping and text round-trip | M3 | P0 | L | 024, 025 |
| 011 | Robustness: fuzzing + I/O error paths | M4 | P2 | M | 001 |
| 012 | Benchmark hardening | M4 | P2 | M | 008 |
| 013 | Internal comment migration to English | M4 | P2 | L | 007 |
| 014 | Release-time CI verification | M1 | P1 | S | 001 |
| 015 | Release tooling completion | M1b | P1 | M | 014 |

RFCs 016 onward are tracked in [`rfcs/README.md`](rfcs/README.md), which is the
authoritative index; this table is kept for the originally planned portfolio only.

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
