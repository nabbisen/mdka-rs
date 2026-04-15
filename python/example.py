"""
mdka Python バインディング — 使用例

実行方法:
    cd python/
    python example.py
"""
import tempfile
from pathlib import Path
import mdka

print(f"mdka v{mdka.__version__}\n")

# ── 1. 同期変換 ────────────────────────────────────────────────────────────
html = """
<h1>mdka デモ</h1>
<p>Rust 製の高速 HTML → Markdown コンバータです。</p>
<h2>特徴</h2>
<ul>
  <li><strong>省メモリ</strong>: html2md 比 40% 削減</li>
  <li><strong>高速</strong>: 非再帰 DFS + シングルパス正規化</li>
  <li><strong>堅牢</strong>: 10,000 段ネストでもクラッシュしない</li>
</ul>
<blockquote>
  <blockquote><p>O(n) スケーリング確認済み</p></blockquote>
</blockquote>
<pre><code class="language-python">md = mdka.html_to_markdown(html)</code></pre>
"""
print("=== html_to_markdown ===")
print(mdka.html_to_markdown(html))

# ── 2. 並列一括変換 ─────────────────────────────────────────────────────────
print("=== html_to_markdown_many (rayon 並列, GIL 解放) ===")
items = [
    "<h1>タイトル</h1>",
    "<p>段落 <em>強調</em> と <strong>太字</strong></p>",
    '<ol start="3"><li>Three</li><li>Four</li></ol>',
    "<blockquote><blockquote><p>ネスト引用</p></blockquote></blockquote>",
]
for i, md in enumerate(mdka.html_to_markdown_many(items)):
    print(f"  [{i}] {md.strip()}")

# ── 3. ファイル並列変換 ──────────────────────────────────────────────────────
print("\n=== html_files_to_markdown ===")
with tempfile.TemporaryDirectory() as tmp:
    names = ["index", "about", "docs"]
    for name in names:
        Path(tmp, f"{name}.html").write_text(
            f"<h1>{name}</h1><p>{name} ページのコンテンツ。</p>"
        )

    results = mdka.html_files_to_markdown(
        [str(Path(tmp, f"{n}.html")) for n in names],
        str(Path(tmp, "out")),
    )
    for r in results:
        status = f"✅ {Path(r.dest).name}" if r.ok else f"❌ {r.error}"
        print(f"  {Path(r.src).name} → {status}")

# ── 4. 例外処理 ─────────────────────────────────────────────────────────────
print("\n=== MdkaError ===")
try:
    mdka.html_files_to_markdown([], "/dev/null/impossible")
except mdka.MdkaError as e:
    print(f"  MdkaError: {e}")
