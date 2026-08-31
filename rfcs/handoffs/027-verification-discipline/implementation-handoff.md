# Developer Handoff — RFC 027 · Verification discipline

**Governing RFC.** [RFC 027](../../proposed/027-verification-discipline.md)
**Milestone.** M2b → `2.2.1`
**Priority.** P1
**Prepared.** 2026-08-31

---

## 0. ⚠ Read first — most of this RFC is not yours

RFC 027 changes governance artifacts. Those split by owner:

| Rule | Artifact | Who |
|---|---|---|
| 2 · justified scope boundary | Handoff template | **Architect** — I write handoffs |
| 3 · tree-vs-artifact evidence | Review-request format | **Architect** |
| 4 · periodic external audit | — | **Owner decision** |
| 1 · the consumer pass | Release checklist + its first execution | **You** |

**Your slice is Rule 1**, and it is the substantive one. I have already begun
applying Rules 2 and 3 — every M2b handoff, including this one, carries a scope
boundary paragraph and asks you to label evidence tree-vs-artifact. If those
additions are unclear or burdensome in practice, say so in your review request;
they are new and unproven.

## 1. Purpose

An external auditor found in one pass what four milestones of internal review did
not. The difference was **position**, not diligence: every internal review read
the artifacts we produced; the auditor consumed the product a user receives.

Your job is to make that position a required step, and to be the first person to
occupy it.

## 2. Build the release checklist

A written checklist, stored where release artifacts live
(`.git-exclude/release/`), containing the consumer pass below plus the existing
release mechanics already proven across 2.1.7, 2.1.8 and 2.2.0 — tag the push
tip, verify CI green on that commit, watch five runs appear, verify each registry
directly rather than trusting workflow status.

Do not invent new steps beyond RFC 027. Capture what we already do, add the
consumer pass, and make it a document a future release follows.

## 3. Perform the consumer pass against 2.2.1

**Entirely from outside the repository.** Ideally on a machine, container or
fresh user account that has never built this project. If that is not available,
say exactly what environment you used and what contamination might remain — an
honest caveat is worth more than an unverifiable claim.

1. **Install from each registry as documented.** `cargo install mdka-cli`,
   `npm install mdka`, `pip install mdka`. Not from the workspace. Not from a
   local build.
2. **Follow the README Quick Start verbatim.** Including the prebuilt-binary
   path — download the archive from GitHub Releases, extract it, run the binary.
   Do exactly what the page says, including the `cd` into the wrapper directory.
3. **Execute every runnable example** in the getting-started docs, as written.
4. **Convert a real web page.** Not a fixture. Fetch something substantial —
   documentation, an article, a page with images and links — convert it, and
   **read the Markdown as a document**. Render it if you can.

### Step 4 is not optional and not mechanical

`A-01`'s linked-image defect passes every automated check we have and is obvious
within seconds of reading real output. Step 4 is where a human notices that the
product is wrong in ways no assertion was written to catch. Budget real attention
for it.

Record what you converted, so the pass is reproducible.

## 4. What the output looks like

A review-request package like any other. It must state:

- The exact environment, and any contamination caveat
- Each step, what you ran, what happened
- **Every discrepancy**, however small — a wrong command, a missing step, output
  that looked odd
- Explicitly: what you did **not** do, and why

**A pass that finds nothing must still say what it did.** An empty pass with no
detail is indistinguishable from a pass that was skipped, and that is how a
checklist becomes a formality.

## 5. Scope boundary

Per RFC 027 Rule 2: this handoff covers the release checklist and the first
consumer pass. The handoff-template and review-format changes are architect work
and are already underway. Rule 4's audit cadence is the owner's decision and
needs nothing from you.

**Sequencing:** the consumer pass runs **after 2.2.1 is published**, since it
installs from registries. Everything else here can be written before.

## 6. Prohibited shortcuts

- Do not substitute a local build for any install step.
- Do not skip the prebuilt-binary path because `cargo install` worked.
- Do not skim step 4.
- Do not omit a discrepancy because it seems minor or because a fix is already
  scheduled — note it and reference the RFC.
- Do not perform the pass on the machine where you implemented 2.2.1 if you can
  avoid it. If you cannot, say so.

## 7. Known risks

| Risk | If it happens |
|---|---|
| A clean environment is unavailable | Use the cleanest you can, state precisely what it was, and name the contamination risk. |
| A step cannot be performed | **That is the finding.** Record it as a defect, not as an obstacle. |
| The pass finds a lot | Likely. Record everything; the architect triages. Do not fix as you go — report first, so the count is honest. |
| It reads as busywork after a green CI run | Then it is being done wrong. CI was green for twelve releases while npm was unusable. |

## 8. Acceptance checklist

- [ ] Release checklist exists, includes the consumer pass and existing mechanics
- [ ] Consumer pass performed against published 2.2.1, from outside the repo
- [ ] All four steps done, including the real-page conversion
- [ ] Environment stated, with any contamination caveat
- [ ] Every discrepancy recorded, however small
- [ ] What was not done, and why, stated explicitly
- [ ] Feedback given on Rules 2 and 3 as they appear in the M2b handoffs

## 9. Escalate rather than decide

Stop and raise if: an install step fails (that is a live defect, not a checklist
problem); the real-page conversion produces output you think is wrong but no
finding covers; or the checklist starts duplicating something that already exists
elsewhere.
