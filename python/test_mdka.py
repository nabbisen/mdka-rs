"""
mdka Python バインディング テストスイート (pytest)

カバレッジ:
  - html_to_markdown    : 全主要タグ・エスケープ・エッジケース
  - html_to_markdown_many : 並列変換・順序保証
  - html_files_to_markdown: ファイル変換・エラーハンドリング
  - FileOutcome          : ok プロパティ・repr
  - MdkaError            : 例外送出
  - version              : バージョン文字列
"""

import os
import tempfile
from pathlib import Path

import pytest
import mdka
from mdka import (
    html_to_markdown,
    html_to_markdown_with,
    html_to_markdown_many,
    html_to_markdown_many_with,
    html_file_to_markdown,
    html_file_to_markdown_with,
    html_files_to_markdown,
    html_files_to_markdown_with,
    ConversionMode,
    FileOutcome,
    MdkaError,
    version,
)


# ─── version ─────────────────────────────────────────────────────────────────

def test_version_returns_semver():
    v = version()
    assert isinstance(v, str)
    parts = v.split(".")
    assert len(parts) == 3
    assert all(p.isdigit() for p in parts)

def test_module_version_attribute():
    assert mdka.__version__ == version()


# ─── html_to_markdown: ヘッダ ────────────────────────────────────────────────

@pytest.mark.parametrize("level", range(1, 7))
def test_headings(level):
    md = html_to_markdown(f"<h{level}>Title</h{level}>")
    assert md.strip() == "#" * level + " Title"

def test_heading_no_leading_newline():
    md = html_to_markdown("<h1>Hello</h1>")
    assert not md.startswith("\n")
    assert md == "# Hello\n"


# ─── html_to_markdown: ブロック要素 ──────────────────────────────────────────

def test_paragraph():
    assert html_to_markdown("<p>Hello world</p>").strip() == "Hello world"

def test_two_paragraphs_blank_line():
    md = html_to_markdown("<p>First</p><p>Second</p>")
    assert "\n\n" in md
    assert "First" in md and "Second" in md

def test_blockquote():
    md = html_to_markdown("<blockquote><p>Quoted</p></blockquote>")
    assert "> " in md
    assert "Quoted" in md

def test_blockquote_nested():
    md = html_to_markdown(
        "<blockquote><blockquote><p>Deep</p></blockquote></blockquote>"
    )
    assert "> > " in md
    assert "Deep" in md

def test_horizontal_rule():
    md = html_to_markdown("<hr>")
    assert "---" in md


# ─── html_to_markdown: インライン要素 ────────────────────────────────────────

def test_strong():
    assert "**bold**" in html_to_markdown("<strong>bold</strong>")

def test_em():
    assert "*italic*" in html_to_markdown("<em>italic</em>")

def test_inline_code():
    md = html_to_markdown("<code>snippet</code>")
    assert "`snippet`" in md

def test_link_basic():
    md = html_to_markdown('<a href="https://example.com">Click</a>')
    assert "[Click](https://example.com)" in md

def test_link_with_title():
    md = html_to_markdown('<a href="https://example.com" title="Ex">Click</a>')
    assert '[Click](https://example.com "Ex")' in md

def test_image():
    md = html_to_markdown('<img src="img.png" alt="Alt">')
    assert "![Alt](img.png)" in md

def test_image_with_title():
    md = html_to_markdown('<img src="img.png" alt="Alt" title="Cap">')
    assert '![Alt](img.png "Cap")' in md

def test_line_break():
    md = html_to_markdown("<p>line1<br>line2</p>")
    assert "  \n" in md


# ─── html_to_markdown: リスト ────────────────────────────────────────────────

def test_unordered_list():
    md = html_to_markdown("<ul><li>A</li><li>B</li><li>C</li></ul>")
    assert "- A" in md
    assert "- B" in md
    assert "- C" in md

def test_ordered_list():
    md = html_to_markdown("<ol><li>One</li><li>Two</li></ol>")
    assert "1. One" in md
    assert "2. Two" in md

def test_ordered_list_start_attr():
    md = html_to_markdown('<ol start="5"><li>Five</li><li>Six</li></ol>')
    assert "5. Five" in md
    assert "6. Six" in md

def test_nested_list():
    md = html_to_markdown("<ul><li>P<ul><li>C</li></ul></li></ul>")
    assert "- P" in md
    assert "  - C" in md


# ─── html_to_markdown: コードブロック ────────────────────────────────────────

def test_code_block_no_lang():
    md = html_to_markdown("<pre><code>fn main() {}</code></pre>")
    assert "```\nfn main() {}" in md

def test_code_block_with_lang():
    md = html_to_markdown(
        '<pre><code class="language-python">print("hi")</code></pre>'
    )
    assert "```python\nprint" in md

def test_pre_preserves_whitespace():
    md = html_to_markdown("<pre><code>line1\n  line2\n    line3</code></pre>")
    assert "  line2" in md
    assert "    line3" in md


# ─── html_to_markdown: エスケープ ────────────────────────────────────────────

def test_escape_asterisk():
    # RFC 010: `*` is escaped only where it could open or close emphasis.
    assert html_to_markdown("<p>2 * 3</p>") == "2 * 3\n"
    assert r"\*" in html_to_markdown("<p>2 *3*</p>")

def test_escape_hash_at_line_start():
    md = html_to_markdown("<p># not a heading</p>")
    assert r"\#" in md

def test_escape_backtick():
    md = html_to_markdown("<p>`code`</p>")
    assert r"\`" in md


# ─── html_to_markdown: 無視タグ ──────────────────────────────────────────────

def test_script_ignored():
    md = html_to_markdown("<script>alert(1)</script><p>Visible</p>")
    assert "alert" not in md
    assert "Visible" in md

def test_style_ignored():
    md = html_to_markdown("<style>body{color:red}</style><p>Text</p>")
    assert "color" not in md
    assert "Text" in md


# ─── html_to_markdown: エッジケース ──────────────────────────────────────────

def test_empty_input():
    assert html_to_markdown("").strip() == ""

def test_whitespace_only():
    assert html_to_markdown("   \n\t  ").strip() == ""

def test_output_ends_with_single_newline():
    md = html_to_markdown("<p>Hello</p>")
    assert md.endswith("\n")
    assert not md.endswith("\n\n")

def test_no_leading_newline():
    for html in ["<h1>T</h1>", "<p>T</p>", "<ul><li>T</li></ul>"]:
        md = html_to_markdown(html)
        assert not md.startswith("\n"), f"leading newline for {html!r}: {md!r}"

def test_deep_nest_no_crash():
    html = "<div>" * 5000 + "<p>deep</p>" + "</div>" * 5000
    md = html_to_markdown(html)
    assert "deep" in md

def test_malformed_html_no_crash():
    html = "<p>Unclosed<ul><li>Item<li>Item2<strong>no close"
    md = html_to_markdown(html)
    assert isinstance(md, str)

def test_unicode_content():
    md = html_to_markdown("<p>日本語テスト 🎉</p>")
    assert "日本語テスト" in md
    assert "🎉" in md

def test_html_entities_decoded():
    md = html_to_markdown("<p>&amp; &lt; &gt;</p>")
    assert "&" in md


# ─── html_to_markdown_many ───────────────────────────────────────────────────

def test_many_basic():
    results = html_to_markdown_many(["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"])
    assert results[0] == "# A\n"
    assert results[1] == "B\n"
    assert "- C" in results[2]

def test_many_preserves_order():
    inputs = [f"<h1>Item {i}</h1>" for i in range(50)]
    results = html_to_markdown_many(inputs)
    assert len(results) == 50
    for i, md in enumerate(results):
        assert f"Item {i}" in md

def test_many_empty_list():
    assert html_to_markdown_many([]) == []

def test_many_large_batch():
    inputs = ["<p>" + "word " * 1000 + "</p>"] * 20
    results = html_to_markdown_many(inputs)
    assert len(results) == 20
    assert all("word" in r for r in results)


# ─── html_to_markdown_many_with ───────────────────────────────────────────────

def test_many_with_matches_mapping_html_to_markdown_with():
    inputs = ["<h1>A</h1>", "<p>B</p>", "<ul><li>C</li></ul>"]
    results = html_to_markdown_many_with(inputs, mode=ConversionMode.Minimal)
    expected = [html_to_markdown_with(h, mode=ConversionMode.Minimal) for h in inputs]
    assert results == expected

def test_many_with_default_mode_matches_html_to_markdown_many():
    inputs = ["<h1>A</h1>", "<p>B</p>"]
    assert html_to_markdown_many_with(inputs) == html_to_markdown_many(inputs)

def test_many_with_empty_list():
    assert html_to_markdown_many_with([]) == []

def test_many_with_drop_interactive_shell():
    inputs = ['<nav><a href="/">Home</a></nav><main><p>Content</p></main>']
    results = html_to_markdown_many_with(
        inputs, mode=ConversionMode.Minimal, drop_interactive_shell=True
    )
    assert "Home" not in results[0]
    assert "Content" in results[0]


# ─── html_files_to_markdown ──────────────────────────────────────────────────

def test_file_conversion_basic(tmp_path):
    src = tmp_path / "page.html"
    src.write_text("<h1>File Test</h1><p>Content here</p>")
    out_dir = tmp_path / "out"

    results = html_files_to_markdown([str(src)], str(out_dir))

    assert len(results) == 1
    r = results[0]
    assert r.ok
    assert r.dest is not None
    assert r.error is None
    content = Path(r.dest).read_text()
    assert "# File Test" in content
    assert "Content here" in content

def test_file_conversion_multiple(tmp_path):
    files = []
    for i in range(4):
        p = tmp_path / f"f{i}.html"
        p.write_text(f"<h{i+1}>Title {i}</h{i+1}>")
        files.append(str(p))
    out_dir = tmp_path / "out"

    results = html_files_to_markdown(files, str(out_dir))
    assert len(results) == 4
    assert all(r.ok for r in results)

def test_file_conversion_creates_out_dir(tmp_path):
    src = tmp_path / "test.html"
    src.write_text("<p>Hello</p>")
    out_dir = tmp_path / "new" / "nested" / "dir"
    assert not out_dir.exists()

    results = html_files_to_markdown([str(src)], str(out_dir))
    assert out_dir.exists()
    assert results[0].ok

def test_file_conversion_nonexistent_file(tmp_path):
    results = html_files_to_markdown(
        [str(tmp_path / "ghost.html")], str(tmp_path)
    )
    assert len(results) == 1
    r = results[0]
    assert not r.ok
    assert r.error is not None
    assert r.dest is None

def test_file_conversion_mixed_results(tmp_path):
    good = tmp_path / "good.html"
    good.write_text("<p>OK</p>")
    out_dir = tmp_path / "out"

    results = html_files_to_markdown(
        [str(good), str(tmp_path / "missing.html")], str(out_dir)
    )
    assert len(results) == 2
    ok_results    = [r for r in results if r.ok]
    error_results = [r for r in results if not r.ok]
    assert len(ok_results) == 1
    assert len(error_results) == 1


# ─── html_files_to_markdown_with ──────────────────────────────────────────────

def test_files_with_matches_html_files_to_markdown_default(tmp_path):
    src = tmp_path / "page.html"
    src.write_text("<h1>File Test</h1><p>Content here</p>")
    out_a = tmp_path / "out_a"
    out_b = tmp_path / "out_b"

    r_with = html_files_to_markdown_with([str(src)], str(out_a))
    r_plain = html_files_to_markdown([str(src)], str(out_b))

    assert r_with[0].ok and r_plain[0].ok
    assert Path(r_with[0].dest).read_text() == Path(r_plain[0].dest).read_text()

def test_files_with_mode_option(tmp_path):
    src = tmp_path / "spa.html"
    src.write_text("<nav>nav</nav><h1>Title</h1><p>Body</p>")
    out_dir = tmp_path / "out"

    results = html_files_to_markdown_with(
        [str(src)], str(out_dir),
        mode=ConversionMode.Minimal, drop_interactive_shell=True,
    )
    assert results[0].ok
    content = Path(results[0].dest).read_text()
    assert "# Title" in content
    assert "nav" not in content.lower()

def test_files_with_error_result(tmp_path):
    results = html_files_to_markdown_with(
        [str(tmp_path / "ghost.html")], str(tmp_path)
    )
    assert len(results) == 1
    assert not results[0].ok
    assert results[0].error is not None


# ─── FileOutcome ─────────────────────────────────────────────────────────────

def test_file_outcome_ok_property(tmp_path):
    src = tmp_path / "x.html"
    src.write_text("<p>x</p>")
    results = html_files_to_markdown([str(src)], str(tmp_path))
    r = results[0]
    assert isinstance(r, FileOutcome)
    assert r.ok is True

def test_file_outcome_has_exactly_one_of_dest_and_error(tmp_path):
    good = tmp_path / "good.html"
    good.write_text("<p>x</p>")
    ok, bad = html_files_to_markdown([str(good), str(tmp_path / "ghost.html")], str(tmp_path / "out"))
    assert ok.ok is True and ok.dest is not None and ok.error is None
    assert bad.ok is False and bad.dest is None and isinstance(bad.error, str)
    for o in (ok, bad):
        assert o.ok == (o.dest is not None)
        assert o.ok == (o.error is None)

def test_file_outcome_names_its_own_file(tmp_path):
    # The outcome says which file it belongs to, so a failure needs no lookup.
    good = tmp_path / "good.html"
    good.write_text("<p>x</p>")
    missing = str(tmp_path / "missing.html")
    results = html_files_to_markdown([str(good), missing], str(tmp_path / "out"))
    assert [r.src for r in results] == [str(good), missing]

def test_file_outcome_repr_shows_all_four_fields_in_python_form(tmp_path):
    # RFC 048 criterion 11. It used to print a Rust Option -- error=Some("...") --
    # and omit dest and ok entirely.
    src = tmp_path / "r.html"
    src.write_text("<p>r</p>")
    ok = html_files_to_markdown([str(src)], str(tmp_path / "out"))[0]
    assert repr(ok) == f"FileOutcome(src={str(src)!r}, dest={ok.dest!r}, error=None, ok=True)"

    bad = html_files_to_markdown(["/no/such/file.html"], str(tmp_path / "out"))[0]
    rep = repr(bad)
    assert rep.startswith("FileOutcome(src='/no/such/file.html', dest=None, error='IO error: "), rep
    assert rep.endswith(", ok=False)"), rep
    for r in (ok, bad):
        assert "Some(" not in repr(r), repr(r)

def test_file_outcome_is_the_only_result_type():
    # ConvertResult and BulkConvertResult were removed in 3.0.
    assert not hasattr(mdka, "ConvertResult")
    assert not hasattr(mdka, "BulkConvertResult")
    assert not hasattr(mdka.mdka_python, "ConvertResult")
    assert not hasattr(mdka.mdka_python, "BulkConvertResult")

def test_a_failing_file_does_not_abort_the_batch(tmp_path):
    files = []
    for name in ("a", "missing", "b"):
        f = tmp_path / f"{name}.html"
        if name != "missing":
            f.write_text(f"<h1>{name}</h1>")
        files.append(str(f))
    results = html_files_to_markdown(files, str(tmp_path / "out"))  # must not raise
    assert [r.ok for r in results] == [True, False, True]
    assert (tmp_path / "out" / "a.md").exists() and (tmp_path / "out" / "b.md").exists()
    assert results[2].dest == str(tmp_path / "out" / "b.md"), "the file after the failure was converted"


# ─── MdkaError ───────────────────────────────────────────────────────────────

def test_mdka_error_is_exception():
    assert issubclass(MdkaError, Exception)

def test_mdka_error_raised_on_bad_out_dir():
    # 既存ファイルをディレクトリとして使うと create_dir_all が失敗する
    with tempfile.NamedTemporaryFile() as f:
        with pytest.raises(MdkaError):
            html_files_to_markdown([], f.name)  # ファイルパスをoutDirとして渡す


# ─── スタブ doctest ───────────────────────────────────────────────────────────

def test_module_all_exported():
    expected = {
        "html_to_markdown",
        "html_to_markdown_many",
        "html_files_to_markdown",
        "FileOutcome",
        "MdkaError",
        "version",
    }
    assert expected.issubset(set(mdka.__all__))


# ─── ConversionMode / html_to_markdown_with ──────────────────────────────────

def test_mode_enum_values():
    assert ConversionMode.Balanced != ConversionMode.Minimal

def test_the_removed_modes_are_gone():
    # 3.0: Strict, Semantic and Preserve were aliases of Balanced. They are not
    # deprecated any more, they are absent: the attribute does not exist.
    for name in ("Strict", "Semantic", "Preserve"):
        assert not hasattr(ConversionMode, name), name
        with pytest.raises(AttributeError):
            getattr(ConversionMode, name)

def test_minimal_drops_nav():
    md = html_to_markdown_with(
        "<nav><a href='/'>Home</a></nav><main><p>Content</p></main>",
        mode=ConversionMode.Minimal,
        drop_interactive_shell=True,
    )
    assert "Content" in md, f"content missing: {md}"
    assert "Home" not in md, f"nav leaked: {md}"

def test_balanced_keeps_href():
    md = html_to_markdown_with(
        '<a href="https://example.com">Link</a>',
        mode=ConversionMode.Balanced,
    )
    assert "[Link](https://example.com)" in md, f"got: {md}"

def test_with_drop_shell_flag():
    md = html_to_markdown_with(
        "<header>HEADER</header><h1>Title</h1><footer>FOOTER</footer>",
        mode=ConversionMode.Balanced,
        drop_interactive_shell=True,
    )
    assert "Title" in md
    assert "HEADER" not in md, f"header leaked: {md}"
    assert "FOOTER" not in md, f"footer leaked: {md}"

def test_a_wrapper_keeps_its_separation_in_both_modes():
    # Bare-sibling-text fixture, not a block-element fixture: neighbouring
    # blocks' own spacing dominates the output otherwise. Minimal unwraps the
    # wrapper and Balanced renders it; RFC 036 §5.2 / slice 036d made the two
    # write the same bytes.
    html = 'Before<div class="wrap"><span>inner</span></div>After'
    for mode in (ConversionMode.Balanced, ConversionMode.Minimal):
        assert html_to_markdown_with(html, mode=mode) == "Before\n\ninner\n\nAfter\n", mode

def test_the_removed_keyword_arguments_are_a_type_error():
    # 3.0: preserve_classes, preserve_data_attrs, preserve_aria_attrs and
    # unwrap_unknown_wrappers were removed. Python reports an unknown keyword
    # loudly, so unlike Node nothing needs to be pinned as a known gap.
    for kwarg in (
        "preserve_classes",
        "preserve_data_attrs",
        "preserve_aria_attrs",
        "preserve_unknown_attrs",
        "drop_presentation_attrs",
        "unwrap_unknown_wrappers",
    ):
        with pytest.raises(TypeError):
            html_to_markdown_with("<p>Hi</p>", **{kwarg: True})

def test_nothing_warns_any_more():
    import warnings
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        # Raises if any warning (including DeprecationWarning) is emitted.
        html_to_markdown_with("<p>Hi</p>")
        html_to_markdown_with("<p>Hi</p>", mode=ConversionMode.Balanced)
        html_to_markdown_with("<div><p>Hi</p></div>", mode=ConversionMode.Minimal)
        html_to_markdown_many_with(["<p>Hi</p>"])

def test_a_string_mode_is_a_type_error():
    # RFC 048 §2.2: Python has never accepted a mode as a string, so the removed
    # names ("strict", ...) never reached it and there is no "removed" message
    # to give -- a str where a ConversionMode belongs is a TypeError, whatever
    # the string says.
    for bad in ("strict", "balanced"):
        with pytest.raises(TypeError):
            html_to_markdown_with("<p>Hi</p>", mode=bad)

def test_html_files_with_mode(tmp_path):
    src = tmp_path / "page.html"
    src.write_text("<nav>nav</nav><h1>Title</h1><p>Content</p>")
    out_dir = tmp_path / "out"

    results = html_files_to_markdown(
        [str(src)], str(out_dir),
        mode=ConversionMode.Minimal,
        drop_interactive_shell=True,
    )
    assert results[0].ok
    content = (tmp_path / "out" / "page.md").read_text()
    assert "Title" in content
    assert "nav" not in content.lower(), f"nav leaked: {content}"

def test_mode_default_is_balanced():
    # モード省略時は balanced
    md1 = html_to_markdown("<p>Hello</p>")
    md2 = html_to_markdown_with("<p>Hello</p>")
    assert md1 == md2

def test_unknown_mode_uses_balanced():
    # from_str が None を返したとき balanced にフォールバック
    # → Python 側では ConversionMode enum しか渡せないので型安全
    md = html_to_markdown_with("<h1>Hi</h1>", mode=ConversionMode.Balanced)
    assert "# Hi" in md

def test_module_all_has_new_exports():
    assert "ConversionMode"              in mdka.__all__
    assert "html_to_markdown_with"       in mdka.__all__
    assert "html_to_markdown_many_with"  in mdka.__all__
    assert "html_file_to_markdown"       in mdka.__all__
    assert "html_file_to_markdown_with"  in mdka.__all__
    assert "html_files_to_markdown_with" in mdka.__all__
    assert "FileOutcome"                 in mdka.__all__


# ─── html_file_to_markdown ────────────────────────────────────────────────────

def test_file_to_markdown_same_dir(tmp_path):
    """out_dir=None のとき入力と同じディレクトリに .md を出力する。"""
    src = tmp_path / "page.html"
    src.write_text("<h1>Single File</h1><p>Content</p>")

    dest = html_file_to_markdown(str(src))  # out_dir 省略

    # 3.0: a single file returns the destination path, as a str -- not a result
    # object with an `error` that could never be set.
    assert isinstance(dest, str), f"expected the destination path, got {dest!r}"
    assert dest == str(tmp_path / "page.md")
    assert (tmp_path / "page.md").exists(), "output file not created"
    content = (tmp_path / "page.md").read_text()
    assert "# Single File" in content, f"got: {content}"
    assert "Content"       in content, f"got: {content}"

def test_file_to_markdown_explicit_out_dir(tmp_path):
    """out_dir を指定するとそのディレクトリに出力する。"""
    src     = tmp_path / "article.html"
    out_dir = tmp_path / "out"
    out_dir.mkdir()
    src.write_text("<h2>Article</h2><p>Body</p>")

    dest = html_file_to_markdown(str(src), str(out_dir))

    assert dest == str(out_dir / "article.md")
    content = (out_dir / "article.md").read_text()
    assert "## Article" in content, f"got: {content}"

def test_file_to_markdown_out_dir_created_automatically(tmp_path):
    """out_dir が存在しなくても自動作成される。"""
    src     = tmp_path / "test.html"
    out_dir = tmp_path / "new" / "nested"
    src.write_text("<p>Hello</p>")

    result = html_file_to_markdown(str(src), str(out_dir))

    assert out_dir.exists(), "out_dir not created"
    assert (out_dir / "test.md").exists(), "output file not created"

def test_file_to_markdown_nonexistent_raises():
    """存在しないファイルを渡すと MdkaError が送出される。

    3.0: a single file fails the call through Python's own channel -- it raises --
    rather than returning a result object. (The bulk function reports per file.)
    """
    with pytest.raises(MdkaError, match=r"^IO error: "):
        html_file_to_markdown("/no/such/file.html")

def test_file_to_markdown_with_mode(tmp_path):
    """mode / drop_interactive_shell オプションが適用される。"""
    src = tmp_path / "spa.html"
    src.write_text("<nav>nav</nav><h1>Title</h1><p>Body</p>")

    result = html_file_to_markdown(
        str(src),
        mode=ConversionMode.Minimal,
        drop_interactive_shell=True,
    )

    content = (tmp_path / "spa.md").read_text()
    assert "# Title"                  in content, f"got: {content}"
    assert "nav" not in content.lower(),           f"nav leaked: {content}"

def test_file_to_markdown_returns_the_destination_as_str_and_nothing_else(tmp_path):
    src = tmp_path / "x.html"
    src.write_text("<p>x</p>")

    dest = html_file_to_markdown(str(src))

    assert type(dest) is str
    assert not hasattr(dest, "src") and not hasattr(dest, "ok")

def test_file_to_markdown_consistency_with_bulk(tmp_path):
    """html_file_to_markdown と html_files_to_markdown が同じ出力を生成する。"""
    src  = tmp_path / "test.html"
    out1 = tmp_path / "out1"
    out2 = tmp_path / "out2"
    out1.mkdir(); out2.mkdir()
    src.write_text("<h1>Hello</h1><p>World <strong>bold</strong></p>")

    r_single = html_file_to_markdown(str(src), str(out1))
    r_bulk   = html_files_to_markdown([str(src)], str(out2))

    c1 = (out1 / "test.md").read_text()
    c2 = (out2 / "test.md").read_text()
    assert c1 == c2, f"single vs bulk mismatch:\n{c1!r}\nvs\n{c2!r}"


# ─── html_file_to_markdown_with ────────────────────────────────────────────────

def test_file_with_matches_html_file_to_markdown_default(tmp_path):
    """RFC 039 §3 A1: html_file_to_markdown_with must exist and behave like
    the plain function under its default options."""
    src = tmp_path / "page.html"
    src.write_text("<h1>Single File</h1><p>Content</p>")
    out_a = tmp_path / "out_a"
    out_b = tmp_path / "out_b"

    d_with = html_file_to_markdown_with(str(src), str(out_a))
    d_plain = html_file_to_markdown(str(src), str(out_b))

    assert Path(d_with).read_text() == Path(d_plain).read_text()

def test_file_with_mode_option(tmp_path):
    src = tmp_path / "spa.html"
    src.write_text("<nav>nav</nav><h1>Title</h1><p>Body</p>")

    dest = html_file_to_markdown_with(
        str(src),
        mode=ConversionMode.Minimal,
        drop_interactive_shell=True,
    )

    content = Path(dest).read_text()
    assert "# Title" in content, f"got: {content}"
    assert "nav" not in content.lower(), f"nav leaked: {content}"

def test_file_with_nonexistent_raises():
    with pytest.raises(MdkaError):
        html_file_to_markdown_with("/no/such/file.html")


# ─── Type stubs (RFC 045) ────────────────────────────────────────────────────
#
# `python -m mypy.stubtest mdka` (run in CI) checks the stubs against this
# module and catches anything missing from the stub, or a signature that
# differs. It does NOT catch a class attribute the stub declares and the
# runtime lacks (verified: an extra `ConversionMode` member passes), so those
# are asserted here: every name a stub class declares as a variable must exist
# on the runtime class.

def test_stub_class_variables_exist_at_runtime():
    import ast

    stub = Path(mdka.__file__).parent / "mdka_python.pyi"
    assert stub.is_file(), f"the type stub is not shipped beside the module: {stub}"
    runtime = mdka.mdka_python
    declared = 0
    for node in ast.parse(stub.read_text()).body:
        if isinstance(node, ast.ClassDef):
            for stmt in node.body:
                if isinstance(stmt, ast.AnnAssign) and isinstance(stmt.target, ast.Name):
                    declared += 1
                    assert hasattr(getattr(runtime, node.name), stmt.target.id), (
                        f"stub declares {node.name}.{stmt.target.id}, which the runtime lacks"
                    )
    assert declared == 2, "expected the two ConversionMode members to be checked"
