# RFC 027 — Verification discipline: the consumer pass and scope completeness

**Status.** Accepted 2026-08-31 — implementer may start
**Tracks.** M2b · Audit remediation → `2.2.1`
**Priority.** P1
**Touches.** `.git-exclude/` governance artifacts (handoff template, review-request format, release checklist), `ROADMAP.md`.
**Source.** Architect analysis of the 2026-08-31 audit. This RFC is about **why we did not find these ourselves**.
**Prepared.** 2026-08-31

## Summary

An external auditor found, in one pass, defects that four milestones of internal
review did not. The difference was not diligence or capability — it was
**position**. Every internal review read the artifacts we produced. The audit
consumed the product a user receives. Codify that position as a required step.

## The evidence

Four milestones. Thirteen RFCs. Every one reviewed, most with evidence packages,
several with multiple rounds. During that period all of the following were true
and none was found internally:

| Defect | How long | Why review missed it |
|---|---|---|
| `S-01` npm unusable | 12 releases, ~4 months | Every review read the workflow. None installed the package. |
| `D-06` CLI docs describe no-op flags as working | Since 2.2.0 | RFC 006's scope named `docs/src/api/`. Review checked that scope faithfully. |
| `D-08` false `py.typed` claim | Unknown | No review installed a wheel. |
| `D-12`/`D-13` examples that do not run | Unknown | No review executed an example. |
| `A-01`–`A-11` invalid Markdown | Since 2.0.0 | Every renderer review compared output to a string we wrote. |

**The pattern is one thing.** In each case the review verified *what we wrote*
against *what we meant to write*. None verified what a consumer receives. Both
the reviewer and the implementer stood inside the same boundary, so the boundary
itself was never examined.

This is the same defect class the project has now recorded five times — a
documented claim treated as a description of reality — but at the level of the
**review process** rather than any individual document. RFC 026 fixes the
mechanical half with CI gates. This RFC fixes the human half.

## Rule 1 — The consumer pass, before every release

Before a release is tagged, one pass is performed **entirely from outside the
repository**, by someone acting as a new user, recorded as a written artifact:

1. Install from each registry as documented — `cargo install`, `npm install`,
   `pip install`. Not from the workspace.
2. Follow the README Quick Start **verbatim**, including the prebuilt-binary
   path, and run what it says to run.
3. Execute every runnable example in the getting-started docs.
4. Convert a real-world HTML page — not a fixture — and read the Markdown output
   as a document.

**No step may substitute a local build.** If a step cannot be performed because
the artifact does not exist yet, that is the finding.

Step 4 is deliberately unstructured. `A-01`'s linked-image defect survives every
mechanical check we have and is obvious the moment a person reads output from a
real page.

The first consumer pass runs against `2.2.1`. Its output is a review-request
package like any other.

## Rule 2 — A scope boundary must be justified

`D-06` exists because RFC 006's scope named `docs/src/api/` and stopped there.
The implementer honoured that scope exactly; so did the review. The boundary was
wrong, and nothing in the process was positioned to ask why it was where it was.

**When a handoff scopes work by path or directory, it must state why the boundary
falls there, and confirm that the remainder of the affected class is either
covered or explicitly deferred to a named owner.**

For RFC 006 that sentence would have read: *"scope is `docs/src/api/`; the
getting-started layer describes the same options and is NOT covered — deferred to
RFC ___."* Writing it would have exposed that there was no reason and no owner.

The architect writes handoffs, so this is a constraint on me. The corresponding
reviewer obligation is to challenge a boundary that has no stated reason, rather
than to check the stated scope faithfully — which is what happened.

## Rule 3 — Evidence must state what was consumed

Review requests already require executed verification. They do not require saying
**what the verification ran against**.

Add to the required format: for each verification, state whether it ran against
the **workspace tree** or an **installed artifact**. Where it is the tree, say so
plainly rather than letting "npm test passed" imply more than it proves.

This is a one-line addition that would have made `S-01` visible as an
unanswered question at every release since 2.0.2.

## Rule 4 — Recommend a periodic external audit

The 2026-08-31 audit returned more than any internal review, and the reason
generalises: **a reviewer who shares the authors' assumptions cannot test those
assumptions.** Internal review remains necessary and is good at what it does —
it caught the `<hr>` and `<pre>` newline bugs, the `verify-ci` failure, the
`node_modules` corruption, and the `anchor_before` drift hazard. All were
*inside* our boundary.

**Recommendation to the owner: commission an external audit once per minor
release, or at least annually.** This is a resourcing decision and is recorded as
a recommendation, not a rule.

## What this RFC deliberately does not add

No new approval stages, no sign-off matrix, no checklist for its own sake. Each
rule above is traceable to a specific defect that shipped. A process control that
cannot name the failure it prevents is overhead, and this project's process is
already heavier than most — the audit called it an asset, and it stays an asset
only while every part of it earns its place.

## Compatibility

Governance only. No product change, no CI change (that is RFC 026).

## Risks

| Risk | Mitigation |
|---|---|
| The consumer pass becomes a formality | It produces a written artifact naming what was installed and what was run. A pass that found nothing must say what it did, so an empty pass is visible. |
| Rule 2 makes handoffs verbose | One sentence per scoped handoff. |
| Rule 4 is read as a standing cost | It is explicitly a recommendation for the owner to accept, defer, or decline. |

## Acceptance criteria

1. The handoff template requires a justified scope boundary (Rule 2).
2. The review-request format requires stating tree-vs-artifact for each
   verification (Rule 3).
3. A release checklist exists containing the consumer pass (Rule 1), and it is
   performed for `2.2.1` with its output recorded.
4. The owner has recorded a decision on Rule 4.
5. Each rule in the governance artifacts names the defect it prevents, so a
   future reader can judge whether it still earns its place.
