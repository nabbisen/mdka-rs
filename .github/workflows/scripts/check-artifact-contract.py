#!/usr/bin/env python3
"""RFC 042 — assert what is inside a built artifact, before it is published.

Every other control in this project asks "does it work here?". This asks "what
is it?": it opens the built bytes and checks them against the contract in
artifact-contract.toml (which is the promise; this file only enforces it). A
binary that runs on the machine testing it can still need a glibc that half the
Linux world does not have; only reading the file shows that.

    check-artifact-contract.py cli  <target-triple> <file>     the CLI binary
    check-artifact-contract.py napi <target-triple> <file>     one .node addon
    check-artifact-contract.py napi-dir <dir>                  node/npm/*/*.node

Every violation of every artifact is reported, not the first: a release that
breaks three targets says so in one run. Exit status 1 on any violation.

Standard library only, so it runs on the ubuntu, macOS and Windows runners the
release workflows use. ELF symbol data needs `readelf` (binutils, present on the
Linux runners); if it is missing the check FAILS rather than passing blind.

What is NOT checked is printed for every artifact, because an honest partial
check beats a fake complete one. In short: for Mach-O and PE the format and
architecture are asserted (and the addon's exports, for PE); minimum OS
versions, imported system libraries and code signing are not.
"""

import argparse
import os
import re
import shutil
import struct
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    sys.exit("check-artifact-contract.py needs Python 3.11+ (tomllib)")

HERE = Path(__file__).resolve().parent
DEFAULT_CONTRACT = HERE / "artifact-contract.toml"

ELF_MACHINE = {62: "x86_64", 183: "aarch64"}
MACHO_CPU = {0x0100000C: "aarch64", 0x01000007: "x86_64"}
PE_MACHINE = {0x8664: "x86_64", 0xAA64: "aarch64"}

GLIBC_TAG = re.compile(r"@{1,2}(GLIBC_[A-Za-z0-9._]+)")


def version_tuple(text):
    return tuple(int(part) for part in text.split("."))


# ── ELF ──────────────────────────────────────────────────────────────────────
def readelf(args, path):
    tool = shutil.which("readelf")
    if tool is None:
        raise RuntimeError("readelf not found (install binutils); refusing to pass without reading the symbols")
    done = subprocess.run([tool, *args, str(path)], capture_output=True, text=True)
    return done.stdout


def elf_facts(path, header):
    """machine, glibc versions, DT_NEEDED, defined global dynamic symbols."""
    facts = {"machine": ELF_MACHINE.get(struct.unpack("<H", header[18:20])[0], "unknown")}
    glibc, defined = set(), set()
    for line in readelf(["--dyn-syms", "-W"], path).splitlines():
        cols = line.split()
        if len(cols) < 8 or not cols[0].endswith(":"):
            continue  # header lines
        bind, ndx, name = cols[4], cols[6], cols[7]
        tag = GLIBC_TAG.search(name)
        if tag:
            glibc.add(tag.group(1))
        # Section symbols and other LOCAL entries are not exports.
        if ndx != "UND" and bind in ("GLOBAL", "WEAK"):
            defined.add(name.split("@")[0])
    facts["glibc"] = glibc
    facts["defined"] = defined
    needed = []
    for line in readelf(["-d"], path).splitlines():
        m = re.search(r"\(NEEDED\)\s+Shared library: \[(.+)\]", line)
        if m:
            needed.append(m.group(1))
    facts["needed"] = needed
    return facts


def check_elf(path, header, spec, kind, contract):
    problems, unchecked = [], []
    if header[4] != 2 or header[5] != 1:
        problems.append("not a 64-bit little-endian ELF")
        return problems, unchecked
    facts = elf_facts(path, header)
    if facts["machine"] != spec["arch"]:
        problems.append(f"built for {facts['machine']}, contract says {spec['arch']}")

    def order(tag):
        return [int(p) if p.isdigit() else -1 for p in re.split(r"[._]", tag[len("GLIBC_"):])]

    versions = sorted(facts["glibc"], key=order)
    if "max_glibc" in spec:
        limit = version_tuple(spec["max_glibc"])
        too_new, unparseable = [], []
        for tag in versions:
            try:
                if version_tuple(tag[len("GLIBC_"):]) > limit:
                    too_new.append(tag)
            except ValueError:
                unparseable.append(tag)
        if too_new:
            problems.append(
                f"needs {too_new[-1]} (all above the limit: {', '.join(too_new)}); "
                f"contract says max GLIBC_{spec['max_glibc']}"
            )
        for tag in unparseable:
            problems.append(f"{tag} is not a released glibc version (contract: max GLIBC_{spec['max_glibc']})")
    elif spec.get("glibc") == "none" and versions:
        problems.append(
            f"references {', '.join(versions)}; contract says a musl target has no GLIBC_ symbol versions at all"
        )

    allowed = set(spec.get("needed_allowlist", []))
    for lib in facts["needed"]:
        if lib not in allowed:
            problems.append(f"links {lib}, not in this target's needed_allowlist")

    if kind == "napi":
        expected = set(contract["napi"]["exports"])
        if facts["defined"] != expected:
            extra = sorted(facts["defined"] - expected)
            missing = sorted(expected - facts["defined"])
            detail = []
            if extra:
                detail.append(f"unexpected: {', '.join(extra)}")
            if missing:
                detail.append(f"missing: {', '.join(missing)}")
            problems.append(f"dynamic exports are not exactly {sorted(expected)} ({'; '.join(detail)})")
    else:
        unchecked.append("exports (a CLI executable is not an addon)")
    unchecked.append("kernel/ABI requirements other than glibc symbol versions")
    return problems, unchecked


# ── Mach-O ───────────────────────────────────────────────────────────────────
def check_macho(path, header, spec, kind, contract):
    problems = []
    magic = struct.unpack("<I", header[:4])[0]
    if magic != 0xFEEDFACF:
        problems.append(f"not a thin 64-bit little-endian Mach-O (magic {header[:4].hex()})")
    else:
        cpu = MACHO_CPU.get(struct.unpack("<I", header[4:8])[0], "unknown")
        if cpu != spec["arch"]:
            problems.append(f"built for {cpu}, contract says {spec['arch']}")
    return problems, ["exports", "minimum macOS version", "imported libraries", "code signature"]


# ── PE ───────────────────────────────────────────────────────────────────────
def pe_exports(data):
    """Names in the PE export directory (PE32+), or None if there is none."""
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    n_sections = struct.unpack_from("<H", data, e_lfanew + 6)[0]
    opt_size = struct.unpack_from("<H", data, e_lfanew + 20)[0]
    opt = e_lfanew + 24
    if struct.unpack_from("<H", data, opt)[0] != 0x20B:
        raise ValueError("not a PE32+ optional header")
    export_rva, export_size = struct.unpack_from("<II", data, opt + 112)
    sections = []
    table = opt + opt_size
    for i in range(n_sections):
        _, vsize, vaddr, rawsize, rawptr = struct.unpack_from("<8sIIII", data, table + 40 * i)
        sections.append((vaddr, max(vsize, rawsize), rawptr))

    def offset(rva):
        for vaddr, size, rawptr in sections:
            if vaddr <= rva < vaddr + size:
                return rva - vaddr + rawptr
        raise ValueError(f"RVA {rva:#x} is in no section")

    if export_rva == 0:
        return None
    ed = offset(export_rva)
    n_names = struct.unpack_from("<I", data, ed + 24)[0]
    names_rva = struct.unpack_from("<I", data, ed + 32)[0]
    names = []
    for i in range(n_names):
        name_rva = struct.unpack_from("<I", data, offset(names_rva) + 4 * i)[0]
        start = offset(name_rva)
        names.append(data[start:data.index(b"\0", start)].decode("ascii"))
    return names


def check_pe(path, header, spec, kind, contract):
    problems = []
    data = Path(path).read_bytes()
    if data[:2] != b"MZ":
        return ["not a PE file (no MZ header)"], []
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    if data[e_lfanew:e_lfanew + 4] != b"PE\0\0":
        return ["not a PE file (no PE signature)"], []
    machine = PE_MACHINE.get(struct.unpack_from("<H", data, e_lfanew + 4)[0], "unknown")
    if machine != spec["arch"]:
        problems.append(f"built for {machine}, contract says {spec['arch']}")
    unchecked = ["minimum Windows version", "imported DLLs / CRT requirements"]
    if kind == "napi":
        try:
            names = set(pe_exports(data) or [])
        except (ValueError, struct.error) as err:
            problems.append(f"cannot read the export table: {err}")
        else:
            expected = set(contract["napi"]["exports"])
            if names != expected:
                problems.append(f"exports are {sorted(names)}, contract says exactly {sorted(expected)}")
    else:
        unchecked.append("exports (a CLI executable is not an addon)")
    return problems, unchecked


CHECKERS = {"elf": check_elf, "macho": check_macho, "pe": check_pe}
MAGIC = {"elf": b"\x7fELF", "pe": b"MZ"}


def check_artifact(path, target, kind, contract):
    """(problems, unchecked) for one artifact."""
    spec = contract["targets"].get(target)
    if spec is None:
        return [f"target {target!r} is not in the contract (known: {', '.join(sorted(contract['targets']))})"], []
    path = Path(path)
    if not path.is_file():
        return [f"{path} does not exist"], []
    with open(path, "rb") as fh:
        header = fh.read(64)
    fmt = spec["format"]
    if fmt in MAGIC and not header.startswith(MAGIC[fmt]):
        return [f"contract says {fmt}, but the file begins {header[:4].hex()}"], []
    try:
        return CHECKERS[fmt](path, header, spec, kind, contract)
    except RuntimeError as err:
        return [str(err)], []


def report(results):
    """Print a report; return the number of violations."""
    total = 0
    in_ci = os.environ.get("GITHUB_ACTIONS") == "true"
    for label, target, path, problems, unchecked in results:
        print(f"{'FAIL' if problems else 'PASS'}  {label}  ({target})  {path}")
        for problem in problems:
            print(f"      violation: {problem}")
            if in_ci:
                print(f"::error title=Artifact contract violated ({label})::{path}: {problem}")
        if unchecked:
            print(f"      not checked: {'; '.join(unchecked)}")
        total += len(problems)
    failed = sum(1 for r in results if r[3])
    print(f"\n{len(results)} artifact(s) checked, {failed} failing, {total} violation(s).")
    return total


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--contract", default=DEFAULT_CONTRACT, type=Path)
    sub = parser.add_subparsers(dest="mode", required=True)
    for name in ("cli", "napi"):
        p = sub.add_parser(name)
        p.add_argument("target")
        p.add_argument("file")
    p = sub.add_parser("napi-dir")
    p.add_argument("dir", type=Path)
    args = parser.parse_args()

    with open(args.contract, "rb") as fh:
        contract = tomllib.load(fh)

    results = []
    if args.mode in ("cli", "napi"):
        problems, unchecked = check_artifact(args.file, args.target, args.mode, contract)
        results.append((args.mode, args.target, args.file, problems, unchecked))
    else:
        by_platform = {spec["npm_platform"]: target for target, spec in contract["targets"].items()}
        present = {p.name for p in args.dir.iterdir() if p.is_dir()} if args.dir.is_dir() else set()
        for platform in sorted(by_platform):
            target = by_platform[platform]
            files = sorted((args.dir / platform).glob("*.node")) if platform in present else []
            if len(files) != 1:
                why = "no package directory" if platform not in present else f"{len(files)} .node files (expected exactly one)"
                results.append((f"npm {platform}", target, args.dir / platform, [why], []))
                continue
            problems, unchecked = check_artifact(files[0], target, "napi", contract)
            results.append((f"npm {platform}", target, files[0], problems, unchecked))
        for stray in sorted(present - set(by_platform)):
            results.append((f"npm {stray}", "?", args.dir / stray, ["a platform package the contract does not list"], []))

    return 1 if report(results) else 0


if __name__ == "__main__":
    sys.exit(main())
