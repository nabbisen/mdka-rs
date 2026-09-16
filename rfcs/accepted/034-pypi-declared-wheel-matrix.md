# RFC 034 — PyPI: a declared wheel matrix, built on purpose and checked where it is published

**Status.** Accepted (2026-09-16, owner) — floor 3.10, raised only on the §6.1 triggers; no free-threaded or PyPy wheels
**Author.** Architect
**Created.** 2026-09-16
**Milestone.** M3 — **must land before `2.3.0` is cut** (§8)
**Source.** RFC 032 review §6 — `.git-exclude/reviewed/032-gates-report-everything/README.md`
**Owner decisions.** §6 — all three decided 2026-09-16 (1 by accepting the §6.1 recommendation).

---

## 1. Summary

Which Python/platform combinations get a wheel on PyPI is **not decided by anyone**.
It is whatever interpreters each GitHub runner image happens to carry at release
time. The published 2.2.3 set breaks the documented promise *"Requires Python 3.8
or later"* on several platforms, nothing checks the set before or after upload,
and a runner image update can change it silently between two releases.

Replace accident with policy:

1. **Declare** the supported matrix in one checked-in file.
2. **Build** it deliberately — PyO3's stable ABI (`abi3`), one wheel per platform.
3. **Assert** the built set matches the declaration **before** the irreversible upload.
4. **Check** the set actually on PyPI, and install from it, on a schedule.
5. **Test** the lowest and highest declared Python in CI, so the promise is exercised.

## 2. The evidence

### 2.1 What 2.2.3 published

`https://pypi.org/pypi/mdka/2.2.3/json`, **46 wheels plus an sdist — 47 files**. (The 2.2.3 consumer pass reported "47 wheels"; that count includes the sdist.)

```
py      glibc-x64   glibc-arm64 musl-x64    musl-arm64  win-x64     mac-arm64
cp38    --          yes         yes         yes         --          --
cp39    yes         yes         yes         yes         --          --
cp310   yes         yes         yes         yes         yes         --
cp311   yes         yes         yes         yes         yes         yes
cp312   yes         yes         yes         yes         yes         yes
cp313   yes         yes         yes         yes         yes         yes
cp314   yes         yes         yes         yes         yes         yes
cp314t  yes         yes         yes         yes         --          --
cp315   yes         --          --          --          --          --
cp315t  yes         --          --          --          --          --
pp311   yes         yes         yes         yes         --          --
```

`python/pyproject.toml:11` — `requires-python = ">=3.8"`.
`docs/src/getting-started/installation.md:81` — *"Requires Python 3.8 or later."*

- **Python 3.8 on glibc x86_64 has no wheel**, while 3.8 on glibc *arm64* does.
- Windows has nothing below 3.10; macOS nothing below 3.11.
- `cp315` — a pre-release interpreter — got wheels on one platform only.

A user in a `--` cell gets the **sdist**, which needs a Rust toolchain to build.
For most Python users that is an install failure.

### 2.2 The cause

Every build job in `.github/workflows/release-pypi.yaml` runs:

```
maturin build --release --out dist --find-interpreter
```

`--find-interpreter` builds for every interpreter found on that runner. And
`python/Cargo.toml` enables PyO3 **without** `abi3`, so each wheel is tied to one
Python version. Coverage therefore equals *the runner's interpreters*, per job.

### 2.3 Nothing checks it, anywhere

| Where | What it checks | Published file set? |
|---|---|---|
| `ci.yaml` `python` job | tests on `python-version: 3.x` — **latest only** | no; floor never tested |
| `pypi wheel gate` | a wheel **built from the tree**, `pip install --no-index` | no |
| `release-pypi.yaml` publish | `find … -exec cp`, then `uv publish dist/*` | **no assertion before upload** |
| Release checklist §3 | PyPI JSON `info.version` | version only |
| Consumer pass | one `pip install` on the performer's machine | one cell |

**PyPI uploads are irreversible** — a filename can never be re-uploaded. The only
point where a wrong set can be prevented rather than lamented is before
`uv publish`, and there is no check there.

## 3. Facts about PyO3 0.28 that shape the design

Read from the vendored `pyo3-0.28.3` / `pyo3-build-config-0.28.3` sources:

- `abi3-py38` through `abi3-py314` features exist; minimum supported CPython is 3.7.
- **Free-threaded CPython, PyPy and GraalPy do not support abi3**
  (`impl_.rs`: *"PyPy, GraalPy, and the free-threaded build don't support abi3"*).
  Those need version-specific wheels, one per interpreter version.
- `python/src/lib.rs:329` is a plain `#[pymodule]` — **no `gil_used = false`**.
  Per PyO3's documented default, importing it on a free-threaded interpreter
  re-enables the GIL. So the free-threaded wheels shipped today deliver no
  free-threading. *Implementer to confirm by import on a `3.14t` interpreter.*

## 4. Design

### 4.1 One declaration — `python/wheel-matrix.toml`

```toml
# The supported PyPI wheel matrix. Read by release-pypi.yaml (before upload)
# and by the pypi published gate (after). Change it deliberately.
applies-from = "2.3.0"          # published versions before this predate the policy
python-floor = "3.10"           # §6 decision 1. Raise ONLY on a trigger in §6.1;
                                # never on the calendar. Minor releases only.
abi3         = true
platforms    = [
  "manylinux_2_17_x86_64", "manylinux_2_17_aarch64",
  "musllinux_1_2_x86_64",  "musllinux_1_2_aarch64",
  "win_amd64",             "macosx_11_0_arm64",
]
extra        = []               # no free-threaded, no PyPy wheels (§6, decided)
sdist        = true
```

Values shown are the recommendations in §6, not decided.

Platforms are **exactly today's six** — this RFC fixes coverage within them and
does not add platforms.

### 4.2 Build — abi3, one interpreter per job

- `python/Cargo.toml`: `pyo3 = { …, features = ["extension-module", "abi3-py310"] }`
  (floor from §6).
- `release-pypi.yaml`: **remove `--find-interpreter`**. Each job sets up one
  interpreter at or above the floor and builds one `cp310-abi3-<platform>` wheel.
- `requires-python` in `pyproject.toml` = the floor.

Result: **6 wheels + 1 sdist**, each abi3 wheel valid on every CPython from the
floor upward — including versions released *after* 2.3.0, which today get no wheel
until someone happens to release on a runner that has them.

**Feasibility first:** build the abi3 wheel and run the full Python test suite
against it. If any API the binding uses is outside the limited API, stop and
raise — do not work around it by dropping abi3 silently.

### 4.3 Before upload — assert the set

A step between *Collect distributables* and *Publish to PyPI* in
`release-pypi.yaml`: read `wheel-matrix.toml`, compute the exact expected
filenames for the tag's version, compare with `dist/`.

**Fail on a missing file and on an unexpected file.** An extra wheel is a cell
nobody declared, i.e. a support promise nobody decided to make.

This is the checkpoint before an irreversible step, which is where this project
puts its checks.

### 4.4 After upload — `pypi published gate`

A **new workflow file**, not `ci.yaml` (RFC 020's rule: a gate in `ci.yaml` can
block the release that would fix it). Triggers mirror `npm-install-gate.yaml`:
push, pull request, dispatch, weekly schedule.

1. Read the latest version from the PyPI JSON API.
2. **If it is ≥ `applies-from`:** assert the published file set equals the
   declaration — same function as §4.3, shared, so the two cannot disagree.
   **If it is older:** print, prominently, that the published version predates
   the declared matrix and the set check is not applied — **do not pass
   silently, and do not fail** a version the policy never governed. This state
   ends by itself when 2.3.0 publishes.
3. **Install from PyPI** — `pip install --only-binary :all: mdka==<version>` — in
   fresh venvs at the **floor** and at the **newest stable** CPython, on
   `ubuntu-latest`. Import and convert, as the npm gate does.

⚠ **`--only-binary :all:` is not optional.** The runner has a Rust toolchain; without
the flag, a missing wheel silently falls back to building the sdist, the install
succeeds, and the gate passes on exactly the defect it exists for.

The report lag is inherent and stated, as in the npm gate: this checks the **last
published** release.

### 4.5 Test the promise — `ci.yaml` `python` job

Matrix over the **floor** and **newest stable** CPython, instead of `3.x` alone. A
declared floor nothing tests is the same unexercised promise as today's `>=3.8`.

### 4.6 Documentation

`installation.md` states the floor, "CPython", and the six platforms. The
published gate asserts the floor written in `installation.md` equals
`python-floor`, so the page cannot drift from the declaration.

## 5. What this does not do

- Add platforms (macOS x86_64, Windows arm64, …). Separate decision.
- Declare free-threaded **support** in the thread-safety sense — that needs a
  review of the binding and `gil_used = false`, and is its own RFC if wanted.
- Change the npm or crates.io distribution.

## 6. Owner decisions

### Decisions 2 and 3 — **decided 2026-09-16**

> *"No niche support except basic package installation support is required."*

- **Free-threaded (`cp314t`, `cp315t`): not declared.** No wheels.
- **PyPy (`pp311`): not declared.** No wheels.

Both remain **installable** in the basic sense: the sdist is still published, so
`pip install mdka` on those interpreters builds from source where a Rust
toolchain is present. `installation.md` states that wheels are for CPython, and
that other interpreters fall back to a source build requiring Rust.

### Decision 1 — the Python floor

Owner, 2026-09-16: raising the floor is allowed; the goal is **stability**, with
balance between users' availability and our maintenance cost.

#### 6.1 Recommendation: floor **3.10**, raised only on a trigger

This replaces the draft's Option C (*oldest non-EOL CPython at each minor
release*), which on reflection is the **least stable** option: it moves the floor
every year by the calendar, dropping users and editing CI, docs and CHANGELOG each
time, whether or not anything required it.

**The floor moves only when one of these fires — never on a date:**

1. **CI can no longer install the floor Python.** §4.5 tests the floor on every
   push, so this surfaces as a red CI job, not as something to remember.
2. **A PyO3 version the project needs drops the floor.**
3. **The floor's limited API blocks a change the project needs.**

When one fires: raise to the **lowest** version that clears it, in a **minor**
release, with a CHANGELOG entry. Nothing else moves it.

#### 6.2 Why 3.10 — checked, not assumed

**CI can test all candidates today** — `actions/python-versions` manifest, fetched
2026-09-16: 3.8, 3.9, 3.10 and 3.11 are all installable on ubuntu-24.04 x64 and
arm64, macOS arm64 and Windows x64. So testability does not decide it yet.

**What developers get from their OS** — distribution package indexes, fetched
2026-09-16:

| Distribution | Default `python3` | Covered by 3.9 | 3.10 | 3.11 |
|---|---|---|---|---|
| Ubuntu 22.04 LTS (jammy) | **3.10.6** | ✔ | ✔ | **✘** |
| Ubuntu 24.04 LTS (noble) | 3.12.3 | ✔ | ✔ | ✔ |
| Debian 12 (bookworm) | 3.11.2 | ✔ | ✔ | ✔ |
| Debian 13 (trixie) | 3.13.5 | ✔ | ✔ | ✔ |
| RHEL 9, Amazon Linux 2023, macOS Command Line Tools | 3.9 — *general knowledge, **not** verified in-session* | ✔ | ✘ | ✘ |

| Floor | Users gained vs the next one up | Stability cost |
|---|---|---|
| 3.9 | system Python on RHEL 9 / AL2023 / macOS CLT *(unverified)* | EOL since October 2025; runner and PyO3 support will lapse first → **earliest forced change** |
| **3.10** | **Ubuntu 22.04 LTS system Python** — very common on developer machines and CI images, with standard support to April 2027 | EOL October 2026, but *upstream EOL is not a trigger*; the abi3 wheel keeps working and CI keeps testing it |
| 3.11 | — | drops Ubuntu 22.04's default Python today, for no present technical need |

**3.10 is the balance point.** It keeps the most common LTS developer platform
that 3.11 would drop, without taking on 3.9, whose support in tooling will
disappear soonest. The users 3.9 would add can obtain a newer CPython from their
platform's own channels (RHEL AppStream, Homebrew, `uv`) — a one-step change for a
developer, stated rather than assumed painless.

#### 6.3 What changes for users, concretely

| User | Today (2.2.3) | After (floor 3.10, abi3) |
|---|---|---|
| CPython 3.10–3.14, glibc/musl Linux, Windows x64 | wheel | wheel |
| **CPython 3.10, macOS arm64** | **no wheel** | **wheel** |
| **CPython 3.8, glibc x86_64** | **no wheel** — broken today | keeps 2.2.x; pip does not offer 2.3.0 |
| CPython 3.8/3.9, elsewhere | wheel | keeps 2.2.x |
| **A future CPython (3.15, 3.16 …)** | **no wheel until a release happens on a runner that has it** | **wheel from day one** — abi3 |
| Free-threaded, PyPy | wheel on Linux only | sdist; source build needs Rust |

pip will not install 2.3.0 on 3.8/3.9: it skips releases whose `requires-python`
excludes the interpreter and resolves 2.2.x instead. **No existing install
breaks.** Those users stop receiving new releases.

#### 6.4 Our maintenance cost, concretely

| | Today | After |
|---|---|---|
| Wheels per release | 46, coverage unchosen | **6**, declared |
| Python versions built | whatever each runner has | one per job |
| Python versions tested in CI | 1 (`3.x`) | **2** — floor and newest stable |
| Recurring floor work | none, because nobody decides | **none until a trigger fires** — and a trigger announces itself as a red CI job |

## 7. Acceptance criteria

- [ ] `python/wheel-matrix.toml` exists with the owner's decisions; `applies-from` set; **the three §6.1 triggers written in it as the only reasons to raise `python-floor`**
- [ ] abi3 wheel built and the **full Python test suite passes against the built wheel**
- [ ] `--find-interpreter` removed; `release-pypi.yaml` builds exactly the declared set
- [ ] **Pre-upload assertion observed red** for a missing wheel **and** for an extra wheel, in a dry run that does not publish
- [ ] Published gate: set check shares the pre-upload function; `applies-from` behaviour printed, not silent
- [ ] Published gate **observed red** on a simulated missing cell; **observed that `--only-binary :all:` matters** — without it, a missing-wheel case installs from sdist and passes
- [ ] `ci.yaml` `python` job tests floor and newest stable
- [ ] `requires-python`, `installation.md` and `python-floor` agree; the gate checks `installation.md`
- [ ] Free-threaded import behaviour confirmed on a `3.14t` interpreter (§3) — recorded for the future RFC, not acted on
- [ ] CHANGELOG: the floor, the dropped free-threaded and PyPy wheels, and what an affected user sees (§6.3)
- [ ] `installation.md`: floor, CPython wheels, six platforms, and the sdist fallback for other interpreters

## 8. Sequencing

**Before `2.3.0` is cut** — the next PyPI upload is the next irreversible chance to
publish another accidental set.

File overlap: `release-pypi.yaml`, `python/Cargo.toml`, `pyproject.toml`,
`ci.yaml` (`python` job), `installation.md`, one new workflow. **No overlap with
RFC 033** (docs gate script, `docs.yaml`, three Rust doc pages, `README.md`) or
with the engine RFCs' test and `src/` files. It can therefore follow RFC 033
directly, or interleave with the engine work; it should not wait until after it.
