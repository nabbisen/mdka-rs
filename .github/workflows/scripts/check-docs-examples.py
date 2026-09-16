#!/usr/bin/env python3
"""Execute every runnable example in docs/src/ (RFC 026 docs-example gate).

A fenced block is runnable when its info string names a language we can run:
rust, python, js, ts. Everything else -- including an unlabelled fence, which
is how output samples are written -- is ignored.

A block that is deliberately a fragment is marked by adding `fragment` to the
info string (```rust,fragment). The marker is what excludes it: there is no
list of exceptions maintained elsewhere that can drift out of step with the
documents. Rust's own `ignore` is honoured too, since rustdoc gives it this
meaning. `no_run` is NOT a skip: it means "compile but do not run", so such a
block is still compiled.

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
# `no_run` is deliberately NOT here. It means "compile, but do not run" -- and
# this gate only ever compiles, so a `no_run` block must still be built. RFC
# 031: it used to be listed, which silently removed any `no_run` block from
# the gate entirely. A block with a type error marked `no_run` passed as
# "0 runnable of 1". Only markers meaning "do not compile" belong here.
SKIP_MARKERS = {"fragment", "ignore", "text", "compile_fail"}
FENCE = re.compile(r"^(?P<indent>\s*)```(?P<info>[A-Za-z0-9,_+-]*)\s*$")


class Block:
    def __init__(self, path, line, lang, markers, code):
        self.path, self.line, self.lang = path, line, lang
        self.markers, self.code = markers, code

    @property
    def where(self):
        return f"{self.path}:{self.line}"


def markdown_files(roots):
    """Every markdown file under each root. A root may be a file or a directory.

    README.md is a root in its own right, not something reachable from
    `docs/src`. It renders on GitHub, crates.io, npmjs.com and PyPI, and it was
    invisible to this gate until RFC 029 -- the gate built to catch broken
    examples was pointed at a directory that excluded the most-read file in the
    project.
    """
    seen, out = set(), []
    for root in roots:
        p = Path(root)
        found = sorted(p.rglob("*.md")) if p.is_dir() else [p]
        for md in found:
            if md.resolve() not in seen:
                seen.add(md.resolve())
                out.append(md)
    return out


def extract(roots):
    """Yield every fenced block under `roots`, in document order."""
    blocks = []
    for md in markdown_files(roots):
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


LINK = re.compile(r"!?\[[^\]]*\]\(([^)\s]+)")


def check_readme_links(path):
    """Assert README.md carries no relative or root-absolute Markdown link.

    Deliberately a grep, not a link checker: nothing is resolved, followed or
    crawled. README.md is published inside the npm tarball (four files) and
    rendered on npmjs.com and the PyPI project page, where a relative path
    resolves against the registry rather than the repository. `./CHANGELOG.md`,
    `./docs/` and a root-absolute logo path were all live and broken on both
    registries at 2.2.2.

    Scoped to README.md alone. `docs/src/` is rendered by mdbook, where
    relative links are correct and expected.
    """
    p = Path(path)
    if not p.exists():
        return []
    # Strip fenced blocks: a `](` inside a code sample is not a link.
    text, fenced = [], False
    for line in p.read_text(encoding="utf-8").splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
            continue
        text.append("" if fenced else line)

    bad = []
    for n, line in enumerate(text, 1):
        for target in LINK.findall(line):
            if target.startswith("#") or "://" in target or target.startswith("mailto:"):
                continue
            bad.append(f"{p}:{n}: relative or root-absolute link: {target}")
    return bad


def runnable(blocks):
    out = []
    for b in blocks:
        if b.lang not in RUNNABLE:
            continue
        if b.markers & SKIP_MARKERS:
            continue
        out.append(b)
    return out


def mdbook_rust_source(code):
    """Return the Rust source mdBook actually compiles for a block.

    mdBook hides lines from the rendered page but still compiles them. The rule
    is line-based and applies after leading whitespace:

      `# ` + rest   -> hidden; compile `rest`
      exactly `#`   -> hidden; compile an empty line
      `##` + rest   -> escape; compile with ONE `#` removed
      `#!` or `#[`  -> an attribute; compile untouched

    The last row is the trap. A naive `startswith("#")` would strip
    `#[derive(Debug)]` and `#![allow(...)]` from every example -- failing in
    confusing ways, or worse, passing because an attribute that mattered
    vanished.

    It is deliberately not string-literal aware, because mdBook is not: a
    `## heading` line inside a raw string compiles as `# heading`.
    """
    out = []
    for line in code.split("\n"):
        body = line.lstrip()
        indent = line[: len(line) - len(body)]
        if body.startswith("#!") or body.startswith("#["):
            out.append(line)
        elif body.startswith("##"):
            out.append(indent + body[1:])
        elif body == "#":
            out.append("")
        elif body.startswith("# "):
            out.append(indent + body[2:])
        else:
            out.append(line)
    return "\n".join(out)


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
        # Hidden lines first, so `fn main` detection sees what mdBook compiles.
        # Otherwise a hidden `# fn main() -> Result<...>` would be seen as
        # "has fn main", skip wrapping, and hand `# fn main` to rustc.
        code = mdbook_rust_source(b.code)
        # Model mdBook, not rustdoc. mdBook is what publishes these pages, and
        # it wraps a block with no `fn main` in a plain `fn main() {` -- no
        # Result, whatever the block contains. RFC 031: this used to wrap a
        # `?`-using block in `fn main() -> Result<...>`, "mirroring rustdoc",
        # which made the gate compile a MORE forgiving program than the one
        # behind the published page's Run button. Two usage-rust.md examples
        # failed with E0277 on the live site while this gate was green.
        #
        # The gate must never be more permissive than the renderer it vouches
        # for. If mdBook's wrapping is inconvenient for an example, fix the
        # example (a hidden fallible main) -- do not teach the gate to forgive.
        #
        # Since 031b the site has no Run button (docs/book.toml sets
        # playground.runnable = false: mdka is not in the playground's crate
        # set). The wrapping rule still holds -- it is what `mdbook test` and a
        # reader pasting the visible code under a plain `fn main` both get, and
        # the hidden `# fn main` lines added for D1 only make sense against it.
        #
        # The body is not indented: mdBook does not, and indenting would inject
        # spaces into a multi-line raw string literal.
        if "fn main" not in code:
            code = "fn main() {\n%s\n}" % code
        name = f"ex{n}"
        names[name] = b
        # mdBook prepends `#![allow(unused)]`; this adds `deprecated`, because
        # examples set deprecated fields on purpose. It never changes pass/fail
        # -- warnings do not fail a normal build -- but it is a SUBSTITUTION
        # (RFC 031 §6) with a consequence: THIS GATE CANNOT VERIFY ANY CLAIM
        # ABOUT DEPRECATION WARNINGS. api/options.md says its snippet "builds
        # under -D warnings"; delete that snippet's narrow #[allow] and this
        # gate stays green. Such claims are verified by executing them under
        # -D warnings instead -- for now the D7 captures kept with the RFC 031
        # and 031b review requests. Do not cite a green gate for them.
        #
        # Removing `deprecated` here would not fix that (the gate still would
        # not build with -D warnings); the comment is the fix.
        (proj / "src" / "bin" / f"{name}.rs").write_text(
            "#![allow(unused, deprecated)]\n" + code, encoding="utf-8"
        )
    # --keep-going: without it cargo stops scheduling after the first failing
    # bin, so how many failures are reported depends on how many were already
    # compiling -- five on a 32-core machine, one on a CI runner, for the same
    # documents (RFC 031). The gate still went red either way, but a reader
    # fixing the one reported example would go red again, one per round trip.
    r = subprocess.run(
        ["cargo", "build", "--keep-going", "--message-format=json"],
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


# The only files any Python example reads, listed explicitly -- see JS_FIXTURES.
#
# `missing.html` is deliberately ABSENT. usage-python.md's Error Handling example
# reads it on purpose and catches mdka.MdkaError. A fixture set built from "every
# filename any example mentions" would create it; the example would then convert
# successfully, never enter its `except` branch, exit 0, and pass without
# exercising the thing it documents.
PY_FIXTURES = ("page.html", "a.html", "b.html", "c.html")

# One targeted output check, not a general output-matching mechanism: an example
# that names this file exists to show the error path, and exit 0 is also what a
# broken error path produces. If no example names it any more, the gate fails
# rather than letting the check silently stop applying.
PY_ERROR_PATH = ("missing.html", "Conversion failed:")


def check_python(blocks, workdir, python):
    """Syntax-check, resolve symbols and kwargs, then EXECUTE each example.

    Resolution first, because its messages are specific: a documented function
    or keyword argument that the installed package does not have. Then each
    example runs with the `--python` interpreter, in a fresh temporary directory
    holding PY_FIXTURES, with a timeout -- as a reader running the page would.
    RFC 032: resolution alone passed usage-python.md:28, which raises NameError.

    SUBSTITUTIONS, per RFC 031 section 6:
      - It runs a wheel built from this tree, not the package on PyPI.
        Packaging defects in that wheel are the `pypi wheel gate`'s; what is
        actually published is checked by neither gate, only by the release
        consumer pass.
      - Fixtures are one-line files, not real documents.
      - Exit 0 is the pass condition, plus the one PY_ERROR_PATH output check.

    NOT PARITY WITH check_js. There, an example reading a file outside the
    fixtures fails loudly. Here, reading an unknown file raises MdkaError: loud
    (exit 1) if uncaught, but an example that catches it exits 0 and passes.
    PY_ERROR_PATH covers the one example written that way today; a new one
    would need its own check.
    """
    import ast

    if not python:
        return [(b, "cannot execute: no --python interpreter with mdka installed")
                for b in blocks]

    fails = []
    error_path_seen = False
    for n, b in enumerate(blocks):
        f = workdir / f"ex{n}.py"
        f.write_text(b.code, encoding="utf-8")
        r = subprocess.run(
            [sys.executable, "-m", "py_compile", str(f)], capture_output=True, text=True
        )
        if r.returncode != 0:
            fails.append((b, r.stderr.strip()))
            continue
        resolution = _python_resolution(b, n, workdir, python)
        if resolution:
            fails.append((b, resolution))
            continue

        box = workdir / f"py{n}"
        box.mkdir(parents=True, exist_ok=True)
        for name in PY_FIXTURES:
            (box / name).write_text(f"<h1>{name}</h1>", encoding="utf-8")
        (box / "example.py").write_text(b.code, encoding="utf-8")
        try:
            r = subprocess.run([python, "example.py"], cwd=box,
                               capture_output=True, text=True, timeout=60)
        except subprocess.TimeoutExpired:
            fails.append((b, "timed out after 60s"))
            continue
        if r.returncode != 0:
            fails.append((b, (r.stderr or r.stdout).strip()[-2000:]))
            continue

        name, expected = PY_ERROR_PATH
        if name in b.code:
            error_path_seen = True
            if expected not in r.stdout:
                fails.append((b, f"exited 0 but did not print {expected!r}: the "
                                 "error path it documents was not exercised\n"
                                 f"stdout: {r.stdout.strip()!r}"))

    if blocks and not error_path_seen:
        fails.append((blocks[0], f"no Python example names {PY_ERROR_PATH[0]!r} any more; "
                                 "the error-path output check no longer applies -- "
                                 "update PY_ERROR_PATH"))
    return fails


def _python_resolution(b, n, workdir, python):
    """Return an error message if a documented symbol or kwarg does not resolve."""
    import ast

    try:
        tree = ast.parse(b.code)
    except SyntaxError:
        return None
    names = set()
    imported = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and (node.module or "").split(".")[0] == "mdka":
            names.update(a.name for a in node.names)
            imported.update(a.asname or a.name for a in node.names)
        elif (
            isinstance(node, ast.Attribute)
            and isinstance(node.value, ast.Name)
            and node.value.id == "mdka"
        ):
            names.add(node.attr)

    # Keyword arguments, not just symbols. `html_to_markdown_with` resolves
    # whether or not `preserve_unknown_attrs=True` is a real parameter --
    # and it is not; it raises TypeError. Documenting a call that raises is
    # worse than documenting nothing, so bind the documented kwargs against
    # the installed signature (RFC 029 §4.2).
    calls = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not node.keywords:
            continue
        fn = node.func
        if isinstance(fn, ast.Attribute) and isinstance(fn.value, ast.Name) and fn.value.id == "mdka":
            target = fn.attr
        elif isinstance(fn, ast.Name) and fn.id in imported:
            target = fn.id
        else:
            continue
        kws = [k.arg for k in node.keywords if k.arg]
        if kws:
            calls.append((target, kws))

    if not names and not calls:
        return None
    probe = (
        "import inspect, mdka, sys\n"
        f"missing = [n for n in {sorted(names)!r} if not hasattr(mdka, n)]\n"
        "if missing:\n"
        "    sys.exit('missing from installed mdka: ' + ', '.join(missing))\n"
        f"problems = []\n"
        f"for fname, kws in {calls!r}:\n"
        "    fn = getattr(mdka, fname, None)\n"
        "    if fn is None:\n"
        "        problems.append(fname + ': not in installed mdka'); continue\n"
        "    try:\n"
        "        sig = inspect.signature(fn)\n"
        "    except (TypeError, ValueError):\n"
        "        continue\n"
        "    params = sig.parameters\n"
        "    if any(p.kind is inspect.Parameter.VAR_KEYWORD for p in params.values()):\n"
        "        continue\n"
        "    bad = [k for k in kws if k not in params]\n"
        "    if bad:\n"
        "        problems.append(fname + '() rejects: ' + ', '.join(bad))\n"
        "if problems:\n"
        "    sys.exit('documented call does not match the installed signature -- '\n"
        "             + '; '.join(problems))\n"
    )
    pf = workdir / f"probe{n}.py"
    pf.write_text(probe, encoding="utf-8")
    pr = subprocess.run([python, str(pf)], capture_output=True, text=True)
    if pr.returncode != 0:
        return (pr.stdout + pr.stderr).strip()
    return None


# The only files any JS example reads. Checked when this was written: six of
# the eight JS blocks read none, the other two read these. An example that
# reads a file not listed here fails -- which is a loud false positive, never a
# silent pass, so the list cannot hide a defect by being incomplete.
JS_FIXTURES = ("page.html", "a.html", "b.html", "c.html")


def check_js(blocks, workdir, binding_dir):
    """Parse each example, then EXECUTE it as a consumer would.

    `node --check` is a syntax check, and it is not what a consumer does. RFC
    031: it passed a block that failed at load with ERR_AMBIGUOUS_MODULE_SYNTAX
    (`require()` plus top-level `await`) and would then have hit undefined
    variables. Neither is a syntax error, so the check could not see either.

    Execution runs against the locally built binding, installed as
    `node_modules/mdka` in a fresh sandbox per example, so one example's output
    files cannot affect another's result.

    SUBSTITUTION (RFC 031 §6). A consumer does not get this: they get the
    published `index.js`, which resolves a platform package (`@mdka/lib-*`)
    through `optionalDependencies`. Copying a local `.node` beside `index.js`
    skips that resolution entirely, so a resolution defect -- RFC 020's class,
    where `npm install mdka` was unusable for twelve releases -- passes here.
    That class is covered by the `npm install gate`, which installs the
    published package from the registry. Do not cite this gate for it.

    What this still lets through, that a consumer would hit:
      - an example that exits 0 but prints output other than the page claims;
      - a failure only on macOS or Windows, or only on real-sized input rather
        than the one-line fixtures;
      - a deprecated option in an example: a DeprecationWarning does not change
        the exit code, and the Async functions never warn at all;
      - a package-resolution defect, per the substitution above.
    An example reading a file outside JS_FIXTURES fails loudly, never passes.
    """
    binding = Path(binding_dir)
    have_binding = (binding / "index.js").exists() and any(binding.glob("*.node"))
    fails = []
    for n, b in enumerate(blocks):
        ext = "mjs" if "import " in b.code else "js"
        box = workdir / f"js{n}"
        box.mkdir(parents=True, exist_ok=True)
        f = box / f"example.{ext}"
        f.write_text(b.code, encoding="utf-8")

        r = subprocess.run(["node", "--check", str(f)], capture_output=True, text=True)
        if r.returncode != 0:
            fails.append((b, r.stderr.strip()))
            continue
        if not have_binding:
            fails.append((b, f"cannot execute: no built binding in {binding} "
                             "(run `npm run build` in node/ first)"))
            continue

        mod = box / "node_modules" / "mdka"
        mod.mkdir(parents=True, exist_ok=True)
        for src in [binding / "index.js", binding / "index.d.ts", *binding.glob("*.node")]:
            if src.exists():
                (mod / src.name).write_bytes(src.read_bytes())
        (mod / "package.json").write_text(
            json.dumps({"name": "mdka", "main": "index.js", "types": "index.d.ts"}),
            encoding="utf-8",
        )
        for name in JS_FIXTURES:
            (box / name).write_text(f"<h1>{name}</h1>", encoding="utf-8")

        try:
            r = subprocess.run(["node", f.name], cwd=box,
                               capture_output=True, text=True, timeout=60)
        except subprocess.TimeoutExpired:
            fails.append((b, "timed out after 60s"))
            continue
        if r.returncode != 0:
            fails.append((b, (r.stderr or r.stdout).strip()[-2000:]))
    return fails


# TypeScript is pinned so the gate is reproducible, and compiled with the options
# `tsc --init` writes in that version. That is what a reader starting a project
# the standard way and pasting the page's example has. RFC 032: the gate used to
# compile with looser, hand-picked flags; under these, usage-nodejs.md's
# example failed with TS1484 (`verbatimModuleSyntax`) while the gate was green.
TYPESCRIPT = "typescript@5.9.3"
TSC_INIT_OPTIONS = {
    # Written by `tsc --init` in TypeScript 5.9.3, except output-only options
    # (sourceMap, declaration, declarationMap, jsx) which cannot affect whether
    # the example compiles or runs.
    "module": "nodenext",
    "target": "esnext",
    "types": [],
    "noUncheckedIndexedAccess": True,
    "exactOptionalPropertyTypes": True,
    "strict": True,
    "verbatimModuleSyntax": True,
    "isolatedModules": True,
    "noUncheckedSideEffectImports": True,
    "moduleDetection": "force",
    "skipLibCheck": True,
}


def check_ts(blocks, workdir, binding_dir):
    """Type-check, emit and EXECUTE each TypeScript example as a consumer would.

    Each example gets a fresh sandbox: a `"type": "module"` package, the built
    binding and its index.d.ts installed as `node_modules/mdka`, and a
    tsconfig.json holding TSC_INIT_OPTIONS. `tsc -p .` then emits JavaScript,
    which runs with `node`. D-13 -- a type the bindings never exported -- is a
    compile error here; an example that compiles but throws at load or run is an
    execution error.

    Other plausible settings, checked when this was written: CommonJS output
    (`--module commonjs`, no `"type"`) compiled and ran the same example; ESM
    without `verbatimModuleSyntax` did too. Neither is used, because the stricter
    `tsc --init` default is the one a new project gets.

    SUBSTITUTIONS, per RFC 031 section 6:
      - It runs the locally built binding, not the published npm package; a
        package-resolution defect (RFC 020's class) is the `npm install gate`'s.
        When this was written, the example under ESM `nodenext` without
        `verbatimModuleSyntax` also compiled and ran against mdka@2.2.3 installed
        from the registry, matching the local binding.
      - Exit 0 is the pass condition; output is not compared.
    """
    if not blocks:
        return []
    binding = Path(binding_dir)
    have_binding = (binding / "index.js").exists() and (binding / "index.d.ts").exists() \
        and any(binding.glob("*.node"))
    if not have_binding:
        return [(b, f"cannot type-check or execute: no built binding in {binding} "
                    "(run `npm run build` in node/ first)") for b in blocks]

    fails = []
    for n, b in enumerate(blocks):
        box = workdir / f"ts{n}"
        mod = box / "node_modules" / "mdka"
        mod.mkdir(parents=True, exist_ok=True)
        for src in [binding / "index.js", binding / "index.d.ts", *binding.glob("*.node")]:
            (mod / src.name).write_bytes(src.read_bytes())
        (mod / "package.json").write_text(
            json.dumps({"name": "mdka", "main": "index.js", "types": "index.d.ts"}),
            encoding="utf-8",
        )
        (box / "package.json").write_text(json.dumps({"type": "module"}), encoding="utf-8")
        (box / "tsconfig.json").write_text(
            json.dumps({"compilerOptions": TSC_INIT_OPTIONS, "files": ["example.ts"]}),
            encoding="utf-8",
        )
        (box / "example.ts").write_text(b.code, encoding="utf-8")

        r = subprocess.run(
            ["npx", "--yes", "--package", TYPESCRIPT, "tsc", "-p", "."],
            cwd=box, capture_output=True, text=True,
        )
        if r.returncode != 0:
            fails.append((b, (r.stdout + r.stderr).strip()))
            continue
        try:
            r = subprocess.run(["node", "example.js"], cwd=box,
                               capture_output=True, text=True, timeout=60)
        except subprocess.TimeoutExpired:
            fails.append((b, "timed out after 60s"))
            continue
        if r.returncode != 0:
            fails.append((b, (r.stderr or r.stdout).strip()[-2000:]))
    return fails


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", action="append", default=None,
                    help="file or directory to scan; repeatable "
                         "(default: docs/src and README.md)")
    ap.add_argument("--types", default="node",
                    help="dir holding the built binding: index.js, index.d.ts and the .node")
    ap.add_argument("--python", default="", help="interpreter with mdka installed; enables symbol resolution")
    ap.add_argument("--list", action="store_true", help="list blocks and exit")
    args = ap.parse_args()

    # Fail loudly rather than with a traceback from deep inside subprocess:
    # a missing interpreter means the symbol-resolution check would silently
    # not happen, and a gate that degrades quietly is worse than one that stops.
    if args.python and not Path(args.python).exists():
        print(f"error: --python {args.python} does not exist", file=sys.stderr)
        return 2

    roots = args.root or ["docs/src", "README.md"]
    blocks = extract(roots)
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
        failures += check_js(by.get("js", []), wd, args.types)
        failures += check_ts(by.get("ts", []), wd, args.types)

    link_problems = check_readme_links("README.md")
    if link_problems:
        print(f"\n{len(link_problems)} README link problem(s):\n")
        for msg in link_problems:
            print("  " + msg)
        print(
            "\nREADME.md ships inside the npm tarball and renders on npmjs.com\n"
            "and the PyPI project page, where a relative path resolves against the\n"
            "registry rather than the repository. Use an absolute URL."
        )

    if not failures and not link_problems:
        print("\nAll runnable examples OK. README links OK.")
        return 0
    if not failures:
        return 1

    print(f"\n{len(failures)} failing example(s):\n")
    for b, err in failures:
        print(f"--- {b.where if b else '<unknown>'} ---")
        print(err)
        print()
    return 1


if __name__ == "__main__":
    sys.exit(main())
