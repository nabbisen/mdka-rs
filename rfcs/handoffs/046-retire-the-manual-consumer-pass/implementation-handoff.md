# Developer Handoff — RFC 046 · Gate the release archives; retire the manual pass

**RFC.** `rfcs/accepted/046-retire-the-manual-consumer-pass.md` — accepted by the owner, 2026-09-25
**Milestone.** Unassigned. CI and process only. **No library, binding or conversion change.**
**Priority.** P2. Nothing is broken today; a whole distribution channel is unverified
**Prepared.** 2026-09-25
**Baseline.** To be stated at dispatch — this handoff is **held until `2.6.0` is tagged and published**
**Order.** §2 before §3. The gate must exist and be shown failing before anything is removed from the checklist.

---

## 1. Why this exists

Every distribution channel has a gate that checks what the registry actually serves — except one.

```
$ grep -ln 'gh release download\|releases/download' .github/workflows/*.yaml
(no output)
```

`npm install gate`, `pypi wheel gate`, `pypi published gate` and `crates package gate` each fetch the
published thing. **Nothing fetches a GitHub release archive**, extracts it or runs the binary inside it.
RFC 042's contract gate runs `check-artifact-contract.py` against the binary the build just made
(`release-executable.yaml`, step *"Assert the built binary satisfies the artifact contract"*) — which is not
the same object as the archive a user downloads, and says nothing about the archive's shape.

That channel is where the wrapper-directory defect came from: the README told readers to extract and run,
and the archive's layout made the instructions fail.

## 2. The gate 🛑

A new workflow, `release-artifact-gate.yaml`, on the pattern of `npm install gate` — **its own file, not in
`ci.yaml`.** The release workflows gate on `ci.yaml`'s conclusion, so an artifact gate living there can block
the very release that would make it pass. That deadlocked `2.2.1`; the existing gates all carry this comment
and the reason has not changed.

**It runs against the latest published release**, like `npm install gate` does — so before a release it
describes the previous version, which is the same standing explanation the other gates have.

For each of the five assets:

1. **Download it from GitHub Releases.** `gh release download`, not a build artifact.
2. **Follow `README.md`'s own instructions**, which today are: extract, then `cd` into the folder the
   archive created, then run `./mdka`. **Assert the instructions succeed — do not assert a particular
   layout.** The wrapper directory is deliberate and the README documents it; if someone changes the layout,
   the gate should fail because the *documented* steps stopped working, not because the shape changed.
   Derive the folder name from the asset name rather than hardcoding it.
3. **Run the binary and check the output.** `echo '<h1>Hello</h1>' | ./mdka` must give `# Hello` — the
   README's own example, so the gate and the page cannot drift.
4. **Apply RFC 042's contract to the downloaded binary**, reusing
   `.github/workflows/scripts/check-artifact-contract.py` unchanged. **Do not write a second contract.**

**On the three you cannot execute:** the runner is Linux x64. `Linux-x64-gnu` and `Linux-x64-musl` run
natively; `Linux-aarch64-musl`, `macOS-aarch64` and `Windows-x64` cannot. **Do steps 1, 2 and 4 for all five
and step 3 only where it can run**, and make the log say which is which. A gate that silently skips three of
five assets is worse than one that states its coverage. If QEMU or a macOS runner is cheap enough to add,
say so with the cost rather than assuming the answer — see §6.

## 3. Show it failing 🛑

**Three demonstrations, with output pasted.** A gate that has only ever passed has not been shown to work.

1. **A layout the README's instructions cannot follow** — repack an archive without the wrapper folder and
   watch step 2 fail.
2. **A contract mismatch** — point the contract at a glibc floor the binary does not satisfy, and watch
   step 4 fail. This is the defect that stayed live for a dozen releases.
3. **A binary that does not run** — truncate or corrupt it, and watch step 3 fail.

Restore each afterwards and show the gate green.

## 4. Then, and only then, amend the process

- `rfcs/done/027-verification-discipline.md` — amend **Rule 1**. It is superseded, not deleted; say by what,
  and leave the history readable. RFC 027 is in `done/`, so this is an amendment section at the end, in the
  style RFC 043 §8 and RFC 045 §5 use, **not** a rewrite of the original text.
- `.git-exclude/release/RELEASE-CHECKLIST.md` — replace **§4** with a pointer to the gates, plus RFC 046 §3.4's
  voluntary step (convert a real page and read it; not blocking, not assigned). **No step that says
  "someone should".** Copy RFC 046 §4's four limits in verbatim, so the next reader sees what relying on
  bekoedit does and does not cover.
- Remove **RFC 042 criterion 4** from the checklist as a manual step: this gate is what satisfies it from
  now on. Say so where it used to be.

## 5. Criteria

1. `release-artifact-gate.yaml` exists, in its own file, and passes against the published `2.6.0` assets.
2. All five assets get steps 1, 2 and 4; step 3 runs where the runner allows and **the log states the
   coverage** rather than leaving it implicit.
3. The three failure demonstrations of §3, with pasted output, then green again.
4. `check-artifact-contract.py` is reused unchanged — one contract, not two.
5. RFC 027 Rule 1 amended; checklist §4 replaced; RFC 042 criterion 4 removed as a manual step; RFC 046 §4's
   limits written in.
6. No change to `src/`, `cli/`, `node/`, `python/` or any existing gate. Test counts unchanged: state them
   with the command beside each (`cargo test --workspace`, not `cargo test`).

## 6. Not in this slice, and one thing to raise rather than decide

- Changing the archive layout. The wrapper directory is the owner's deferred decision and this gate records
  the current documented behaviour; it does not rule on it.
- Weakening or merging any existing gate.
- `npm-install-gate.yaml`'s single-platform coverage. Same shape of gap, tracked separately in `ROADMAP.md`.

**Raise, do not decide:** if covering `Linux-aarch64-musl` under QEMU, or macOS/Windows on their own
runners, is worth the minutes — **price it cumulatively.** Seven workflows already run on every push. Bring
the number and the owner will choose; do not add runners on your own judgement, and do not quietly leave
three assets unexecuted without saying so in the log.

## 7. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.

Report to `.git-exclude/review-request/046-release-artifact-gate/README.md`, leading with §3 — the three
failure demonstrations are the deliverable. The passing run is the easy half.
