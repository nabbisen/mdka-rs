# Developer Handoff — RFC 047 §3.2 · Execute all five archives, and write the README from what that proves

**RFC.** `rfcs/accepted/047-release-process-in-the-repository.md` — accepted by the owner, 2026-09-25
**Milestone.** Unassigned. CI and documentation. **No library, binding or conversion change.**
**Priority.** P2. Nothing is broken; three of five published archives are executed by nothing, and Windows users are given no command at all
**Prepared.** 2026-09-25
**Baseline.** `82fd657` — 605 Rust (`cargo test --workspace`), 42 Node, 25 loader, 91 Python; eight workflows green
**Scope.** `release-artifact-gate.yaml`, and the README lines its new jobs prove. **§3.1 is already done** — the checklist is tracked at `RELEASE-CHECKLIST.md`; do not touch it.
**Credit.** Your RFC 046 report priced this and hedged it correctly. The hedge is why it is happening.

---

## 1. Why this is cheap, and why your estimate was right to hedge

You wrote: *"macOS runners start slowest and are the scarce kind; **a private repository** would bill them at
a multiple."*

```
$ gh repo view --json visibility -q .visibility
PUBLIC
```

GitHub documents standard hosted runners as **free for public repositories**, so the cost here is wall-clock,
not money — and your measured gate job is **10 seconds**. **Confirm that against current billing
documentation before you add jobs**, and if it has changed, stop and say so rather than adding them anyway:
the whole justification is that hedge turning out not to apply.

## 2. The three archives nothing executes

`Linux-aarch64-musl`, `macOS-aarch64` and `Windows-x64` get steps 1, 2 and 4. Step 3 — run the binary on the
README's own example — is skipped, and the `coverage:` block says so honestly. Close it:

1. **`Linux-aarch64-musl` under QEMU**, in the existing Linux job. **You have already shown this works** —
   you ran that exact published binary under `qemu-aarch64` by hand and got the README's output. This is the
   cheap one; do it first.
2. **`macos-latest`** and **`windows-latest`** jobs, each running steps 1–4 for its own archive with
   `check-release-archives.py` **unchanged in substance**. One checker, as with the contract script.

**Keep the coverage block truthful whatever happens.** If a leg cannot run, it must still print why. A gate
that quietly drops an asset is worse than one that states its limits — that was the point of your original
design and it does not change because the limits shrink.

## 3. The README, written from the runner and not from anyone's memory 🛑

**This is the deliverable, not the jobs.** A Windows user is currently given a POSIX pipeline into `./mdka`
while their archive contains `mdka.exe`. I was going to write a Windows line myself; I have no Windows
machine, and an untested instruction in the Quick Start is exactly the defect class RFC 046 exists to catch.

So: **make the Windows job run the documented steps, then write the documentation as the steps that passed.**
Not the reverse.

- The checker parses the README's example with ``re.match(r"echo '(.*)' \| \./mdka\s*$", line)`` and takes the
  **first** match. A second, differently-shaped block will not be picked up by that regex — so **extend the
  parser deliberately** to find each platform's block, rather than letting a new block be silently ignored.
  A README the gate does not read is the drift this gate exists to prevent.
- **macOS: do not assume it just works.** A zip downloaded from a browser carries `com.apple.quarantine`, and
  the binary is unsigned. The runner's `gh release download` may not reproduce what a human browser does.
  **Report what you actually observe.** If a real user needs an extra step, that step belongs in the README;
  if the runner cannot reproduce the human case, say that plainly rather than documenting a guess.
- **If a platform's documented steps cannot be made to work, that is a finding, not an obstacle.** Report it
  and stop. Do not paper over it with a command you have not seen succeed.

## 4. Criteria

1. The gate executes **all five** archives; `coverage:` reports five of five, and still states coverage
   explicitly rather than implying it.
2. **Each new leg shown failing** on a deliberately broken archive — QEMU, macOS and Windows separately —
   with output, then green again. Three new demonstrations, in the style of your RFC 046 three. **A leg that
   has only ever passed has not been shown to work.**
3. `README.md` carries working per-platform instructions for the prebuilt binary, and **each is the command
   the gate executes** — parsed from the README, not a second copy living in the script.
4. The billing confirmation of §1, stated as what you checked and when.
5. **Measured wall-clock for the whole gate, all jobs**, as a number. Your 10-second measurement replaced my
   estimate; do the same here.
6. `check-artifact-contract.py` still unchanged. No change to `src/`, `cli/`, `node/`, `python/`, or any
   other gate. Test counts unchanged, with the command beside each (`cargo test --workspace`, not
   `cargo test` — those differ by 27).

## 5. Not in this slice

- `RELEASE-CHECKLIST.md` — §3.1 is done and the file is tracked now.
- The archive wrapper directory. The gate records the documented behaviour; changing the layout is the
  owner's deferred decision.
- A preflight script for checklist §1. Recorded in `ROADMAP.md`, deliberately out of RFC 047 (§3.3).
- `npm-install-gate.yaml`'s single-platform coverage — same shape, tracked separately.

## 6. Committing and pushing

Only work that is yours and approved. Report first; a green run is not approval. Tagging, releasing and
triggering release workflows are not yours.

**One thing worth knowing rather than acting on:** this gate is *expected* to be red on a release-prep
commit, because it checks published archives against the tree's README and contract. That is now the second
standing explanation in `RELEASE-CHECKLIST.md` §1. If you see it red on someone else's release-prep push,
that is why — and it is only acceptable when every failure traces to a change in that same commit.

Report to `.git-exclude/review-request/047-platform-coverage/README.md`, leading with §3 — the README lines
and the runs that prove them. The jobs are the easy half.
