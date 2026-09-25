# RFC 047 — The release process lives in the repository, and its instructions are verified

**Status.** Implemented — shipped in `2.7.0`, 2026-09-25
**Author.** Architect
**Created.** 2026-09-25
**Milestone.** Unassigned. Governance and CI; no conversion change.
**Source.** Owner, 2026-09-25, on being handed two items to decide: *"Why me? Manual operation or local effort which is not automated contains risk around cost burst or quality unstability."*
**Amends.** RFC 002 (governance artifacts in `.git-exclude/`).
**Relates to.** RFC 046, whose gate this completes; RFC 027, whose Rule 1 RFC 046 retired.

---

## 1. Two things I raised as owner decisions, and should not have

After RFC 046 landed I put two items to the owner. Neither was a decision; both were work I had not
identified as work. The challenge was correct and this RFC is the answer.

### 1.1 The release checklist exists on one machine

`.git-exclude/release/RELEASE-CHECKLIST.md` is the document that says how this project releases — the
full-SHA CI rule, tag the pushed tip, never hand-make the release, poll the registries, the npm first-publish
prerequisite that `2.5.0` was broken by. It is ignored via `.git/info/exclude`, which is **per-clone and
never shared**. So it has **no history, no review, no backup, and exists in exactly one place.**

Losing it loses the accumulated lesson of eleven releases. Nothing about that is an owner preference.

**Separately, and worse, the same fact was a safety hole:** because the ignore rule was per-clone, a fresh
clone would not ignore `.git-exclude/` at all, and a routine `git add -A` could have committed the bekoedit
correspondence. **Already fixed** — the rule is now in the tracked `.gitignore`, where it travels with the
repository.

### 1.2 The README has no Windows instructions, and nothing could check them if it did

RFC 046's gate follows `README.md`'s extract-and-run steps against every published archive. For
`Windows-x64` and `macOS-aarch64` it can only confirm the file exists, because the runner is Linux — and for
Windows there is nothing to follow anyway: the README's only example is a POSIX shell pipeline into
`./mdka`, and the Windows archive contains `mdka.exe`.

**So a Windows user is given no command at all, and the gap is invisible to the gate.** I was about to write
a Windows line by hand. I cannot test it, and an untested instruction in the Quick Start is precisely the
defect class RFC 046 exists to catch.

## 2. What changed the cost answer

The dev team priced executing the other three archives and correctly hedged: *"macOS runners start slowest
and are the scarce kind; **a private repository** would bill them at a multiple."*

**This repository is public** (`gh repo view --json visibility` → `PUBLIC`), and GitHub documents standard
hosted runners as free for public repositories. **The cost is wall-clock, not money** — and the gate's
measured job is **10 seconds** (13s wall), the cheapest of the eight.

The implementer confirms this against current billing documentation before adding jobs rather than taking it
from this RFC; if it has changed, §3.2 becomes a real cost question and comes back.

## 3. Proposal

### 3.1 Move the release checklist into the repository

To `RELEASE-CHECKLIST.md` at the root, beside `ROADMAP.md` and `CHANGELOG.md`.

**Consistency, not a new exposure.** This project already publishes its RFCs, its handoffs, its review
amendments and a `ROADMAP.md` that records every open finding by name. A release checklist is less revealing
than any of those, and other projects publish theirs as a matter of course. The checklist is the odd one out.

It then gets what every other tracked document has: history, review in the same commit as the change it
describes, and a copy on every clone and on the remote.

**What must be handled:** the file points into `.git-exclude/` in places (release records, review requests).
Those pointers become dangling for a public reader. Rewrite them as descriptions of where the internal record
lives, not as paths — the same treatment applied to the living documents on 2026-09-24. **Nothing in
`.git-exclude/` moves**; correspondence, release records and review requests stay internal.

### 3.2 Complete the gate's platform coverage, and write the README from what it verifies

Add to `release-artifact-gate.yaml`:

- **`windows-latest`** and **`macos-latest`** jobs, each running steps 1–4 for its own archive with the same
  checker.
- **QEMU** for `Linux-aarch64-musl` in the existing Linux job — the dev team already ran that binary under
  `qemu-aarch64` by hand and it produced the README's output, so this is known to work before it is written.

Then **every published archive is executed by the gate**, and the `coverage:` block reports five of five
rather than two of five.

**The README follows from that, not from me.** Once a Windows job can run the documented Windows steps, the
Windows command is written and the gate proves it works on every push. Same for macOS, including whatever
Gatekeeper requires of a downloaded unsigned binary — **which nobody here has tested and this RFC does not
guess at.** If the macOS job shows the documented steps failing on a freshly downloaded archive, that is a
finding about our own Quick Start and it is reported, not worked around.

### 3.3 Not proposed

**A script for the rest of checklist §1.** The version/tree/CI/gate checks could become a preflight program,
and probably should one day. It is a separate piece of work with its own design, and bundling it here would
let it hide behind the two items that have a clear answer. Recorded in `ROADMAP.md`, not started.

## 4. Acceptance criteria

1. `RELEASE-CHECKLIST.md` is tracked at the root, with no `.git-exclude/` paths in it, and
   `.git-exclude/release/RELEASE-CHECKLIST.md` no longer exists — **one copy, not two.** The directory is the
   state; a stale second copy would be the exact failure this project keeps writing rules about.
2. Every inbound reference to the old path is swept (`grep -rn 'release/RELEASE-CHECKLIST'`), including in
   `rfcs/` and `ROADMAP.md`.
3. The gate executes **all five** archives; `coverage:` reports five of five executed, and the log still
   states coverage explicitly rather than implying it.
4. **The QEMU and Windows/macOS legs are each shown failing** on a deliberately broken archive, as RFC 046's
   three demonstrations were. A new leg that has only ever passed has not been shown to work.
5. `README.md` carries working per-platform instructions for the prebuilt binary, and **each one is the
   command the gate executes** — not a second copy of it. If a platform's documented steps cannot be made to
   work, that is reported rather than papered over.
6. Measured wall-clock for the whole gate, all jobs, stated as a number.
7. No change to `src/`, `cli/`, `node/`, `python/`, or any other gate.

## 5. What this does not do

- It does not move anything else out of `.git-exclude/`. Correspondence, release records, reviews and review
  requests stay internal, and the `.gitignore` fix is what keeps them there on every clone.
- It does not change the release mechanics themselves — only where they are written down and whether the
  instructions we publish are checked.
- It does not revisit the archive wrapper directory, which remains the owner's deferred decision.

---

## 6. Owner decision, 2026-09-25

**Accepted as proposed.**

**Split in two, because the halves need different hands:**

- **§3.1, the checklist move — the architect's, done immediately.** It is a governance document and the
  judgement in it is which pointers describe something internal and which are simply paths. That is not
  mechanical and should not be handed over as if it were.
- **§3.2, the gate's platform coverage and the README instructions — the dev team's.** Handoff:
  `rfcs/handoffs/047-release-process-in-the-repository/platform-coverage-handoff.md`.

The two are independent; §3.2 does not wait on §3.1.

**Criterion 1 binds the first half:** one copy of the checklist, not two. A tracked copy alongside the
`.git-exclude/` original would be the exact failure this project keeps writing rules about.
