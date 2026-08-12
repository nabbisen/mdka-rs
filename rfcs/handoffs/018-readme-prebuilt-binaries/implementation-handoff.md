# Developer Handoff — RFC 018 · README Quick Start: prebuilt binaries

**Governing RFC.** [RFC 018](../../proposed/018-readme-prebuilt-binaries.md) — Proposed
**Prepared.** 2026-08-12
**Sequencing.** **After `2.1.8` is fully published.** Do not touch `README.md` while the release is in flight.

---

## 1. Purpose

Quick Start currently offers only `cargo install mdka-cli` and states Rust is
required. Five prebuilt binaries ship with every release and go unmentioned, so
a visitor without a Rust toolchain has no way to try the tool.

Add a download-and-run path.

## 2. Wait for the release

`2.1.8` is being tagged and published. `README.md` is packaged into the
crates.io, npm, and PyPI artifacts, so editing it mid-release risks a mismatch
between what shipped and what the repository says.

**Start only once `2.1.8` is confirmed published on all four registries.**

## 3. Change scope

| Path | Change |
|---|---|
| `README.md` | Quick Start section only |

Nothing else. No `docs/`, no code, no workflows, no manifests.

## 4. Required implementation

A short subsection **before** the existing `cargo install` path — it is the
lower-friction route, and Quick Start exists to get someone running fast.

Cover four things, briefly:

1. **Where** — link `https://github.com/nabbisen/mdka-rs/releases/latest`.
   **Never a versioned asset URL**; it would break on every release.
2. **Which asset** — the five targets in §5, compactly. A small table or a
   tight list; do not pad.
3. **What extraction gives** — the binary sits at the archive root with no
   wrapper directory, so it is runnable immediately.
4. **A first command** — reuse the existing
   `echo '<h1>Hello</h1>…' | mdka` example so both paths converge on the same
   thing. Do not invent a second example.

Then rework the existing path so the sentence *"`cargo` (Rust language)
installed is required"* attaches to **that option**, not to trying the tool at
all. Right now it reads as a precondition for the whole section.

## 5. The five targets — take these from the workflow, not from me

| Target | Asset pattern |
|---|---|
| Linux x64 (glibc) | `mdka@Linux-x64-gnu-<version>.tar.gz` |
| Linux x64 (musl) | `mdka@Linux-x64-musl-<version>.tar.gz` |
| Linux aarch64 (musl) | `mdka@Linux-aarch64-musl-<version>.tar.gz` |
| macOS Apple Silicon | `mdka@macOS-aarch64-<version>.zip` |
| Windows x64 | `mdka@Windows-x64-<version>.zip` |

**Verify against `.github/workflows/release-executable.yaml`'s matrix**, which is
the source of truth — not against a past release's asset list, and not against
this table. If they disagree, the workflow wins and you should report the
discrepancy.

## 6. State what is *not* covered — this is the part most likely to be dropped

**There is no macOS Intel (`x86_64-apple-darwin`) build.** Only Apple Silicon.

An Intel Mac user following a download link finds nothing that runs. The README
must say so and point them to `cargo install`.

One sentence covering the general case is enough — something to the effect that
platforms not listed should use `cargo install mdka-cli`. That also covers
Windows ARM and Linux aarch64-gnu, both likewise unbuilt.

Do not quietly omit this. A download section that lists five targets and stays
silent about the gaps is the same failure mode as documentation that describes
options which do not work — the defect class this project has spent two
milestones removing.

## 7. Keep it short

The project rules require a concise README, and Quick Start already covers CLI,
Rust, Node.js, and Python.

If the section no longer fits on a screen, it has grown too far. Detail belongs
in `docs/src/getting-started/installation.md`, which is not in scope here.

## 8. Non-change scope — do not touch

- Any README section other than Quick Start. Not the badges, not Why mdka?, not
  Conversion Modes, not Learn More.
- `docs/` in its entirety.
- The release workflows. **Do not add build targets** — the missing macOS Intel
  build is a release-matrix decision for the project owner, recorded in RFC 018
  as a finding. Report, do not act.
- Japanese comments — RFC 007 and RFC 013.

### ⚠ `.github/workflows/create-release.yaml`

Still untracked, still not gitignored. Stage by explicit path:

```
git add README.md
git status        # confirm create-release.yaml is still untracked
```

## 9. Required verification

- Every asset name matches the workflow matrix — show the comparison.
- The `/releases/latest` link resolves.
- **Actually follow your own instructions on this machine**: download the
  Linux x64 asset from the latest release, extract it, and run the example
  command. If the steps you wrote do not work as written, they are wrong.
  This is the standing lesson — verify in an environment resembling the reader's,
  not by re-reading what you wrote.
- `cargo test --workspace --locked` — unchanged. No code is touched, so the
  count must not move.
- `cargo fmt --check`, clippy — both clean.

## 10. Required evidence

1. The Quick Start section as published, in full.
2. Asset names compared against the workflow matrix.
3. Transcript of downloading, extracting, and running the binary per §9.
4. Test count, unchanged.
5. `git status` showing `create-release.yaml` still untracked.

## 11. Acceptance checklist

- [ ] Prebuilt-binary path precedes the `cargo install` path
- [ ] All five targets listed, names verified against the workflow matrix
- [ ] Link is `/releases/latest`, not a versioned asset URL
- [ ] macOS Intel explicitly noted as not covered, with `cargo install` fallback
- [ ] Rust requirement now attaches to the `cargo install` option specifically
- [ ] Extraction behaviour (binary at archive root) stated
- [ ] Quick Start still fits on a screen
- [ ] No README section other than Quick Start modified
- [ ] No build targets added
- [ ] Test count unchanged; fmt and clippy clean

## 12. Required review-request format

Standard eleven parts. Particular care on:

3. **The Quick Start section in full**
4. **The download-extract-run transcript** from §9
5. Any discrepancy between the workflow matrix and what you wrote

## 13. Escalate rather than decide

Stop and raise it if: the workflow matrix disagrees with §5; the latest release
is missing an asset you were told to document; or covering the gaps honestly
makes the section too long to stay a *quick* start.
