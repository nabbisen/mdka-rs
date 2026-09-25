#!/usr/bin/env python3
"""RFC 046 -- check the release archives the way a user meets them.

Every distribution channel has a gate that fetches what the registry actually
serves, except GitHub Releases. This is that gate's checker: given the archives
*downloaded from a published release*, it does what README.md tells a reader to
do, and asserts that it works.

Per asset:

  1. the archive was downloaded (the workflow does that with `gh release
     download`; a missing asset fails here);
  2. **follow the README's own instructions**: extract, `cd` into the folder the
     archive created (named after the asset -- derived, not hardcoded), and find
     `./mdka` there. What is asserted is that the *documented steps work*, not any
     particular layout: if the layout changes and the README follows, this still
     passes; if the README stops being true, it fails;
  3. run the binary on the README's own example and compare its output with the
     output the README shows -- only where this machine can execute it, and the
     log says which assets were and were not;
  4. apply RFC 042's contract to the extracted binary, by running
     check-artifact-contract.py unchanged. There is one contract, not two.

Nothing here is written to be a second source of truth: the assets expected come
from the release workflow's own matrix, the instructions and expected output
from README.md, and the contract from artifact-contract.toml.

    check-release-archives.py --assets-dir DIR --tag TAG [--only NAME]...

Exit status 1 on any failure. Standard library only.
"""

import argparse
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]


# ── what the release is supposed to contain ──────────────────────────────────
def expected_assets(matrix_file):
    """[(name, target, ext)] from release-executable.yaml's build matrix."""
    text = Path(matrix_file).read_text(encoding="utf-8")
    # Only the matrix: the steps below it have `name:` lines of their own.
    start = text.index("include:")
    end = text.find("\n    steps:", start)
    text = text[start:end if end != -1 else len(text)]
    entries, current = [], {}
    for line in text.splitlines():
        m = re.match(r"\s*-?\s*(name|target|archive_ext):\s*(\S*)\s*$", line)
        if not m:
            continue
        key, value = m.groups()
        if key == "name" and current.get("name"):
            entries.append(current)
            current = {}
        current[key] = value
        if all(k in current for k in ("name", "target", "archive_ext")) and current["archive_ext"]:
            entries.append(current)
            current = {}
    seen, out = set(), []
    for e in entries:
        if e["name"] not in seen and e.get("target") and e.get("archive_ext"):
            seen.add(e["name"])
            out.append((e["name"], e["target"], e["archive_ext"]))
    if not out:
        raise SystemExit(f"no build matrix entries found in {matrix_file}")
    return out


# ── what the README tells a reader to do ─────────────────────────────────────
def readme_example(readme):
    """(html, expected_output) from the README's extract-and-run block.

    The block is the one that `cd`s into the extracted folder and pipes a
    snippet into `./mdka`; the lines after it, each prefixed `#`, are the output
    the README shows."""
    lines = Path(readme).read_text(encoding="utf-8").splitlines()
    for i, line in enumerate(lines):
        m = re.match(r"echo '(.*)' \| \./mdka\s*$", line)
        if not m:
            continue
        expected = []
        for out in lines[i + 1:]:
            if not out.startswith("#"):
                break
            expected.append(re.sub(r"^# ?", "", out))
        while expected and expected[-1] == "":
            expected.pop()
        if not expected:
            raise SystemExit(f"{readme}: the ./mdka example shows no output lines")
        return m.group(1), "\n".join(expected) + "\n"
    raise SystemExit(f"{readme}: no `echo '...' | ./mdka` example found")


# ── can this machine run the binary? ─────────────────────────────────────────
def host_can_run(target):
    machine = platform.machine().lower()
    x64 = machine in ("x86_64", "amd64")
    arm64 = machine in ("aarch64", "arm64")
    if target.endswith("linux-gnu") or target.endswith("linux-musl"):
        if not sys.platform.startswith("linux"):
            return False, f"Linux binary on a {sys.platform} host"
        want = "x86_64" if target.startswith("x86_64") else "aarch64"
        have = "x86_64" if x64 else "aarch64" if arm64 else machine
        if want != have:
            return False, f"{want} binary on a {have} host (no emulator installed)"
        return True, ""
    if target.endswith("apple-darwin"):
        ok = sys.platform == "darwin" and arm64
        return ok, "" if ok else f"macOS arm64 binary on a {sys.platform}/{machine} host"
    if target.endswith("windows-msvc"):
        ok = sys.platform == "win32" and x64
        return ok, "" if ok else f"Windows binary on a {sys.platform}/{machine} host"
    return False, f"unknown target {target}"


def extract(archive, into):
    if archive.name.endswith(".tar.gz"):
        with tarfile.open(archive) as tar:
            tar.extractall(into, filter="data")
    elif archive.name.endswith(".zip"):
        # `unzip` keeps the executable bit recorded in the archive. If it is not
        # installed (or fails), extract with the standard library and restore
        # the recorded mode ourselves -- a missing tool must not be reported as
        # a broken archive.
        unzip = shutil.which("unzip")
        if unzip and subprocess.run([unzip, "-q", str(archive), "-d", str(into)]).returncode == 0:
            return
        with zipfile.ZipFile(archive) as z:
            for info in z.infolist():
                target = z.extract(info, into)
                mode = info.external_attr >> 16
                if mode:
                    os.chmod(target, mode & 0o7777)
    else:
        raise RuntimeError(f"unknown archive type: {archive.name}")


def check_asset(name, target, ext, tag, assets_dir, html, want_output, contract_args):
    """Returns (lines, failures, coverage) for one asset."""
    lines, failures = [], []
    asset = assets_dir / f"mdka@{name}-{tag}{ext}"
    lines.append(f"== {asset.name}  ({target})")

    # 1. downloaded
    if not asset.is_file():
        failures.append(f"step 1: {asset.name} is not among the downloaded release assets")
        return lines, failures, "missing"
    lines.append(f"   1 download   ok    {asset.stat().st_size} bytes")

    # 2. the README's own instructions: extract, cd into the folder, ./mdka
    folder_name = asset.name[: -len(ext)]  # the folder the archive extracted to
    work = Path(tempfile.mkdtemp(prefix="mdka-archive-"))
    try:
        extract(asset, work)
    except Exception as err:  # noqa: BLE001 -- any failure to extract is the finding
        failures.append(f"step 2: the archive does not extract: {err}")
        return lines, failures, "download only"
    folder = work / folder_name
    if not folder.is_dir():
        found = sorted(p.name for p in work.iterdir())
        failures.append(
            f"step 2: README says to `cd {folder_name}` after extracting, but the archive "
            f"extracted to {found or 'nothing'}; the documented steps do not work"
        )
        lines.append("   2 README     FAIL  cd into the extracted folder")
        return lines, failures, "download only"
    binary = folder / ("mdka.exe" if target.endswith("windows-msvc") else "mdka")
    if not binary.is_file():
        found = sorted(p.name for p in folder.iterdir())
        failures.append(
            f"step 2: README says to run ./mdka in {folder_name}/, which holds {found or 'nothing'}"
        )
        lines.append("   2 README     FAIL  ./mdka is not there")
        return lines, failures, "download only"
    if not target.endswith("windows-msvc") and not os.access(binary, os.X_OK):
        failures.append(f"step 2: ./mdka in {folder_name}/ is not executable, so `./mdka` fails")
        lines.append("   2 README     FAIL  ./mdka is not executable")
        return lines, failures, "download only"
    lines.append(f"   2 README     ok    extracted to {folder_name}/, ./{binary.name} present and executable")

    # 3. run it on the README's own example
    runnable, why = host_can_run(target)
    ran = False
    if runnable:
        try:
            done = subprocess.run(
                [str(binary)], input=html, capture_output=True, text=True, timeout=60, cwd=folder
            )
            ran = True
            if done.returncode != 0:
                failures.append(
                    f"step 3: ./mdka exited {done.returncode} on the README's example: "
                    f"{(done.stderr or done.stdout).strip()[:300]!r}"
                )
                lines.append(f"   3 run        FAIL  exit {done.returncode}")
            elif done.stdout != want_output:
                failures.append(
                    f"step 3: the README shows {want_output!r} for `echo '{html}' | ./mdka`; "
                    f"the binary wrote {done.stdout!r}"
                )
                lines.append("   3 run        FAIL  output differs from the README's")
            else:
                lines.append(f"   3 run        ok    EXECUTED, output matches the README: {want_output!r}")
        except (OSError, subprocess.TimeoutExpired) as err:
            failures.append(f"step 3: ./mdka could not be run: {err}")
            lines.append("   3 run        FAIL  could not execute")
    else:
        lines.append(f"   3 run        SKIP  NOT EXECUTED: {why}")

    # 4. RFC 042's contract, unchanged
    done = subprocess.run(
        [sys.executable, str(HERE / "check-artifact-contract.py"), *contract_args, "cli", target, str(binary)],
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        failures.append(f"step 4: the downloaded binary violates the artifact contract:\n"
                        + "\n".join("      " + l for l in done.stdout.strip().splitlines()))
        lines.append("   4 contract   FAIL  see below")
    else:
        lines.append("   4 contract   ok    artifact-contract.toml satisfied")
    coverage = "steps 1-4 (executed)" if ran else "steps 1, 2 and 4 (not executed here)"
    return lines, failures, coverage


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--assets-dir", required=True, type=Path)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--only", action="append", default=[], help="check just these asset names (for demonstrations)")
    ap.add_argument("--readme", default=REPO / "README.md", type=Path)
    ap.add_argument("--matrix", default=REPO / ".github/workflows/release-executable.yaml", type=Path)
    ap.add_argument("--contract", type=Path, default=None, help="passed through to check-artifact-contract.py")
    args = ap.parse_args()

    html, want = readme_example(args.readme)
    assets = expected_assets(args.matrix)
    # Every archive the matrix declares, whatever --only selects: an asset on
    # the release that is not in this set is verified by nothing.
    declared = {f"mdka@{n}-{args.tag}{e}" for n, _, e in assets}
    if args.only:
        assets = [a for a in assets if a[0] in args.only]
    contract_args = ["--contract", str(args.contract)] if args.contract else []

    print(f"release {args.tag}: {len(assets)} asset(s) expected by {args.matrix.name}")
    print(f"README example: echo '{html}' | ./mdka   ->   {want!r}")
    print(f"this host: {sys.platform}/{platform.machine()}\n")

    all_failures, coverage = [], []
    for name, target, ext in assets:
        lines, failures, cov = check_asset(name, target, ext, args.tag, args.assets_dir, html, want, contract_args)
        print("\n".join(lines))
        for f in failures:
            print(f"   FAILURE: {f}")
            if os.environ.get("GITHUB_ACTIONS") == "true":
                print(f"::error title=Release archive check ({name})::{f.splitlines()[0]}")
        print()
        all_failures += [(name, f) for f in failures]
        coverage.append((name, cov))

    # The other direction: a published archive the matrix does not declare (a
    # target removed from the build while its archive stays on the release page)
    # would otherwise be downloadable and checked by nothing.
    for extra in sorted(p.name for p in args.assets_dir.glob("mdka@*") if p.name not in declared):
        msg = (f"{extra} is on the release but the build matrix in {args.matrix.name} does not declare it, "
               f"so nothing verifies it")
        print(f"FAILURE: unexpected asset: {msg}")
        if os.environ.get("GITHUB_ACTIONS") == "true":
            print(f"::error title=Release archive check (unexpected asset)::{msg}")
        all_failures.append((extra, msg))

    print("coverage:")
    for name, cov in coverage:
        print(f"  {name:22s} {cov}")
    executed = sum(1 for _, c in coverage if c.startswith("steps 1-4"))
    print(f"\n{len(assets)} asset(s) checked; the binary was executed for {executed} of them; "
          f"{len(all_failures)} failure(s).")
    return 1 if all_failures else 0


if __name__ == "__main__":
    sys.exit(main())
