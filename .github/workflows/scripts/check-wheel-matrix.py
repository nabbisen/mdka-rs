#!/usr/bin/env python3
"""Check PyPI distributables against python/wheel-matrix.toml (RFC 034).

ONE script for both sides of the irreversible upload, so they cannot disagree:

  release-pypi.yaml       `files`     -- dist/ before `uv publish`
  pypi-published-gate     `published` -- the file list PyPI actually serves
  ci.yaml, published gate `floor-*`   -- the floor written elsewhere equals
                                         the declared python-floor

The file-set check fails on every MISSING file and every UNEXPECTED file, and
lists each. An extra wheel is a support promise nobody declared; a missing one
is a broken promise. A same-named file from two build jobs overwriting each
other shows up here as a missing file.

Filenames are parsed with `packaging.utils` (normalised project names, PEP 440
versions, compound platform tags). A wheel's tags are a SET: a manylinux wheel
is named `...-manylinux_2_17_x86_64.manylinux2014_x86_64.whl`, and a declared
platform matches when its tag is a member of that set -- never a substring, so
`manylinux_2_17_x86_64` cannot match an aarch64 wheel.

Requires Python >= 3.11 (tomllib) and `packaging`.
"""

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

from packaging.tags import Tag
from packaging.utils import (
    InvalidSdistFilename,
    InvalidWheelFilename,
    canonicalize_name,
    parse_sdist_filename,
    parse_wheel_filename,
)
from packaging.version import Version

PROJECT = "mdka"
ROOT = Path(__file__).resolve().parents[3]
MATRIX = ROOT / "python" / "wheel-matrix.toml"


def load_matrix(path):
    with open(path, "rb") as f:
        m = tomllib.load(f)
    if not m.get("abi3"):
        # Without abi3 every Python version needs its own wheel, and the
        # declaration would have to list each one. Not modelled; refuse
        # rather than check something other than what is declared.
        sys.exit(f"{path}: abi3 = false is not supported by this check")
    return m


def expected_tags(m):
    """Declared wheel tags, each as a packaging Tag."""
    major, minor = m["python-floor"].split(".")
    interp = f"cp{major}{minor}"
    tags = [Tag(interp, "abi3", p) for p in m["platforms"]]
    for extra in m.get("extra", []):
        i, a, p = extra.split("-")
        tags.append(Tag(i, a, p))
    return tags


def check_files(m, version, filenames):
    """Return a list of problems; empty means the set is exactly as declared."""
    version = Version(version)
    problems = []
    wanted = expected_tags(m)
    matched = {t: [] for t in wanted}
    sdists = []

    for raw in filenames:
        name = Path(raw).name
        if name.endswith(".whl"):
            try:
                proj, ver, _build, tags = parse_wheel_filename(name)
            except InvalidWheelFilename as e:
                problems.append(f"unexpected: {name} (unparseable wheel name: {e})")
                continue
            if proj != canonicalize_name(PROJECT) or ver != version:
                problems.append(f"unexpected: {name} (project/version is {proj} {ver}, expected {PROJECT} {version})")
                continue
            hits = [t for t in wanted if t in tags]
            if not hits:
                problems.append(f"unexpected: {name} (tags {sorted(map(str, tags))} match no declared cell)")
            for t in hits:
                matched[t].append(name)
        elif name.endswith(".tar.gz"):
            try:
                proj, ver = parse_sdist_filename(name)
            except InvalidSdistFilename as e:
                problems.append(f"unexpected: {name} (unparseable sdist name: {e})")
                continue
            if proj != canonicalize_name(PROJECT) or ver != version:
                problems.append(f"unexpected: {name} (project/version is {proj} {ver}, expected {PROJECT} {version})")
                continue
            sdists.append(name)
        else:
            problems.append(f"unexpected: {name} (neither a wheel nor an sdist)")

    for t, names in matched.items():
        if not names:
            problems.append(f"missing: wheel for {t} (mdka-{version}-{t}.whl)")
        elif len(names) > 1:
            problems.append(f"unexpected: {len(names)} wheels for {t}: {names}")

    if m.get("sdist"):
        if not sdists:
            problems.append(f"missing: sdist (mdka-{version}.tar.gz)")
        elif len(sdists) > 1:
            problems.append(f"unexpected: {len(sdists)} sdists: {sdists}")
    elif sdists:
        problems.append(f"unexpected: sdist {sdists} (sdist = false)")

    return problems


def report(problems, what):
    for p in problems:
        print(f"  {p}")
    if problems:
        print(f"::error::{len(problems)} problem(s) in {what} against python/wheel-matrix.toml")
        return 1
    print(f"OK: {what} is exactly the declared matrix")
    return 0


def cmd_files(args, m):
    pyproject = tomllib.loads((ROOT / "python" / "pyproject.toml").read_text())
    source = pyproject["project"]["version"]
    if args.tag is not None:
        # On a release the tag names the version; it must be what was built.
        if args.tag != source:
            print(f"::error::tag {args.tag!r} != python/pyproject.toml version {source!r}")
            return 1
    # On a dry run (workflow_dispatch) there is no tag: the source decides.
    names = list(args.files)
    if args.dir:
        names += sorted(p.name for p in Path(args.dir).iterdir() if p.is_file())
    print(f"checking {len(names)} file(s) for mdka {source}:")
    for n in names:
        print(f"  {Path(n).name}")
    return report(check_files(m, source, names), "the built distributables")


def cmd_published(args, m):
    data = json.loads(Path(args.json).read_text())
    version = data["info"]["version"]
    names = [u["filename"] for u in data["urls"]]
    print(f"PyPI latest: mdka {version}, {len(names)} file(s)")
    if Version(version) < Version(m["applies-from"]):
        print(
            f"::notice::mdka {version} predates the declared wheel matrix "
            f"(applies-from {m['applies-from']}); the file-set check is NOT applied to it."
        )
        return 0
    return report(check_files(m, version, names), f"the files published for {version}")


DOC_FLOOR = re.compile(r"^Requires CPython (\d+\.\d+) or later\.$", re.M)


def cmd_floor_doc(args, m):
    text = Path(args.path).read_text()
    found = DOC_FLOOR.findall(text)
    if len(found) != 1:
        print(
            f"::error::{args.path}: expected exactly one line 'Requires CPython X.Y or later.', "
            f"found {len(found)}. The floor check cannot pass on a page that does not state it."
        )
        return 1
    if found[0] != m["python-floor"]:
        print(f"::error::{args.path} states floor {found[0]}, wheel-matrix.toml says {m['python-floor']}")
        return 1
    print(f"OK: {args.path} states floor {found[0]}")
    return 0


def cmd_floor_equals(args, m):
    if args.value != m["python-floor"]:
        print(f"::error::{args.what} floor {args.value!r} != wheel-matrix.toml python-floor {m['python-floor']!r}")
        return 1
    print(f"OK: {args.what} floor {args.value} matches")
    return 0


def cmd_floor_manifests(args, m):
    floor = m["python-floor"]
    problems = []
    pyproject = tomllib.loads((ROOT / "python" / "pyproject.toml").read_text())
    rp = pyproject["project"]["requires-python"]
    if rp != f">={floor}":
        problems.append(f"python/pyproject.toml requires-python {rp!r} != '>={floor}'")
    cargo = (ROOT / "python" / "Cargo.toml").read_text()
    features = re.findall(r'"abi3-py(\d)(\d+)"', cargo)
    if [f"{a}.{b}" for a, b in features] != [floor]:
        problems.append(f"python/Cargo.toml abi3 features {features} != exactly one abi3-py{floor.replace('.', '')}")
    for p in problems:
        print(f"::error::{p}")
    if not problems:
        print(f"OK: requires-python and the PyO3 abi3 feature match floor {floor}")
    return 1 if problems else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--matrix", default=MATRIX)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("files", help="check built distributables (pre-upload)")
    p.add_argument("--dir")
    p.add_argument("--tag", help="release tag; must equal pyproject version")
    p.add_argument("files", nargs="*")
    p.set_defaults(fn=cmd_files)

    p = sub.add_parser("published", help="check PyPI's JSON for the latest release")
    p.add_argument("json")
    p.set_defaults(fn=cmd_published)

    p = sub.add_parser("floor-doc", help="installation.md states the floor")
    p.add_argument("path")
    p.set_defaults(fn=cmd_floor_doc)

    p = sub.add_parser("floor-equals", help="a floor written elsewhere equals the declared one")
    p.add_argument("what")
    p.add_argument("value")
    p.set_defaults(fn=cmd_floor_equals)

    p = sub.add_parser("floor-manifests", help="requires-python and abi3 feature match the floor")
    p.set_defaults(fn=cmd_floor_manifests)

    p = sub.add_parser("floor", help="print python-floor")
    p.set_defaults(fn=lambda a, m: print(m["python-floor"]) or 0)

    args = ap.parse_args()
    sys.exit(args.fn(args, load_matrix(args.matrix)))


if __name__ == "__main__":
    main()
