#!/usr/bin/env python3
"""Execute every runnable example in docs/src/ (RFC 026 docs-example gate).

A fenced block is runnable when its info string names a language we can run:
rust, python, js, ts. Everything else -- including an unlabelled fence, which
is how output samples are written -- is ignored.

A block that is deliberately a fragment is marked by adding `fragment` to the
info string (```rust,fragment). The marker is what excludes it: there is no
list of exceptions maintained elsewhere that can drift out of step with the
documents. Rust's own `ignore` and `no_run` are honoured too, since rustdoc
already gives them this meaning.

`D-12` and `D-13` were examples that failed on their first line while the
documentation presented them as runnable. This is the control for that.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

RUNNABLE = {"rust", "python", "js", "ts"}
SKIP_MARKERS = {"fragment", "ignore", "no_run", "text", "compile_fail"}
FENCE = re.compile(r"^(?P<indent>\s*)```(?P<info>[A-Za-z0-9,_+-]*)\s*$")


class Block:
    def __init__(self, path, line, lang, markers, code):
        self.path, self.line, self.lang = path, line, lang
        self.markers, self.code = markers, code

    @property
    def where(self):
        return f"{self.path}:{self.line}"


def extract(root):
    """Yield every fenced block under `root`, in document order."""
    blocks = []
    for md in sorted(Path(root).rglob("*.md")):
        lines = md.read_text(encoding="utf-8").splitlines()
        i = 0
        while i < len(lines):
            m = FENCE.match(lines[i])
            if not m:
                i += 1
                continue
            info = m.group("info")
            start, body = i + 1, []
            i += 1
            # closing fence: same indent, no info string
            while i < len(lines) and not re.match(
                r"^%s```\s*$" % re.escape(m.group("indent")), lines[i]
            ):
                body.append(lines[i])
                i += 1
            i += 1
            parts = [p for p in info.split(",") if p]
            if not parts:
                continue
            lang, markers = parts[0], set(parts[1:])
            blocks.append(Block(md, start, lang, markers, "\n".join(body)))
    return blocks


def runnable(blocks):
    out = []
    for b in blocks:
        if b.lang not in RUNNABLE:
            continue
        if b.markers & SKIP_MARKERS:
            continue
        out.append(b)
    return out


def check_rust(blocks, workdir):
    """Type-check each Rust example against the real crate.

    Compiled rather than executed: most examples name input files that do not
    exist here, so running them would assert on the fixture rather than on the
    example. `D-12`-class defects -- undeclared names, wrong imports, wrong
    types -- are all compile-time in Rust.
    """
    if not blocks:
        return []
    proj = workdir / "rustcheck"
    (proj / "src" / "bin").mkdir(parents=True, exist_ok=True)
    repo = Path.cwd().resolve()
    (proj / "Cargo.toml").write_text(
        "[package]\nname = 'docs-examples'\nversion = '0.0.0'\nedition = '2024'\n"
        f"[dependencies]\nmdka = {{ path = '{repo}' }}\n"
        "[workspace]\n",
        encoding="utf-8",
    )
    (proj / "src" / "lib.rs").write_text("", encoding="utf-8")
    names = {}
    for n, b in enumerate(blocks):
        code = b.code
        if "fn main" not in code:
            body = "\n".join("    " + ln for ln in code.splitlines())
            # Mirror rustdoc: an example using `?` is written as if inside a
            # fallible function, so wrap it in one rather than reporting the
            # absence of a Result-returning main as the example's defect.
            if "?" in code:
                code = (
                    "fn main() -> Result<(), Box<dyn std::error::Error>> {\n"
                    "%s\n    Ok(())\n}" % body
                )
            else:
                code = "fn main() {\n%s\n}" % body
        name = f"ex{n}"
        names[name] = b
        (proj / "src" / "bin" / f"{name}.rs").write_text(
            "#![allow(unused, deprecated)]\n" + code, encoding="utf-8"
        )
    r = subprocess.run(
        ["cargo", "build", "--message-format=json"],
        cwd=proj,
        capture_output=True,
        text=True,
    )
    if r.returncode == 0:
        return []
    failed = {}
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        if msg.get("message", {}).get("level") != "error":
            continue
        tgt = msg.get("target", {}).get("name")
        if tgt in names:
            failed.setdefault(tgt, msg["message"].get("rendered", "").strip())
    if not failed:
        failed["<build>"] = r.stderr.strip()[-2000:]
    return [(names.get(k), v) for k, v in failed.items()]


def check_python(blocks, workdir, python):
    """Syntax-check each example, then resolve every `mdka` symbol it names.

    Executing them outright is not possible -- several convert files that do
    not exist here, so the run would assert on the fixture rather than on the
    example. Resolving the symbols against the *installed* package is the part
    that catches a documented function which does not exist, which syntax
    alone would pass.
    """
    import ast

    fails = []
    for n, b in enumerate(blocks):
        f = workdir / f"ex{n}.py"
        f.write_text(b.code, encoding="utf-8")
        r = subprocess.run(
            [sys.executable, "-m", "py_compile", str(f)], capture_output=True, text=True
        )
        if r.returncode != 0:
            fails.append((b, r.stderr.strip()))
            continue
        if not python:
            continue
        try:
            tree = ast.parse(b.code)
        except SyntaxError:
            continue
        names = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.ImportFrom) and (node.module or "").split(".")[0] == "mdka":
                names.update(a.name for a in node.names)
            elif (
                isinstance(node, ast.Attribute)
                and isinstance(node.value, ast.Name)
                and node.value.id == "mdka"
            ):
                names.add(node.attr)
        if not names:
            continue
        probe = (
            "import mdka, sys\n"
            f"missing = [n for n in {sorted(names)!r} if not hasattr(mdka, n)]\n"
            "sys.exit('missing from installed mdka: ' + ', '.join(missing)) if missing else None\n"
        )
        pf = workdir / f"probe{n}.py"
        pf.write_text(probe, encoding="utf-8")
        pr = subprocess.run([python, str(pf)], capture_output=True, text=True)
        if pr.returncode != 0:
            fails.append((b, (pr.stdout + pr.stderr).strip()))
    return fails


def check_js(blocks, workdir):
    fails = []
    for n, b in enumerate(blocks):
        ext = "mjs" if "import " in b.code else "js"
        f = workdir / f"ex_js{n}.{ext}"
        f.write_text(b.code, encoding="utf-8")
        r = subprocess.run(["node", "--check", str(f)], capture_output=True, text=True)
        if r.returncode != 0:
            fails.append((b, r.stderr.strip()))
    return fails


def check_ts(blocks, workdir, types_dir):
    """Type-check TypeScript examples against the real generated index.d.ts.

    `node --check` cannot parse TypeScript, so it would reject these for their
    annotations rather than for anything real. `D-13` was a TS example
    importing a type the bindings never exported -- only a type-checker that
    resolves `mdka` to the shipped declarations can catch that.
    """
    if not blocks:
        return []
    proj = workdir / "tscheck"
    (proj / "node_modules" / "mdka").mkdir(parents=True, exist_ok=True)
    dts = Path(types_dir) / "index.d.ts"
    if not dts.exists():
        return [(blocks[0], f"cannot type-check: {dts} not found")]
    (proj / "node_modules" / "mdka" / "index.d.ts").write_text(
        dts.read_text(encoding="utf-8"), encoding="utf-8"
    )
    (proj / "node_modules" / "mdka" / "package.json").write_text(
        json.dumps({"name": "mdka", "version": "0.0.0", "types": "index.d.ts"}),
        encoding="utf-8",
    )
    fails = []
    for n, b in enumerate(blocks):
        f = proj / f"ex_ts{n}.ts"
        f.write_text(b.code, encoding="utf-8")
        r = subprocess.run(
            [
                "npx", "--yes", "--package", "typescript@5", "tsc",
                "--noEmit", "--skipLibCheck", "--target", "es2022",
                "--module", "esnext", "--moduleResolution", "node",
                str(f),
            ],
            cwd=proj,
            capture_output=True,
            text=True,
        )
        if r.returncode != 0:
            fails.append((b, (r.stdout + r.stderr).strip()))
    return fails


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default="docs/src")
    ap.add_argument("--types", default="node", help="dir holding the generated index.d.ts")
    ap.add_argument("--python", default="", help="interpreter with mdka installed; enables symbol resolution")
    ap.add_argument("--list", action="store_true", help="list blocks and exit")
    args = ap.parse_args()

    blocks = extract(args.root)
    run = runnable(blocks)

    if args.list:
        for b in blocks:
            mark = "RUN " if b in run else "skip"
            print(f"{mark} {b.lang:8} {sorted(b.markers)} {b.where}")
        print(f"\n{len(run)} runnable of {len(blocks)} fenced blocks")
        return 0

    by = {}
    for b in run:
        by.setdefault(b.lang, []).append(b)

    print(f"docs-example gate: {len(run)} runnable of {len(blocks)} fenced blocks")
    for lang, bs in sorted(by.items()):
        print(f"  {lang}: {len(bs)}")

    failures = []
    with tempfile.TemporaryDirectory() as td:
        wd = Path(td)
        failures += check_rust(by.get("rust", []), wd)
        failures += check_python(by.get("python", []), wd, args.python)
        failures += check_js(by.get("js", []), wd)
        failures += check_ts(by.get("ts", []), wd, args.types)

    if not failures:
        print("\nAll runnable examples OK.")
        return 0

    print(f"\n{len(failures)} failing example(s):\n")
    for b, err in failures:
        print(f"--- {b.where if b else '<unknown>'} ---")
        print(err)
        print()
    return 1


if __name__ == "__main__":
    sys.exit(main())
