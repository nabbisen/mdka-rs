# RFC 046 — Retire the manual consumer pass; gate the one channel that has none

**Status.** Accepted — owner, 2026-09-25
**Author.** Architect
**Created.** 2026-09-25
**Milestone.** Unassigned. Process and CI; no conversion change.
**Source.** Owner challenge, 2026-09-25: *"Why did CI not work properly? Manual operation or local effort (not automated) contains risk around cost burst and quality stability. As to consumer, we have a channel to bekoedit."*
**Amends.** RFC 027 Rule 1, and §4 of the release checklist.
**Relates to.** RFC 020, 026, 031, 032, 034 (the gates that already exist), RFC 042 (artifact contents).

---

## 1. The challenge is correct, and the evidence is ours

RFC 027 Rule 1 requires a manual consumer pass before every release, performed by someone who did not
implement it. The argument was *position, not diligence*. That argument still holds for one part of the pass
and **has been overtaken by automation for the rest** — by gates this project built in response to the very
findings that justify the rule.

| What the pass found, historically | Automated since |
|---|---|
| `npm install mdka` unusable for twelve releases | **`npm install gate`** — installs the *published* package from the registry (RFC 020, 026) |
| Getting-started examples failing on their first line (`D-12`, `D-13`) | **`docs example gate`** — executes every non-fragment fenced block (RFC 031, 032) |
| `usage-python.md:28` using a variable defined at `:11` | Same gate, **each example in a fresh process** (RFC 032) |
| A crate that builds in-workspace and fails standalone | **`crates package gate`** (RFC 026 §4.2) |
| What PyPI actually serves, versus what we declared | **`pypi wheel gate`** and **`pypi published gate`** (RFC 026 §4.1, RFC 034 §4.4) |

**Five of the six are gates now.** Re-performing them by hand at each release is duplicated cost with worse
properties: one machine, one time, no record, and a different scope each release depending on who runs it.
The checklist's own warning — *"that is how a checklist becomes a formality"* — applies to the manual pass
more than to anything else in it.

## 2. What is genuinely not covered — verified, not assumed

**Two things. One is automatable and one is not.**

### 2.1 The GitHub Releases archive has no gate at all

crates.io, npm and PyPI each have a gate that fetches what the registry actually serves. **GitHub Releases
does not.** Checked rather than recalled:

```
$ grep -ln 'gh release download\|releases/download' .github/workflows/*.yaml
(no output)
```

No workflow downloads a published archive, extracts it, or runs the binary inside it. RFC 042's contract
gate asserts artifact contents **before publish**, on the files the build produced — which is not the same
object, and its criterion 4 (the same checks over *downloaded published* artifacts) is currently a one-off
manual step owed at each release.

This is the channel that produced the wrapper-directory finding: the README told readers to extract and run,
and the archive's shape made those instructions fail. **That is a gate, not a judgement.**

### 2.2 Reading converted output as a document

`A-01` — linked images converted wrongly — passes every assertion this project has written, to this day, and
is obvious within seconds of reading real output. **No gate can find this class**, because the defect is
"the result is wrong in a way we did not think to assert", and a gate can only assert what someone thought
of.

## 3. Proposal

1. **Retire RFC 027 Rule 1's manual pass as a per-release requirement.** It is replaced, not abandoned: §1's
   five gates are the mechanical half, and they run on every push rather than once per release.
2. **Add a `release artifact gate`** covering §2.1: download each published archive from GitHub Releases,
   verify the extracted layout against the README's own instructions, run the binary, and apply RFC 042's
   contract to the downloaded file. **This makes RFC 042 criterion 4 permanent** instead of a manual step
   repeated at every release — which is the same substitution this RFC makes everywhere else.
3. **Treat bekoedit's adoption report as the §2.2 channel**, with its limits written down rather than
   assumed (§4).
4. **Keep one voluntary step, owned by nobody:** convert a real page and read it. Not a checklist item, not
   blocking, not assigned. It costs two minutes and it is where `A-01` came from.

## 4. What relying on bekoedit does not cover — stated so it is not discovered later

They are a real consumer channel and they report well. They are also **one** consumer:

- **One mode.** They use `Minimal`. Four other modes get no reading.
- **One input class.** Editor-generated paste HTML. Not documentation, not reference pages, not scraped web
  content.
- **One binding.** The Rust crate. Not the CLI, not npm, not PyPI.
- **Late, and on their schedule.** They pin `= 2.5.1` exactly and adopt deliberately — *"their current
  release, then two fixes of their own, then the paste path. No date."* A defect shipped now may be reported
  after several further releases.

**None of these is an argument against the proposal.** They are the price of it, and the price is
defensible: the alternative costs a manual pass every release and, on the evidence of §1, mostly re-runs
work that automation now does better. But the coverage claim must be *"one consumer, one mode, eventually"*,
not *"we have a consumer channel."*

## 5. Acceptance criteria

1. RFC 027 Rule 1 amended, and the release checklist's §4 replaced by a pointer to the gates plus §3.4's
   voluntary step. **No step that says "someone should".**
2. A `release artifact gate` exists, downloads every published archive for the latest release, and fails on:
   a layout the README's instructions cannot follow, a binary that does not run, or a contract mismatch.
3. **Demonstrated failing** on the wrapper-directory shape and on a deliberately mismatched glibc floor,
   with output. A gate that has only ever passed has not been shown to work.
4. RFC 042 criterion 4 is satisfied **by that gate**, and is removed from the release checklist as a manual
   step.
5. §4's four limits are written into the checklist where the consumer pass used to be, so the next reader
   sees what is and is not covered.

## 6. What this does not do

- It does not remove any existing gate, or weaken RFC 042's pre-publish contract.
- It does not ask bekoedit for anything. Their reports are volunteered and stay volunteered.
- It does not claim the manual pass was worthless. It found five real defects; four of them became the gates
  that make it redundant, which is the best outcome a manual control can have.

---

## 7. Owner decision, 2026-09-25

**Accepted as proposed.** Nothing in §1–§6 changes.

**Sequencing: the handoff is dispatched after the `2.6.0` tag**, for the same reason as RFC 044 — a push to
`main` during release prep moves the tip that has to be tagged. Here it also happens to be the better order
on the merits: **the gate in §3.2 needs published archives to be built against**, and `2.6.0` produces a
fresh set. The one-off RFC 042 criterion 4 check performed at this release becomes the reference data the
gate is written from, which is the last time that check is done by hand.

**`2.6.0` gets no consumer pass, manual or otherwise** — decided the same day, on the same reasoning.
