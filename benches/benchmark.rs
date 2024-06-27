///
/// usage:
///
/// ```
/// cargo bench
/// ```
///
/// will be generated: target/criterion/report/index.html
///
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mdka::from_html;

const BENCHMARK_NAME: &str = "mdka";
const BENCHMARK_INPUT: &str = r#"
<h1>Heading 1</h1>

<h2>ヘディング　２</h2>
<p>こんにちは、世界。</p>

<div>Lorem, ipsum, dolor.</div>

<ul>
    <li contenteditable=\"true\">lorem</li>
    <li contenteditable=\"true\">let see...
        <ul>
            <li contenteditable=\"true\">ipsum</li>
        </ul>
    <li contenteditable=\"true\">dolor</li>
</ul>

<ol>
    <li>١
    <li>٢
    <li>٣
</ol>

<table>
    <thead></thead>
        <tr>
            <th>٤</th>
            <th>٥</th>
            <th>٦</th>
        </tr>
    <tbody>
        <tr>
            <td>٧</td>
            <td>٨</td>
            <td>٩</td>
        </tr>
        <tr>
            <td>い</td>
            <td>ろ</td>
            <td>は</td>
        </tr>
    </tbody>
</table>

<pre><code lang="rust">
println!(\"Hello, world.\")
</code></pre>

<blockquote>
<b>Lorem</b>, <strong>ipsum</strong>, dolor.<br>
<i>Lorem</i>, <em>ipsum</em>, dolor.
</blockquote>

<hr>

<a href="/somewhere">something</a>

<img src="nice-image" alt="great-caption">
<audio src="nice-audio" alt="great-caption">
<video src="nice-video" alt="great-caption">
"#;

fn benchmarkee(c: &mut Criterion) {
    let input = BENCHMARK_INPUT.to_string().repeat(100);
    // disable compiler optimization
    let black_boxed = black_box(input.as_str());

    c.bench_function(BENCHMARK_NAME, |b| b.iter(|| from_html(black_boxed)));
}

criterion_group!(benches, benchmarkee);
criterion_main!(benches);
