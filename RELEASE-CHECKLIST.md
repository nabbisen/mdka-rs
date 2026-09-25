# Release checklist

**Status.** Active from `2.2.2`. Built for RFC 027 Rule 1; **§4 replaced by RFC 046** (2026-09-25).
**Where this lives.** In the repository, since RFC 047 (2026-09-25). It was kept in the internal governance
directory until then, ignored by a rule that was per-clone — so it had no history, no review and existed on
one machine. It is process, not correspondence: less revealing than the RFCs and the roadmap this project
already publishes, and it now gets reviewed in the same commit as the change it describes.
**What is *not* here.** Release records, review requests, reviews and upstream correspondence stay internal.
This file names no internal path; where one is needed it says what the document is.
**Scope.** Every release. Follow it in order; nothing here is optional without
saying so in the release record.
**Written.** 2026-09-16, from the mechanics proven across `2.1.7`, `2.1.8`,
`2.2.0` and `2.2.1`, plus the consumer pass RFC 027 adds.

This captures what we already do. It invents nothing except §4, which exists
because an external auditor found in one pass what four milestones of internal
review did not — and the difference was **position**, not diligence.

---

## 1. Before the tag

- [ ] Every RFC in the release is implemented, reviewed and approved.
- [ ] `sh version.sh --list` shows all four crates at the target version, and
      `version.sh`'s own post-update assertion reported no manifest retaining
      the previous one.
- [ ] `CHANGELOG.md` has a section for the version, describing **user-visible
      effect**, not commit subjects.
- [ ] RFCs moved to `rfcs/done/`, `rfcs/README.md` updated in the same commit,
      inbound references swept (`grep -rl 'NNN-slug.md' rfcs/`).
- [ ] `ROADMAP.md` reflects what this release actually contains.
- [ ] Working tree clean; local `HEAD` and `origin/main` agree.
- [ ] **If this release introduces a new npm package, it must already exist on the registry with trusted
      publishing configured — before the tag.** npm trusted publishing is attached to a package, and a
      package that does not exist cannot hold that configuration, so CI **cannot create it**: the publish
      fails with `404 Not Found - PUT`, because npm answers 404 rather than 403 for a resource a credential
      cannot reach. The owner must publish each new package once by hand and then configure its publisher.
      Publishing a new *version* of an existing package proves nothing about this, and neither does checking
      that the manifest carries `publishConfig.access: public` — that was checked, and the release still
      failed twice.
      Also note `napi pre-publish` **aborts on the first failure** and walks `napi.targets` in order, so one
      unpublishable package strands every target after it. `2.5.0` announced six npm platforms, created none
      of the three new ones, and left `mdka` on npm at `2.4.2` for two releases.
- [ ] **CI green on the exact commit to be tagged**, checked with the **full
      SHA** — `gh run list --commit <full-sha>`. An abbreviated SHA silently
      returns empty, which has bitten this project twice.
- [ ] The consumer-artifact gates are green, or each red one is explained:
      `npm install gate`, `pypi wheel gate`, `pypi published gate`,
      `crates package gate`, `docs example gate`, `release artifact gate`.
      **Two standing explanations, and no others:**
      1. The npm gate reports on the **last published** release, so before a
         release it describes the previous version.
      2. **`release artifact gate` may legitimately be RED on a release-prep
         commit**, and this is expected at most releases. It checks the
         *published* archives against the *tree's* `README.md`,
         `artifact-contract.toml` and build matrix — so any prep commit that
         tightens the contract or updates the Quick Start makes the two
         disagree until the tag publishes. It is an acceptable explanation
         **only when every failure is a contract or README mismatch traceable
         to a change in that same commit**; anything else is a finding. Had it
         existed at `2.6.0`, the prep commit would have been red on exactly
         this — the published `2.5.1` binary needed `GLIBC_2.34` while the tree
         already said `2.17`. **The check that settles it is the post-publish
         dispatch in §4**, not this one.
- [ ] **`crates package gate` is green.** It has no standing exception. Until
      RFC 030 it failed at every release commit, because packaging the
      dependents separately made each resolve `mdka` from crates.io at a
      version not yet published — and that failure skipped `mdka-node` and
      `mdka-python` entirely, so its red hid the fact that it was checking two
      fewer crates than it claimed. A red here is a finding, not a known issue.

### Checkpoint

- [ ] Raise a pre-tag checkpoint to the owner and wait for the go-ahead.
      Tagging publishes to crates.io, which is irreversible. This is a
      re-confirmation immediately before the irreversible step, not a
      re-opening of the decision — and it has caught something real three
      times.

## 2. The tag

- [ ] **Tag the commit that was pushed as the tip.** GitHub only runs `ci.yaml`
      against a push's tip commit, so an intermediate commit has no CI run and
      `verify-ci` will fail closed.
- [ ] Annotated tag, **no `v` prefix**: `git tag -a X.Y.Z -m X.Y.Z <full-sha>`
- [ ] `git push origin refs/tags/X.Y.Z`
- [ ] `git ls-remote --tags origin refs/tags/X.Y.Z^{}` resolves to the intended
      commit.
- [ ] **Do not create the GitHub release by hand.** `create-release.yaml` does
      it on tag push and then dispatches the four publishers. A hand-made
      release collides with its `gh release create` and additionally fires
      `release: created`, racing the dispatch.

## 3. Watch the five runs

`Create Release`, then `Crates`, `npm`, `PyPi`, `Executable`.

- [ ] `verify-ci` passes in each — it is the enforcement point, and it keys on
      `ci.yaml` specifically.
- [ ] **Verify each registry directly — and poll, do not check once.** Workflow
      status is not publication, *and publication is not immediate*: npm's
      provenance-signed publishes took ~7 minutes to become available at
      `2.2.2`, during which the main package 404'd while its platform packages
      were already current — the exact shape of the `2.2.1` failure. Read the
      publish log before concluding: `+ pkg@version` plus *"being processed"*
      means it worked. Poll to a timeout, then escalate.
      `cargo search` / the crates.io API for all four crates, `npm view mdka
      version` plus each `@mdka/lib-*` platform package, and the PyPI JSON API.
- [ ] Stop and raise rather than improvise on: a 402 or access error, any
      failure that is not obviously "already exists"-shaped, or **partial
      platform publication** — some platform packages up and others not is the
      worst state to leave, because `optionalDependencies` then resolves on
      some platforms and not others.

## 4. Consumer verification — RFC 046 (replaces RFC 027 Rule 1's manual pass)

**There is no manual consumer pass.** Each part of it is a gate that runs on every push, on a clean machine,
with a record. Read them; do not repeat them by hand.

| What the pass used to check | The gate | Where it runs |
|---|---|---|
| `npm install mdka` works, from the registry | `npm install gate` | every push |
| `pip install mdka` works and the wheel is complete | `pypi wheel gate`, `pypi published gate` | every push |
| `cargo install` / the crate builds standalone | `crates package gate` | every push |
| Every runnable example in the docs, each in a fresh process | `docs example gate` | every push |
| **The GitHub Releases archives**: extract, `cd` into the folder, run `./mdka`, on the README's own example; and RFC 042's contract over the *downloaded* binary | **`release artifact gate`** | every push, against the **latest published** release |

- [ ] Confirm every per-push workflow is green on the tag's tip commit (eight now, with `release artifact gate`, which describes the *previous* release until this one is published; §3 already watches the release runs).
- [ ] **After the release is published**, dispatch `release artifact gate` and read it: `gh workflow run
      "release artifact gate"`, then `gh run watch`. It reports on the last *published* release, so this is the
      first moment it describes the new one. **Read its `coverage:` block** — it lists which assets were
      executed (Linux x64 gnu and musl) and which got only steps 1, 2 and 4 (Linux aarch64 musl, macOS, Windows).
      That is the stated coverage, not an omission.

**RFC 042 criterion 4 is no longer a manual step.** "Run the artifact contract over the *downloaded published*
artifacts" is step 4 of that gate, for all five CLI archives, every time. The npm addons are covered by
`release-npm.yaml`'s pre-publish gate (RFC 042 §3), which a real tag first exercises. Anywhere this used to say
the check was owed by hand, it is now this gate.

### The one voluntary step — owned by nobody, not blocking

Convert a real web page — something substantial, with images and links — and read the Markdown as a document.
It costs two minutes and it is where `A-01` (linked images converted wrongly) came from: a defect that passes
every assertion this project has written, because a gate can only assert what someone thought of. Record what
you converted if you do it. Nothing here waits on it.

### What relying on bekoedit does not cover

bekoedit is the project's consumer channel for reading converted output, and they report well. They are also
**one** consumer (RFC 046 §4, verbatim):

- **One mode.** They use `Minimal`. Four other modes get no reading.
- **One input class.** Editor-generated paste HTML. Not documentation, not reference pages, not scraped web
  content.
- **One binding.** The Rust crate. Not the CLI, not npm, not PyPI.
- **Late, and on their schedule.** They pin `= 2.5.1` exactly and adopt deliberately — *"their current
  release, then two fixes of their own, then the paste path. No date."* A defect shipped now may be reported
  after several further releases.

The coverage claim is *"one consumer, one mode, eventually"*, not *"we have a consumer channel."* These are the
price of retiring the manual pass, and the price is defensible; it must not be forgotten.

## 5. Correspondence held for this release

Some letters to downstream consumers are written in advance and held for a trigger — usually "when the
version that fixes their report is published". A held draft with a trigger and no owner is a draft that never
goes out: one slipped three consecutive releases before this section existed.

- [ ] **Check every upstream's draft folder** in the internal correspondence record and send anything held
      for this cut. **Sending is always the owner's decision**; the architect never writes outward on its own
      initiative.
- [ ] **Move what was sent out of the draft folder**, so the next release does not re-send it.
- [ ] A draft must be sendable **exactly as it stands**, by anyone, with nothing to strip first. Internal
      notes live in a separate folder outside the send tree — they were twice mailed out with the letter
      before that rule existed.
- [ ] **A pause is not a trigger.** If correspondence is deferred to a later release, the draft written
      around *this* one is superseded, not held: it will not be true by then. Move it to the suspended folder
      and write the new letter when the time comes.

**What is held right now, and for which release, is state rather than process**: it lives in the internal
release-state record, not here, so that this file does not go stale between releases.

## 6. After

- [ ] Record the release in the internal release record, one directory per
      version, with the evidence gathered above — the full SHA, the registry
      results, and what was verified at the destination rather than inferred
      from a green workflow.
- [ ] Update the internal release-state record: what shipped, what is carried
      forward, and anything this release closed.
- [ ] Deprecate superseded broken versions where applicable — e.g.
      `npm deprecate`. Registry rights are the owner's.
- [ ] Open the next milestone's RFCs, or confirm the current one closed.

---

## Why each part of §4 exists

| Part | The defect it would have caught |
|---|---|
| `npm install gate` | `npm install mdka` was unusable for twelve releases while CI was green — CI verified the tree, never the artifact |
| `release artifact gate` (the README's steps) | The release archive's wrapper directory, and a clone instruction naming a directory that does not exist — the README told readers to extract and run, and the layout made the instructions fail |
| `release artifact gate` (the contract over the download) | The CLI binary that required `GLIBC_2.34` for a dozen releases while every control was green (RFC 042) |
| `docs example gate` | `D-12`/`D-13`: examples that failed on their first line; `usage-python.md:28` using a variable defined at `:11` |
| The voluntary step | `A-01`: obvious on sight, invisible to every assertion we have written |
