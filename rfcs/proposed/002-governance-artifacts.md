# RFC 002 — Governance artifacts

**Status.** Proposed
**Tracks.** M1 · Trustworthy baseline. Makes release history and design history
auditable.
**Touches.** New `CHANGELOG.md` and `NOTICE` at the repository root; the
`Status` field of `rfcs/done/000-rfc-lifecycle-policy.md`; git tracking of the
`rfcs/` tree; `README.md` links.

## Summary

The project has no changelog, no `NOTICE` file, and its RFC directory is
untracked in git. This RFC adds the missing artifacts and brings the existing
ones into conformance with RFC 000 and the project rules.

## Motivation

Release history currently exists only as git tags. There is no document that
explains what changed between 1.6.9 and 2.0.0, or why 2.0.x became 2.1.x.
Downstream consumers on crates.io, npm, and PyPI have no migration reference.
The handoff bundle raised this as RISK-004 and left it open.

Separately, `rfcs/` exists on disk but is untracked, so the design record this
project depends on is not actually under version control.

`ROADMAP.md` and `rfcs/README.md` were produced during Phase 0 planning and are
already present. This RFC ratifies them as the maintained baseline and adds
what is still missing.

## Goals

- A `CHANGELOG.md` covering every published version.
- A `NOTICE` file, as the project rules require alongside Apache-2.0.
- `rfcs/` under version control.
- RFC 000's `Status` field conformant with the policy it defines.

## Non-goals

- Retroactively writing RFCs for work already shipped. History is captured in
  the changelog, not reconstructed as design documents.
- A release-automation change. Generating changelog entries from CI is a
  possible follow-up, not part of this RFC.
- Any code change whatsoever.

## Proposed design

### `CHANGELOG.md`

Keep a Changelog format, newest first, one section per published version with
its release date. Reconstructed from git tags and commit history.

Sections use `Added` / `Changed` / `Fixed` / `Removed`. Every entry states
user-visible effect, not commit subject. Where a version's intent cannot be
determined from history with confidence, the entry says so explicitly rather
than guessing — an honest gap is more useful than a fabricated summary.

Minimum coverage: the 2.x line in full (2.0.0 through 2.1.6), plus a
consolidated entry for the 1.x line summarising the 1.x → 2.0.0 transition.
Per-patch reconstruction of 1.x history is not required.

The unreleased section carries M1's changes and becomes the `2.1.7` entry at
release.

It must include the **MSRV correction** landing under RFC 001: the published
minimum supported Rust version is corrected from 1.85 to **1.88**. Record it as
a documentation correction, not a change — the 1.85 figure was never true for
the 2.x line, so no user loses a capability. See RFC 001 §MSRV correction.

### `NOTICE`

Apache-2.0 attribution notice naming the project and copyright holder
(`nabbisen`), consistent with `LICENSE` and the existing start-year-only
copyright convention established in commit `07dfbb3`. Per project rules, full
licence text stays in `LICENSE` and out of `README.md`.

### RFC 000 status field

`rfcs/done/000-rfc-lifecycle-policy.md` currently reads `**Status.** Implemented`
with no release tag. RFC 000 itself requires implemented RFCs to carry the
version in which the work shipped. Correct it to `**Status.** Implemented (2.1.6)`.

This is the policy's own self-application rule applied to the policy document.

### Git tracking

Add `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, and `NOTICE` to version control.
Confirm `.gitignore` does not exclude them — it currently does not, the tree is
simply unadded.

### README links

Add links to `CHANGELOG.md` and `ROADMAP.md`. Per project rules the README stays
concise, so these belong as entries in the existing "Learn More" table, not as
new prose sections.

## Compatibility

None. Documentation and repository metadata only.

## Security

No secrets are involved. When reconstructing the changelog, do not copy commit
messages verbatim without reading them — confirm no credential, token, or
internal path was ever recorded in a subject line before reproducing it.

## Testing and verification

No automated test. Verification is by inspection:

- Every git tag has a corresponding `CHANGELOG.md` section.
- Every version listed in the changelog matches a published artifact on
  crates.io, npm, or PyPI, or is marked as unreleased.
- `rfcs/README.md` lists every file under `rfcs/`, and every link resolves.
- No file under `rfcs/<state>/` has a `Status` field contradicting its folder.

The last two checks are the RFC 000 index invariants. Automating them as a
script is optional and explicitly out of scope here.

## Acceptance criteria

1. `CHANGELOG.md` exists, covers 2.0.0 through 2.1.6 plus a 1.x summary, and has an unreleased section.
2. `NOTICE` exists and is consistent with `LICENSE` and the project's copyright convention.
3. `rfcs/done/000-rfc-lifecycle-policy.md` carries `Implemented (2.1.6)`.
4. `rfcs/`, `ROADMAP.md`, `CHANGELOG.md`, and `NOTICE` are tracked in git.
5. `rfcs/README.md` links all resolve and match folder contents.
6. `README.md` links to the changelog and roadmap from the existing table.

## Prohibited shortcuts

- Do not invent changelog entries for versions whose intent cannot be
  established from history. Mark them as such.
- Do not paste the Apache-2.0 licence text into `README.md` or `NOTICE`.
- Do not reorganise or renumber existing RFCs.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| 1.x history is too sparse to summarise accurately | Changelog understates or misstates old behaviour | Scope 1.x to a single consolidated entry; state explicitly that per-patch 1.x history was not reconstructed |
| Changelog drifts once merged | Same audit gap returns | Release policy in `ROADMAP.md` makes a CHANGELOG entry a required release deliverable |

## Alternatives considered

- **Generate the changelog from conventional commits.** Rejected: the existing
  history does not follow a conventional-commit format, so generation would
  produce noise. Worth revisiting for future versions once entries are written
  by hand and a format is established.
- **Track release history in RFC files only.** Rejected: the project rules name
  `CHANGELOG.md` as an accepted mechanism, and downstream consumers expect a
  changelog at the repository root.
