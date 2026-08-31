# RFC 000 — RFC lifecycle policy

**Status.** Implemented (2.1.6)
**Tracks.** Cross-cutting documentation policy. Not tied to any
single feature; applies to the RFC directory itself.
**Touches.** `rfcs/` folder structure, the index file at
`rfcs/README.md`, the Status field convention used inside each
RFC, any cross-references between RFCs, and optional companion
handoff documents under `rfcs/handoffs/`.

## Summary

This RFC defines a lifecycle for RFCs themselves: where they live,
how they move between states, and what each state means. It is
written to be portable — any project starting an `rfcs/` (or
similarly-named) directory can adopt this policy verbatim. The
recommendations are deliberately conservative: the smallest set of
folders that gives implementers a clear answer to "what should I
look at next?" without imposing process overhead that small teams
will route around.

The policy's central claim is that **completed RFCs are not
deleted**. They move to a fixed location and stay there as a
record of the design decisions, alternatives considered, and
open questions resolved.

This policy also defines a small convention for optional
implementation handoffs. Handoffs are companion execution
documents for RFCs; they do not have their own lifecycle state.
Their status is inherited from the related RFC. The reasoning,
anti-patterns, and adoption guidance are spelled out below.

## Why a written policy

A common pattern in young projects: an `rfcs/` folder accumulates
fifteen or twenty Markdown files in flat structure, all named
`NNN-slug.md`. Some are implemented; some are abandoned; some
are mid-review. New contributors cannot tell which is which
without reading each file's prose. The maintainer keeps the
state in their head.

Eventually one of three things happens, all bad:

1. The maintainer prunes the folder, deleting "obviously done"
   RFCs to clean up. The design rationale is lost. Six months
   later, someone proposes the same idea, and the discussion
   restarts from zero.
2. The folder grows to fifty files. Contributors give up on
   reading any of them.
3. A second informal status system emerges — issue labels,
   spreadsheets, project boards — that the RFC files
   themselves don't reflect, creating a drift problem.

This RFC averts all three by fixing the rules up front.

## Scope and applicability

This policy targets projects whose RFC directory has roughly
**5 to 100 RFCs** at any given time, maintained by **one to ten
core people**, with **occasional outside contributors**. Above
that scale, the policy still works but probably needs additional
machinery (review SLAs, automated state checks, a dedicated RFC
shepherd role) that is out of scope here.

Below that scale (a project with three RFCs total), this policy
is overkill — a single flat folder with a README is enough.
Adopt only when flat starts to hurt.

The policy makes no claim about *what* an RFC contains. Each
project decides whether RFCs are tightly templated or
free-form, whether they cover features only or also operational
decisions, and whether they double as Architecture Decision
Records. This RFC governs only their lifecycle and storage.

Some projects also maintain developer handoffs, PR plans, or QA
checklists that explain how to implement a particular RFC. Those
are not RFCs and should not introduce a parallel status system.
This policy gives them a small companion-document convention so
they remain findable without becoming a second lifecycle.

## Lifecycle states

An RFC is in exactly one of the following states at any time:

| State | Meaning |
|---|---|
| **Draft** | The author is still writing. Not ready for review by anyone but the author and immediate collaborators. |
| **Proposed** | Open for review and discussion. Implementer should *not* yet start work — the design may change. |
| **Implemented** | The work has shipped (in a release, on `main`, or wherever the project's stability marker lives). The RFC is now a historical record. |
| **Withdrawn** | The author or maintainer decided not to pursue this RFC. The work will not happen. |
| **Superseded** | A later RFC replaces this one. The replacement RFC's identifier is recorded in this RFC's Status field. |

The states are deliberately few. Some projects add **Accepted**
between Proposed and Implemented (meaning "design is settled,
implementer may start, but work has not yet shipped"). For most
projects this state is formalisation overhead — proposed and
accepted collapse in practice because the same person both
proposes and implements. Larger organisations with separated
roles may want it; the variant is described in
[§ Folder layout: 5-folder variant](#folder-layout-5-folder-variant)
below.

## Folder layout

The recommended structure is **four folders**, three of which
hold RFCs:

```
rfcs/
  README.md           ← index; lists all RFCs by state
  proposed/           ← Proposed RFCs (open for review)
    NNN-slug.md
  done/               ← Implemented RFCs
    NNN-slug.md
  archive/            ← Withdrawn or Superseded RFCs
    NNN-slug.md
```

A fourth, optional folder holds Drafts:

```
  draft/              ← (optional) Draft RFCs not yet in review
    NNN-slug.md
```

Most small projects do not need `draft/` — authors can write
in a personal branch or a gist until they're ready to open a
review. Add `draft/` only when multiple authors regularly
need a shared place to share drafts.

A project may also add an optional `handoffs/` folder for
implementation companion documents:

```
  handoffs/          ← optional; companion execution docs for RFCs
    NNN-slug/
      implementation-handoff.md
      task-breakdown-pr-plan.md
      acceptance-qa-checklist.md
      README.md      ← optional index / scope note for this RFC's handoffs
```

`handoffs/` is deliberately not split into `proposed/`, `done/`,
or `archive/`. A handoff's status is inherited from the matching
RFC number. If `rfcs/proposed/057-slug.md` is Proposed, then
`rfcs/handoffs/057-slug/` is a proposed implementation companion.
If the RFC moves to `done/`, the handoff becomes historical with
it. If the RFC moves to `archive/`, the handoff follows that
meaning. Do not manage handoff status separately.

Each top-level RFC state folder corresponds 1-to-1 with a lifecycle state.
**The folder is the source of truth for the state.** A file's
location is what determines its state, not the Status field
inside the file (the file's Status field must be kept consistent
with the folder, but if the two ever disagree, the folder wins).

Movement between folders is the operation that changes an RFC's
state. To accept an RFC for implementation, you move it from
`proposed/` to `accepted/` (in the 5-folder variant) or leave
it in `proposed/` until it ships (in the 4-folder variant). To
mark it Implemented, you move it to `done/`. To withdraw or
supersede it, you move it to `archive/`.

Handoffs do not move between state folders. They remain under
`rfcs/handoffs/NNN-slug/`; their lifecycle meaning is derived
from the RFC's current folder and Status field.

### Folder layout: 5-folder variant

For organisations where the design and implementation roles
are clearly separated, a fifth folder makes the boundary
explicit:

```
rfcs/
  proposed/    ← under review
  accepted/    ← review complete; implementer may start
  done/        ← shipped
  archive/     ← withdrawn or superseded
  draft/       ← (optional)
```

Use this variant if "the maintainer signed off" is a meaningful
event distinct from "the implementer finished." Skip it
otherwise — `accepted/` will sit empty in projects where the
two events collapse, and an empty folder is a maintenance
burden with no payoff.

This RFC is written for the 4-folder variant. The 5-folder
variant works identically with one extra transition.

## Companion handoffs

A handoff is an optional implementation companion to an RFC. It
is useful when an RFC is large enough that implementers need a
separate execution package: implementation notes, PR sequencing,
acceptance checks, QA cases, migration reminders, or release
risks.

A handoff should answer a different question from the RFC:

- the RFC records **what decision was made and why**;
- the handoff records **how to implement and verify it safely**.

Handoffs must not override RFC decisions. If handoff work
uncovers a design conflict, update the RFC first, then update
the handoff to match. The RFC remains the authority for design
and lifecycle status.

A conventional handoff directory is:

```text
rfcs/handoffs/NNN-slug/
  implementation-handoff.md
  task-breakdown-pr-plan.md
  acceptance-qa-checklist.md
  README.md                 ← optional
```

These filenames are recommendations, not mandatory policy. Small
RFCs often need no handoff. Large multi-PR RFCs benefit from one.
Projects should avoid putting rough chat transcripts, obsolete
review notes, or every intermediate discussion into
`rfcs/handoffs/`; only current, reviewed, implementation-useful
companion documents belong there.

## Status field inside each RFC

Each RFC carries a `Status` field at the top, alongside other
metadata. The exact format is up to each project; one common
shape:

```markdown
# RFC NNN — Title

**Status.** Proposed
**Tracks.** What this addresses.
**Touches.** Where the work lands.
```

The Status field's value mirrors the folder. When an RFC moves
between folders, the Status field updates in the same commit.
For Implemented RFCs, the Status field carries the version or
release tag in which the work shipped:

```markdown
**Status.** Implemented (v1.4.0)
```

For Superseded RFCs, the field names the replacement:

```markdown
**Status.** Superseded by RFC 042
```

For Withdrawn RFCs, the field carries a one-line reason:

```markdown
**Status.** Withdrawn — overlapped with RFC 035; merged there.
```

Two reasons to keep this redundancy with the folder:

1. **Self-contained files.** A reader who opens an RFC by URL
   without seeing the folder context can still tell the state.
2. **Version-control history.** `git log -p path/to/rfc.md`
   shows state transitions inline, even if the file moves
   between folders (some VCS tools track moves better than
   others).

## Naming and numbering

RFCs are numbered sequentially from `001`. Numbers are assigned
when the file is first created — not when it ships, not when
it's accepted. **Numbers are stable forever**: a file does not
get renumbered when it moves between folders, even if its
priority or order changes.

The filename is `NNN-slug.md` where `NNN` is the zero-padded
number and `slug` is a short, lowercase, hyphen-separated
description. The slug is for human readers; the number is for
unambiguous reference.

```
001-feature-flags.md
015-deprecate-old-api.md
142-storage-backend-abstraction.md
```

Three-digit numbering covers the first 999 RFCs, which is more
than most projects ever reach. If the project does cross 999,
switch to four digits prospectively (new RFCs use `NNNN`); do
not retroactively renumber existing files.

**Numbers are never reused.** If RFC 005 is withdrawn, the
number stays in `archive/` and the next new RFC is 006.
Renumbering would invalidate cross-references, audit logs, and
any external link to the file.

## Cross-references between RFCs

When one RFC references another, use a relative path that
reflects the target's current folder:

```markdown
See [RFC 010](../done/010-revoke-tokens.md) for the prior work.
```

This means cross-references break when an RFC moves between
folders. That is acceptable — it's a small, finite, mechanical
fix that surfaces during review and is easy to grep for.
Tooling can help (see [§ Optional CI invariants](#optional-ci-invariants)
below), but for projects without CI on RFC files, periodic
manual sweeps suffice.

A common convention to make this less painful: when an RFC
moves to `done/` or `archive/`, the maintainer also runs a
quick `grep -l "<NNN>-<slug>.md" rfcs/` to find inbound
references and updates them in the same commit. The cost is
seconds per move; the alternative — broken links accumulating
silently — is worse.

If your project's review tool renders relative links (GitHub,
GitLab, sourcehut all do), broken links become visible in the
preview, which gives a second line of defence.

If a handoff exists for an RFC, the RFC may link to it with a
relative path such as `../handoffs/057-slug/README.md`, and the
handoff should link back to the RFC. These links are references,
not lifecycle state. Moving the RFC between `proposed/`, `done/`,
and `archive/` requires the same link sweep described above.

## Review and transitions

The transitions between states are:

```
                    ┌─────────────────────┐
                    ▼                     │
[author writes]──▶ Draft? ──▶ Proposed ──▶ Implemented
                                │             │
                                ▼             ▼
                          Withdrawn      (lives in done/)
                          Superseded     forever
                                │
                                ▼
                          (lives in archive/)
                          forever
```

State transitions are operations performed by the maintainer
(or whoever has commit authority on the RFC directory). The
operations:

- **Open.** New file in `proposed/` (or `draft/` if used).
  Triggered by an author opening a pull request adding the
  file.
- **Accept and ship.** RFC is implemented; the implementer or
  maintainer moves the file from `proposed/` to `done/` and
  updates the Status field with the release tag. Done in the
  same commit (or commit series) that ships the implementation.
- **Withdraw.** The author or maintainer decides not to pursue
  the RFC. Move to `archive/` with Status updated and a brief
  reason added in the file.
- **Supersede.** A new RFC takes over the design space of an
  older one. Move the older RFC to `archive/`, update its
  Status to `Superseded by RFC NNN`, and add a reciprocal note
  in the new RFC.

There is no "rejected" state distinct from "withdrawn". An RFC
that the maintainer declines to accept is moved to `archive/`
with a reason — the file is preserved as evidence that the
discussion happened.

### Granularity of transitions

A single RFC does not need to enumerate all its sub-features
to qualify as Implemented. Partial implementation is fine if
the partial work captures the RFC's main design decision; any
deferred work either gets a follow-up RFC or is logged in the
RFC's Status section as an explicit "deferred" note.

This is a judgement call. The principle: **don't keep an RFC
in `proposed/` indefinitely just because one open question
remains**. Move it to `done/` when the design has shipped, and
record what didn't make it.

## README integrity

The `rfcs/README.md` file serves as the index. It should:

1. List all RFCs across all folders, grouped by state or
   priority (whichever is more useful for the project).
2. Use relative links that reflect each RFC's current folder.
3. Be updated in the same commit that moves an RFC between
   folders.
4. Optionally indicate that a handoff exists for RFCs that have
   one, without treating the handoff as a separate lifecycle
   item.

A typical structure:

```markdown
# Project RFCs

## Proposed
| ID | Title | Priority |
|----|-------|----------|
| 042 | [Feature flags](./proposed/042-feature-flags.md) | High |
| 047 | [Caching layer](./proposed/047-caching.md) | Medium |

## Implemented
| ID | Title | Shipped in |
|----|-------|------------|
| 010 | [Revoke tokens](./done/010-revoke-tokens.md) | v1.4.0 |
| 015 | [Deprecate API](./done/015-deprecate-old-api.md) | v1.5.0 |

## Archive
| ID | Title | Reason |
|----|-------|--------|
| 023 | [Multi-region](./archive/023-multi-region.md) | Withdrawn |
| 035 | [Old caching](./archive/035-old-caching.md) | Superseded by RFC 047 |
```

Some projects prefer prose to tables; either works. The point
is that the index reflects every RFC's current state and
location, and the index is what new contributors read first.

## Optional CI invariants

For projects with CI on the RFC directory, the following
invariants are worth checking:

- Every file under `rfcs/<state>/` has a Status field whose
  value matches the folder.
- Every relative link inside an RFC resolves to an existing
  file.
- No RFC number is duplicated across folders.
- Every RFC listed in `rfcs/README.md` exists at the linked
  path; every RFC under `rfcs/` is listed in `README.md`.
- Filenames match the `NNN-slug.md` pattern and the slug
  matches the title (loosely).
- Every `rfcs/handoffs/NNN-slug/` directory, if handoffs are
  used, corresponds to an existing RFC number.
- Handoff directories do not duplicate lifecycle state with
  their own `proposed/`, `done/`, or `archive/` subfolders.

A simple script in `scripts/check-rfcs.sh` or
`xtask check-rfcs` can run these checks. None of them need
sophisticated parsing — `grep`, `find`, and basic shell
suffice.

For projects without CI on the RFC directory, these checks
are still useful as a periodic manual hygiene pass. Don't
build elaborate tooling before the project's scale demands it.

## Adoption guidance for new projects

If you're starting an `rfcs/` directory from scratch, the
minimum viable adoption of this policy is:

1. Create `rfcs/proposed/`, `rfcs/done/`, `rfcs/archive/`.
2. Add `rfcs/README.md` with a state-grouped index.
3. Adopt the `NNN-slug.md` naming and start at `001`.
4. Write the first RFC. Put it in `proposed/`.
5. When the work ships, move it to `done/` with a Status
   field carrying the release tag.

If an RFC is large enough to need execution guidance, add an
optional companion directory under `rfcs/handoffs/NNN-slug/`. Do
not add a separate handoff status folder; the RFC's state is
enough.

That's the entire policy in five steps, plus optional handoffs
when they are genuinely useful. The other sections of
this RFC exist to handle edge cases as the directory grows;
ignore them until you hit the relevant case.

If you're adopting this policy for an *existing* RFC directory:

1. Audit each existing file. Decide its state.
2. Move it to the corresponding folder.
3. Add or update the Status field in each file.
4. Rewrite cross-references with the new paths.
5. Rebuild `rfcs/README.md` to reflect the new structure.
6. If handoffs already exist, move only reviewed, current,
   implementation-useful handoffs under `rfcs/handoffs/NNN-slug/`
   and let their state inherit from the matching RFC.

The migration is mechanical but tedious. Schedule it as a
single dedicated change rather than spreading it across
unrelated commits.

## Anti-patterns

Patterns that look reasonable but cause long-term harm:

### Deleting completed RFCs to "clean up"

The most common mistake. The reasoning sounds correct: "the
RFC is implemented, the code and CHANGELOG capture the
result, the design document is now redundant." It isn't.
RFCs capture the *why* — alternatives considered, trade-offs
weighed, open questions resolved. Code captures the *what*.
The two are different artifacts; both are needed.

When an RFC is deleted, future contributors see the current
code and have no record of why it isn't different. They
re-derive the design space from scratch, often missing the
constraints that drove the original choice. Months later,
someone proposes the same alternative that the original RFC
already considered and rejected, and the discussion repeats.

The fix: never delete. Move to `done/` and leave it there.

### Renumbering RFCs during reorganisation

Tempting when migrating an old flat directory: renumber to
fill gaps from withdrawn RFCs, or renumber by priority order
in the new folders. Don't. External references — issue
trackers, commit messages, Slack history, design-review
documents — all reference RFC numbers. Renumbering breaks
every one of those references silently.

The numbering is permanent. Withdrawn numbers stay withdrawn.

### Formalising `accepted/` in small projects

The 5-folder variant is appealing because it makes the
"maintainer approved" event explicit. In small projects this
event collapses with "implementation complete" — the same
person makes both decisions, often in the same commit. The
`accepted/` folder ends up perpetually empty (because RFCs
go straight from `proposed/` to `done/`) or perpetually full
(because nothing ever ships and they all sit there).

Adopt the 5-folder variant only if the design and
implementation roles are genuinely separate. Otherwise the
4-folder variant is right-sized.

### Letting cross-references rot

When an RFC moves between folders, inbound references break.
If the project doesn't fix them in the same commit, broken
references accumulate. After a few moves, the index becomes
unreliable and contributors stop trusting links.

The fix: a one-line `grep -l 'NNN-slug.md' rfcs/` before
every move, plus updating every match. Or CI enforcement.
Either works; doing nothing does not.

### Status fields that lie

If an RFC's Status field says `Proposed` but the file lives
in `done/`, contributors don't know which is correct. The
folder is authoritative by this policy, but a misleading
Status field still causes friction every time someone reads
the file directly (without seeing the folder context).

The fix: update Status in the same commit that moves the
file. CI can enforce; manual review can catch.

### Silent withdrawal

An RFC that's been abandoned but not formally withdrawn
sits in `proposed/` indefinitely. Contributors waste effort
reviewing it; the maintainer's unspoken "I'm not going to
do this" is invisible.

The fix: when you decide not to pursue an RFC, move it to
`archive/` with a one-line reason. Even "didn't pan out;
priorities shifted" is enough. Silence is worse.

### Turning handoffs into a second RFC lifecycle

Implementation handoffs can be useful, but they become harmful
when they gain their own parallel status system. A handoff in
`handoffs/proposed/` beside an RFC in `done/` forces readers to
ask which status is authoritative. A handoff that says "accepted"
while the RFC says `Proposed` creates the same confusion as a
lying Status field.

The fix: keep handoffs under `rfcs/handoffs/NNN-slug/` and let
the matching RFC's folder and Status field define their state.
Handoffs are companions, not separate lifecycle artifacts.

### Letting handoffs override RFC decisions

A handoff is closer to an implementation guide than a design
record. If it changes the architecture, narrows scope, or adds a
new acceptance requirement that contradicts the RFC, future
maintainers have to read two documents and guess which one wins.

The fix: when a handoff discovers a design problem, patch or
supersede the RFC first. Then update the handoff so it describes
execution of the current RFC, not a competing design.

## Self-application

This RFC describes its own placement: it is itself an RFC
governed by the policy it defines, and it lives in
`rfcs/done/` because it is implemented (the policy is now in
effect for this project's RFC directory).

The transition that landed this RFC is the simultaneous
adoption of the policy and the migration of every existing
RFC into the new folder structure. Both happened in the same
release. This is the recommended adoption pattern for
existing directories: combine the policy's introduction with
the migration into a single, atomic change.

## Open questions

None at time of acceptance. Future refinements (review SLAs,
automated state-machine checks, integration with project
management tools) will, if needed, land as follow-up RFCs
referencing this one.

---

## Project adoption — the 5-folder variant, 2026-08-31

**This project uses the 5-folder variant.** Recorded here because the policy text
above is written for the 4-folder one, and the folder layout is the source of
truth for state — a reader must not have to infer which variant is in force.

```
rfcs/
  proposed/    ← under review by the architect and owner
  accepted/    ← owner has accepted; the implementer may start
  done/        ← shipped in a release
  archive/     ← withdrawn or superseded
```

`draft/` is not used.

### Why the anti-pattern above does not apply here

§ Anti-patterns warns against formalising `accepted/` in small projects, because
"maintainer approved" and "implementation complete" collapse when the same person
does both.

**They do not collapse here.** This project runs a three-tier model: the owner
sets scope and accepts, the architect designs and writes handoffs, and a separate
implementer builds. "Accepted" is a real, dated event performed by a different
party than the one who ships. The variant is adopted for exactly the reason the
policy names as its precondition.

The empty-folder failure mode the anti-pattern predicts is also worth watching
for the opposite reason: if RFCs accumulate in `accepted/` without shipping, that
is a queue, and the queue depth is a signal. `accepted/` standing full is as
much a finding as it standing empty.

### The extra transition

**Accept.** The owner accepts an RFC. Move it from `proposed/` to `accepted/`,
set `**Status.** Accepted <date> — implementer may start`, sweep inbound
references, and update `README.md` — all in the same commit, per the rules above.

`Accept and ship` in § Review and transitions therefore splits in two here:
accept (`proposed/` → `accepted/`) and ship (`accepted/` → `done/`).

### Applies to handoffs unchanged

Handoffs stay at `rfcs/handoffs/NNN-slug/` and inherit state from the RFC's
folder. A handoff beside an RFC in `accepted/` is an accepted handoff. The
prohibition on giving handoffs their own lifecycle folders is unchanged.

### A note for whoever implements § Optional CI invariants

That section proposes checking that "every relative link inside an RFC resolves
to an existing file." **This document would fail that check**, because its
examples reference illustrative RFCs — `010-revoke-tokens.md`,
`042-feature-flags.md`, `023-multi-region.md` and others — that do not and will
never exist here. They are samples of the convention, not references.

Exclude this file from that invariant, or teach the check to skip fenced blocks
and the § README integrity sample. Discovered by running the check by hand during
the 5-folder migration, which also found **seven genuinely broken links** created
at the 2.2.0 release, when RFCs 005, 006, 018 and 019 moved to `done/` without
the reference sweep this policy requires. The rule was right and I did not follow
it; a mechanical check would have caught it the same day.
