# Implementation handoff — RFC 034 · PyPI: a declared wheel matrix

**To.** Implementer (mid-capability model)
**From.** Architect
**RFC.** [`rfcs/done/034-pypi-declared-wheel-matrix.md`](../../done/034-pypi-declared-wheel-matrix.md) — **Accepted 2026-09-16 by the owner**
**Owner decisions.** Floor **CPython 3.10**, raised **only** on the RFC's §6.1 triggers. **No** free-threaded wheels. **No** PyPy wheels. sdist still published.
**Milestone.** M3. **Must land before `2.3.0` is cut** — the next PyPI upload is irreversible.
**Size.** Medium. Release workflow, one new workflow, one shared script, manifests, CI job, one doc page.

---

## 0. Preconditions

None. RFC 033 is approved, and there is **no file overlap** with it or with the
engine RFCs. **This handoff is an instruction to start.**

## 1. Why

2.2.3 published 46 wheels whose Python/platform coverage was chosen by nobody —
`maturin build --find-interpreter` builds for whatever interpreters each runner
has. Python 3.8 on glibc x86_64 got no wheel despite `requires-python = ">=3.8"`;
macOS got nothing below 3.11. Nothing checks the set before or after upload. RFC
034 §2 has the full matrix.

## 2. Order of work

1. **§3 feasibility** — abi3 builds and the test suite passes against the wheel.
   **If not, stop and raise.**
2. **§4** the declaration and the shared check script, with local red proofs.
3. **§5** release workflow.
4. **§6** published gate.
5. **§7** CI floor testing.
6. **§8** docs, metadata, CHANGELOG.
7. **§9** one real dry run of `release-pypi.yaml`.

## 3. Feasibility — abi3

`python/Cargo.toml:15`: add `"abi3-py310"` to PyO3's features.

Build a wheel locally with `maturin build --release`, and confirm:

- the filename carries **`cp310-abi3`**;
- installed into a fresh **CPython 3.10** venv *and* a fresh **newest-stable** venv,
  `python/test_mdka.py` passes in both — run against the **installed wheel**, from
  a directory outside the workspace so `import mdka` cannot resolve to the tree.

⚠ **The test suite has only ever run on `3.x` — the newest Python.** It may use
syntax or stdlib APIs newer than 3.10. If it does, fix the **tests** to run on
3.10 (tests are not published). If the **binding** needs an API outside the
3.10 limited API, **stop and raise** — do not drop abi3 silently or lower
anything to make it build.

## 4. The declaration and the shared check

### 4.1 `python/wheel-matrix.toml`

As RFC 034 §4.1, with the decided values:

```toml
applies-from = "2.3.0"
python-floor = "3.10"
abi3         = true
platforms    = [
  "manylinux_2_17_x86_64", "manylinux_2_17_aarch64",
  "musllinux_1_2_x86_64",  "musllinux_1_2_aarch64",
  "win_amd64",             "macosx_11_0_arm64",
]
extra        = []
sdist        = true
```

**Write the three §6.1 triggers into a comment in this file**, as the only reasons
to raise `python-floor`, and that a raise happens only in a minor release with a
CHANGELOG entry. This file is where the next person to touch the floor will look.

**Confirm each platform tag from your §3 build, not from this list** — for the
platform you built on. For the others, §9's dry run confirms them. If the dry run
produces a different tag (e.g. a newer `manylinux` or `macosx` version), **stop and
report it** — it is a support change for users on older systems, and choosing to
accept it is a decision, not an edit.

### 4.2 `.github/workflows/scripts/check-wheel-matrix.py`

**One script, used by both §5 and §6**, so the pre-upload and post-upload checks
cannot disagree. Given a version and a list of filenames:

- expected = one `mdka-<version>-cp310-abi3-<platform>.whl` per platform, plus
  `mdka-<version>.tar.gz`;
- **fail on any missing file and on any unexpected file**, listing each.

⚠ **Traps, all real:**

1. **Compound platform tags.** manylinux wheels are named like
   `…-manylinux_2_17_x86_64.manylinux2014_x86_64.whl`. Match on the tag *set*
   parsed from the filename (split on `.`), requiring the declared tag to be a
   member — not string equality on the whole filename, and not a substring
   search (`manylinux_2_17_x86_64` must not match an `aarch64` wheel).
2. **Normalised names.** Wheel filenames use normalised project names and PEP 440
   versions. Parse with `packaging.utils.parse_wheel_filename` /
   `parse_sdist_filename` rather than hand-rolled splitting, if `packaging` is
   available in each context; if not, say what you did instead.
3. **The tag's version is not the source's version on a dry run.** On a tag, also
   assert tag == `pyproject.toml` version. On dispatch, take the version from
   `pyproject.toml`.

### 4.3 Red proofs — local, no workflow needed

Feed the script fabricated filename lists:

| Case | Expect |
|---|---|
| exact declared set | EXIT 0 |
| one platform missing | EXIT 1, names it |
| one extra `cp311-cp311-…` wheel | EXIT 1, names it |
| an extra `cp314t` wheel | EXIT 1 |
| sdist missing | EXIT 1 |
| `…manylinux_2_28_x86_64.whl` instead of `2_17` | EXIT 1 |
| aarch64 wheel only, x86_64 missing (substring trap) | EXIT 1 |

Capture with `EXIT:` inside the file.

## 5. `release-pypi.yaml`

1. In the four build jobs (`linux`, `musllinux`, `windows`, `macos`): **remove
   `--find-interpreter`**. Each job must produce **exactly one** abi3 wheel. Set up
   an interpreter at or above the floor as the build interpreter, and add a step
   after the build asserting `python/dist` holds exactly one `.whl`.
2. Add a step **between "Collect distributables" and "Publish to PyPI"** running
   §4.2 against `dist/`. It must run on **both** tag and `workflow_dispatch`, so a
   dry run exercises it.
3. `find … -exec cp -f` silently overwrites same-named files from different jobs.
   With one wheel per job it cannot collide — but the §4.2 check will catch it if
   it ever does, because a collision shows up as a missing file. Say so in a
   comment rather than restructuring.

**Do not touch** `verify-ci`, the `environment: pypi`, `id-token: write`, or the
publish condition. Trusted Publishing is keyed to this workflow **filename** —
do not rename it and do not convert it to `workflow_call`.

## 6. `pypi published gate` — new workflow

`.github/workflows/pypi-published-gate.yaml`. **Not in `ci.yaml`** — `verify-ci`
keys on `ci.yaml`, and a gate there can block the release that would turn it green
(RFC 020's correction record). Triggers mirror `npm-install-gate.yaml`: push,
pull_request, `workflow_dispatch`, weekly `schedule`.

1. Read the latest version from `https://pypi.org/pypi/mdka/json` and its file list.
2. **If version ≥ `applies-from`:** run §4.2 against the published filenames.
   **If older:** print a `::notice::` stating the version predates the declared
   matrix and the set check is **not applied** — not silent, not red. Today this
   branch runs, since 2.2.3 < 2.3.0; it ends by itself when 2.3.0 publishes.
3. Install **from PyPI** into fresh venvs at the **floor** (read from the toml)
   and **newest stable** (`3.x`) on `ubuntu-latest`:

   ```
   pip install --only-binary :all: "mdka==<version>"
   ```

   then import and convert, as the npm gate does.

⚠ **`--only-binary :all:` is the whole point.** The runner has Rust. Without the
flag, a missing wheel falls back to building the sdist, the install succeeds, and
the gate passes on exactly the defect it exists for.

**Today, the floor install runs against 2.2.3.** The architect checked the
published matrix: 2.2.3 has a `cp310` wheel for glibc x86_64, so the step should
pass now. If it does not, report it — do not weaken the step.

4. Assert the floor stated in `docs/src/getting-started/installation.md` equals
   `python-floor` (RFC 034 §4.6). Keep the match narrow and fail loudly if the
   sentence cannot be found, rather than passing because nothing matched.

### 6.1 Proofs

- `applies-from` branch: shown printing the notice against the live 2.2.3.
- The set check: shown red against a **fabricated** published list with one cell
  missing (call the script directly; do not fake PyPI).
- **`--only-binary :all:` matters:** in a scratch venv, request a combination with
  no wheel — e.g. `mdka==2.2.3` on CPython **3.8** under glibc x86_64, which has
  none — once **with** the flag (must fail) and once **without** (show it attempts
  an sdist build). If you cannot obtain that interpreter/platform, pick any
  published version/interpreter pair with no wheel and say which.
- `installation.md` floor check: red on a mismatched number, and red when the
  sentence is absent.

## 7. `ci.yaml` — test the floor

The `python` job tests `python-version: 3.x` only. Make it a matrix over
**`"3.10"`** and **`"3.x"`**, and make the venv use the matrix interpreter
(`uv venv --python …`, or equivalent — `uv venv` alone may pick a different one;
**print `python --version` inside the venv** to prove which ran).

YAML cannot read the toml, so add a step asserting the matrix's floor value equals
`python-floor`. Otherwise the two drift the first time someone raises one.

This job is what makes §6.1 trigger 1 announce itself: when CI can no longer
install 3.10, this goes red.

## 8. Metadata, docs, CHANGELOG

- `python/pyproject.toml`: `requires-python = ">=3.10"`. Classifiers: add
  `Programming Language :: Python :: 3 :: Only` and one per supported minor version
  only if you also add a check that keeps them in step with the floor — otherwise
  leave version classifiers out. **State which.**
- **Leave `pyproject.toml`'s two Japanese comments alone** — assigned to RFC 013 at
  the RFC 007 review.
- `docs/src/getting-started/installation.md:81`: floor, "CPython", the six
  platforms, and that other interpreters (PyPy, free-threaded CPython) install
  from the sdist, which needs a Rust toolchain.
- `CHANGELOG.md` `[Unreleased]`: new floor; free-threaded and PyPy wheels no longer
  published; what an affected user sees — **pip keeps 3.8/3.9 users on 2.2.x
  automatically; nothing already installed breaks** (RFC 034 §6.3); and that new
  CPython releases get a wheel from day one.

## 9. One real dry run

`gh workflow run release-pypi.yaml --ref main` — `workflow_dispatch` builds
everything and **does not publish** (`Publish to PyPI` is conditioned on a tag).
**Confirm that condition in the file before dispatching.**

Required outcome:

- all four build jobs green, **one wheel each**;
- the §5 assertion green on the real `dist/`;
- the artifact filenames recorded — these are the evidence that §4.1's platform
  tags are what the toolchain actually produces.

If a tag differs from §4.1, **stop and report** (§4.1).

## 10. Out of scope

- New platforms. npm, crates.io. The binding's source.
- Free-threaded support (thread safety, `gil_used`). Do run RFC 034 §3's import
  check on a `3.14t` interpreter if you can obtain one and **record** the result —
  do not act on it.

## 11. Acceptance checklist

- [ ] §3 abi3 wheel; tests pass on 3.10 and newest, against the installed wheel
- [ ] §4.1 toml with triggers comment; tags confirmed by build/dry run
- [ ] §4.2 shared script; §4.3 all seven cases, captured
- [ ] §5 `--find-interpreter` removed; one-wheel assertion per job; pre-upload check on tag and dispatch
- [ ] §6 gate workflow, own file; notice branch shown; set check red on fabricated list; `--only-binary` shown to matter; installation.md check red ×2
- [ ] §7 CI matrix 3.10 + 3.x; interpreter printed; floor-equality assertion
- [ ] §8 `requires-python`, classifiers decision stated, installation.md, CHANGELOG
- [ ] §9 dry run: 4 jobs × 1 wheel, assertion green, filenames recorded
- [ ] All our workflows green, including the new gate — count stated, Graph Update excluded

## 12. Report back

`.git-exclude/review-request/034-pypi-declared-wheel-matrix/README.md`, evidence
under `evidence/`, exit codes inside the files.
