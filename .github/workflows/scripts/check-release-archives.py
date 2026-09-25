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
class Block:
    """One platform's extract-and-run instructions, as the README writes them."""

    def __init__(self, name, lang, setup, run, expected):
        self.name = name          # the asset name the README uses, e.g. Linux-x64-gnu
        self.lang = lang          # the fence's language: bash or powershell
        self.setup = setup        # commands up to and including the `cd`
        self.run = run            # the commands after it
        self.expected = expected  # the output the README shows


def readme_blocks(readme):
    """{OS prefix: Block} for every fenced block that does `cd mdka@<Name>-<version>`.

    A block is recognised by that `cd`, not by its position, so a platform
    added to the README is read and one that is malformed fails loudly instead
    of being silently ignored. The commands run from the start of the block
    to the end of the last command; the `#` lines after them, prefixed, are the
    output the README shows. `<version>` is the release being checked."""
    lines = Path(readme).read_text(encoding="utf-8").splitlines()
    blocks, i = {}, 0
    while i < len(lines):
        m = re.match(r"```(\w+)\s*$", lines[i])
        if not m:
            i += 1
            continue
        lang, body, i = m.group(1), [], i + 1
        while i < len(lines) and not lines[i].startswith("```"):
            body.append(lines[i])
            i += 1
        i += 1
        cd = next((n for n, l in enumerate(body) if re.match(r"cd\s+mdka@\S+-<version>\s*$", l)), None)
        if cd is None:
            continue
        name = re.match(r"cd\s+mdka@(\S+)-<version>", body[cd]).group(1)
        commands, expected = [], []
        for l in body:
            (expected if (commands and l.startswith("#")) else commands).append(l)
        # a block's expected output starts at its first `#` line after a command
        expected = [re.sub(r"^# ?", "", l) for l in expected if l.startswith("#")]
        while expected and expected[-1] == "":
            expected.pop()
        if not expected:
            raise SystemExit(f"{readme}: the {name} block shows no output lines")
        os_prefix = name.split("-")[0]
        if os_prefix in blocks:
            raise SystemExit(f"{readme}: two blocks for {os_prefix}; the gate cannot tell which is meant")
        blocks[os_prefix] = Block(name, lang, commands[: cd + 1], commands[cd + 1:], "\n".join(expected) + "\n")
    if not blocks:
        raise SystemExit(f"{readme}: no `cd mdka@<Name>-<version>` block found")
    return blocks


def run_block(block, asset, asset_name, tag, timeout=120):
    """Run the README's commands verbatim, in a directory holding only the
    downloaded archive, the way a reader would. Returns (entered, output, error)."""
    sub = lambda l: l.replace(block.name, asset_name).replace("<version>", tag)  # noqa: E731
    setup = "\n".join(sub(l) for l in block.setup)
    run = "\n".join(sub(l) for l in block.run)
    work = Path(tempfile.mkdtemp(prefix="mdka-readme-"))
    shutil.copy2(asset, work / asset.name)
    marker = "__MDKA_ENTERED__"
    if block.lang == "powershell":
        exe = shutil.which("powershell") or shutil.which("pwsh")
        if not exe:
            return False, "", "no PowerShell on this host"
        script = (f"$ErrorActionPreference = 'Stop'\n{setup}\nWrite-Output '{marker}'\n{run}\n"
                  "if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }")
        cmd = [exe, "-NoProfile", "-NonInteractive", "-Command", script]
    else:
        cmd = ["bash", "-c", f"set -e\n{setup}\necho {marker}\n{run}"]
    done = subprocess.run(cmd, cwd=work, capture_output=True, text=True, timeout=timeout)
    out = done.stdout.replace("\r\n", "\n")
    entered = marker in out
    output = out.split(marker, 1)[1].lstrip("\n") if entered else ""
    error = (done.stderr or "").strip() if done.returncode != 0 else ""
    if done.returncode != 0 and not error:
        error = f"exit {done.returncode}"
    return entered, output.rstrip("\n") + "\n" if output.strip() else "", error


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
            # Registered user-mode emulation (qemu-user-static's binfmt_misc
            # entry) lets ./mdka run as a reader would run it, just slower.
            entry = Path(f"/proc/sys/fs/binfmt_misc/qemu-{want}")
            if have == "x86_64" and want == "aarch64" and entry.is_file() and "enabled" in entry.read_text():
                return True, "under QEMU user-mode emulation"
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


def check_asset(name, target, ext, tag, assets_dir, blocks, contract_args):
    """Returns (lines, failures, coverage) for one asset."""
    lines, failures = [], []
    asset = assets_dir / f"mdka@{name}-{tag}{ext}"
    lines.append(f"== {asset.name}  ({target})")

    # 1. downloaded
    if not asset.is_file():
        failures.append(f"step 1: {asset.name} is not among the downloaded release assets")
        return lines, failures, "missing"
    lines.append(f"   1 download   ok    {asset.stat().st_size} bytes")

    # 2. the archive holds what the README says it does: the folder it tells a
    #    reader to `cd` into, with an executable ./mdka in it. (Step 3 then
    #    follows the README's own commands; this names *what* is wrong, on every
    #    host, including ones that cannot run the binary.)
    folder_name = asset.name[: -len(ext)]  # the folder the archive extracted to
    block = blocks.get(name.split("-")[0])
    if block is None:
        failures.append(
            f"step 2: README.md has no `cd mdka@{name.split('-')[0]}-...-<version>` instructions for "
            f"{name}; a reader of the prebuilt binary is given nothing to follow"
        )
        lines.append("   2 README     FAIL  no instructions for this platform")
        return lines, failures, "download only"
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
            f"step 2: README says to run mdka in {folder_name}/, which holds {found or 'nothing'}"
        )
        lines.append("   2 README     FAIL  the binary is not there")
        return lines, failures, "download only"
    if not target.endswith("windows-msvc") and not os.access(binary, os.X_OK):
        failures.append(f"step 2: ./mdka in {folder_name}/ is not executable, so `./mdka` fails")
        lines.append("   2 README     FAIL  ./mdka is not executable")
        return lines, failures, "download only"
    lines.append(f"   2 README     ok    extracted to {folder_name}/, {binary.name} present"
                 + ("" if target.endswith("windows-msvc") else " and executable"))

    # 3. do what the README says, verbatim, and compare with the output it shows
    runnable, note = host_can_run(target)
    ran = False
    if runnable:
        try:
            entered, output, error = run_block(block, asset, name, tag)
            ran = True
            how = f"the README's {block.lang} block"
            if not entered:
                failures.append(
                    f"step 3: {how} failed before the binary was run "
                    f"(extract and `cd {folder_name}`): {error[:400]!r}"
                )
                lines.append("   3 run        FAIL  the README's extract/cd commands failed")
            elif error:
                failures.append(f"step 3: {how} failed: {error[:400]!r}")
                lines.append("   3 run        FAIL  the README's commands failed")
            elif output != block.expected:
                failures.append(
                    f"step 3: {how} shows {block.expected!r}; running it wrote {output!r}"
                )
                lines.append("   3 run        FAIL  output differs from the README's")
            else:
                lines.append(f"   3 run        ok    EXECUTED{(' ' + note) if note else ''}, "
                             f"the README's {block.lang} commands ran; output matches: {block.expected!r}")
        except (OSError, subprocess.TimeoutExpired) as err:
            failures.append(f"step 3: the README's commands could not be run: {err}")
            lines.append("   3 run        FAIL  could not execute")
    else:
        lines.append(f"   3 run        SKIP  NOT EXECUTED: {note}")

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
    if ran:
        coverage = "steps 1-4 (executed" + (f", {note}" if note else "") + ")"
    else:
        coverage = "steps 1, 2 and 4 (not executed here)"
    return lines, failures, coverage


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--assets-dir", required=True, type=Path)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--only", action="append", default=[], help="check just these asset names")
    ap.add_argument("--require-executed", action="append", default=[], metavar="NAME",
                    help="fail unless this asset's binary was executed on this host (step 3 ran)")
    ap.add_argument("--readme", default=REPO / "README.md", type=Path)
    ap.add_argument("--matrix", default=REPO / ".github/workflows/release-executable.yaml", type=Path)
    ap.add_argument("--contract", type=Path, default=None, help="passed through to check-artifact-contract.py")
    args = ap.parse_args()

    blocks = readme_blocks(args.readme)
    assets = expected_assets(args.matrix)
    # Every archive the matrix declares, whatever --only selects: an asset on
    # the release that is not in this set is verified by nothing.
    declared = {f"mdka@{n}-{args.tag}{e}" for n, _, e in assets}
    if args.only:
        assets = [a for a in assets if a[0] in args.only]
    contract_args = ["--contract", str(args.contract)] if args.contract else []

    print(f"release {args.tag}: {len(assets)} asset(s) expected by {args.matrix.name}")
    for os_prefix, b in sorted(blocks.items()):
        print(f"README {os_prefix}: {b.lang} block, shows {b.expected!r}")
    print(f"this host: {sys.platform}/{platform.machine()}\n")

    all_failures, coverage = [], []
    for name, target, ext in assets:
        lines, failures, cov = check_asset(name, target, ext, args.tag, args.assets_dir, blocks, contract_args)
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

    # An asset a job is responsible for executing must have been executed: a leg
    # that cannot run fails, with the reason, rather than being reported and passed.
    by_name = dict(coverage)
    for req in args.require_executed:
        cov = by_name.get(req)
        if cov is None:
            all_failures.append((req, "not checked"))
            print(f"FAILURE: {req} was required to be executed here but was not among the assets checked")
        elif not cov.startswith("steps 1-4"):
            msg = f"{req} was required to be executed on this host but was not ({cov})"
            all_failures.append((req, msg))
            print(f"FAILURE: {msg}")
            if os.environ.get("GITHUB_ACTIONS") == "true":
                print(f"::error title=Release archive check (coverage)::{msg}")

    print("coverage:")
    for name, cov in coverage:
        print(f"  {name:22s} {cov}")
    executed = sum(1 for _, c in coverage if c.startswith("steps 1-4"))
    print(f"\n{len(assets)} asset(s) checked; the binary was executed for {executed} of them; "
          f"{len(all_failures)} failure(s).")
    return 1 if all_failures else 0


if __name__ == "__main__":
    sys.exit(main())
