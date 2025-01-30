use mdka::from_html;

#[test]
fn heading() {
    let cases = vec![
        ("<h1>1</h1>", "# 1\n\n"),
        ("<h2>2</h2>", "## 2\n\n"),
        ("<h3>3</h3>", "### 3\n\n"),
        ("<h4>4</h4>", "#### 4\n\n"),
        ("<h5>5</h5>", "##### 5\n\n"),
        ("<h6>6</h6>", "###### 6\n\n"),
        (
            "<h1>1</h1>\n<h2>2</h2>\n<h3>3</h3>",
            "# 1\n\n## 2\n\n### 3\n\n",
        ),
        (
            "<h1>1</h1>\n\n\n<h2>2</h2>\n\n\n<h3>3</h3>",
            "# 1\n\n## 2\n\n### 3\n\n",
        ),
    ];
    assert(cases);
}

#[test]
fn block() {
    let cases = vec![
        ("<span>1</span><span>2</span>", "12"),
        ("<div>1</div><div>2</div>", "1\n2\n"),
        ("<p>1</p><p>2</p>", "1\n\n2\n\n"),
    ];
    assert(cases);
}

#[test]
fn bold() {
    let cases = vec![("<b>1</b>", " **1** "), ("<strong>2</strong>", " **2** ")];
    assert(cases);
}

#[test]
fn italic() {
    let cases = vec![("<i>1</i>", " _1_ "), ("<em>2</em>", " _2_ ")];
    assert(cases);
}

#[test]
fn bold_italic() {
    let cases = vec![
        ("<b>1<i>2</i></b>", " **1_2_** "),
        ("<b>1<em>2</em></b>", " **1_2_** "),
        ("<strong>1<i>2</i></strong>", " **1_2_** "),
        ("<strong>1<em>2</em></strong>", " **1_2_** "),
        ("<i>1<b>2</b></i>", " _1**2**_ "),
        ("<i>1<strong>2</strong></i>", " _1**2**_ "),
        ("<em>1<b>2</b></em>", " _1**2**_ "),
        ("<em>1<strong>2</strong></em>", " _1**2**_ "),
    ];
    assert(cases);
}

#[test]
fn list() {
    let cases = vec![
        ("<ul><li>1<li>2</ul>", "- 1\n- 2\n\n"),
        ("<ul><li>1</li><li>2</li></ul>", "- 1\n- 2\n\n"),
        ("<ol><li>1<li>2</ol>", "1. 1\n1. 2\n\n"),
        ("<ol><li>1</li><li>2</li></ol>", "1. 1\n1. 2\n\n"),
        (
            "<ul><li>1<ul><li>1-1<li>1-2</ul><li>2</ul>",
            "- 1\n    - 1-1\n    - 1-2\n- 2\n\n",
        ),
        (
            "<ul><li>1<ul><li>1-1<ul><li>1-1-1<li>1-1-2</ul><li>1-2</ul><li>2</ul>",
            "- 1\n    - 1-1\n        - 1-1-1\n        - 1-1-2\n    - 1-2\n- 2\n\n",
        ),
        (
            "<ul><li><ul><li><ul><li>1-1-1<li>1-1-2</ul><li>1-2</ul><li>2</ul>",
            "- \n    - \n        - 1-1-1\n        - 1-1-2\n    - 1-2\n- 2\n\n",
        ),
        (
            "<ul><li>1<ol><li>1-1<li>1-2</ol><li>2</ul>",
            "- 1\n    1. 1-1\n    1. 1-2\n- 2\n\n",
        ),
        (
            "<ol><li>1<ul><li>1-1<li>1-2</ul><li>2</ol>",
            "1. 1\n    - 1-1\n    - 1-2\n1. 2\n\n",
        ),
    ];
    assert(cases);
}

// more test: variety
#[test]
fn table() {
    let cases = vec![
        (r#"
<table>
    <thead>
        <tr>
            <th>h1</th>
            <th>h2</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>d1-1</td>
            <td>d1-2</td>
        </tr>
        <tr>
            <td>d2-1</td>
            <td>d2-2</td>
        </tr>
    </tbody>
</table>
"#, "| h1 | h2 |\n| --- | --- |\n| d1-1 | d1-2 |\n| d2-1 | d2-2 |\n\n\n"),
        (r#"
<table><thead><tr>
    <th style=\"text-align: left ;\">h1</th>
    <th style=\"text-align:center;\">h2</th>
    <th style=\"text-align :right;\">h3</th>
</thead><tbody><tr><td>d1</td><td>d2</td><td>d3</td></tr></tbody></table>"#,
        "| h1 | h2 | h3 |\n|:--- | --- | ---:|\n| d1 | d2 | d3 |\n\n"),
        (r#"
<table><thead><tr>
    <th class=\"text-left\">h1</th>
    <th class=\"text-center text-italic\">h2</th>
    <th class=\"text-bold text-right text-center\">h3</th>
</thead><tbody><tr><td>d1</td><td>d2</td><td>d3</td></tr></tbody></table>"#,
        "| h1 | h2 | h3 |\n|:--- | --- | ---:|\n| d1 | d2 | d3 |\n\n"),
        (r#"<table><thead><tr><th><img src="image1.png" alt="alt-text"></th><th>another column</th></tr></thead><tbody>
<tr><td><img src="image2.jpg"></td><td>first row</td></tr>
<tr><td><img src="image3.gif" alt="alt-text"></td><td>second row</td></tr>
</tbody></table>"#,
        "| ![alt-text](image1.png) | another column |\n| --- | --- |\n| ![](image2.jpg) | first row |\n| ![alt-text](image3.gif) | second row |\n\n")
    ];
    assert(cases);
}

#[test]
fn preformatted() {
    let cases = vec![
        ("<pre>1</pre>", "```\n1\n```\n\n"),
        ("<code>1</code>", " `1` "),
        ("<pre><code>1</code></pre>", "```\n1\n```\n\n"),
        ("<pre><code class=\"language-rust\">1</code></pre>", "```rust\n1\n```\n\n"),
        ("<pre><code class=\"language-rust some-class\">1</code></pre>", "```rust\n1\n```\n\n"),
        ("<pre><code class=\"some-class language-rust\">1</code></pre>", "```rust\n1\n```\n\n"),
        ("<pre><code class=\"some-class language-rust some-class\">1</code></pre>", "```rust\n1\n```\n\n"),
        ("<pre><code lang=\"rust\">1</code></pre>", "```rust\n1\n```\n\n"),
        ("<pre><div>1</div></pre>", "```\n<div>1</div>\n```\n\n"),
        ("<code><div>1</div></code>", " `<div>1</div>` "),
        ("<p>start <pre><div>a</div></pre> end</p>", "start\n\n```\n<div>a</div>\n```\n\nend"),
        ("<p>start <code><div>a</div></code> end</p>", "start\n\n `a` \nend"),
        (r#"
<ul>
    <li>a
        <ol>
            <li>
                <pre>
                <div>1</div>2

                </pre>
        </ol>
    <li>b
</ul>
        "#, "- a\n    1. \n        ```\n        <div>1</div>2\n        ```\n        \n        \n- b\n\n\n        "),
    ];
    assert(cases);
}

// more test: nested elements
#[test]
fn blockquote() {
    let cases = vec![
        (
            "<blockquote>a\nbc\ndef</blockquote>",
            "> a\n> bc\n> def\n\n",
        ),
        (
            "<blockquote>a<br>bc<br>def</blockquote>",
            "> a    \n> bc    \n> def\n\n",
        ),
        (
            "<blockquote>a\nbc<br>\ndef<hr></blockquote>",
            "> a\n> bc    \n> def\n> ---\n> \n\n",
        ),
        (
            r#"
<ul>
    <li>due to it:<br>
        <blockquote>lorem</blockquote>
    </li>
    <li>ipsum</li>
</ul>
"#,
            "- due to it:    \n    > lorem\n- ipsum\n\n\n",
        ),
    ];
    assert(cases);
}

#[test]
fn link() {
    let cases = vec![
        ("<a href=\"https://some-fqdn/some-dir/some-point\">Click me</a>", " [Click me](https://some-fqdn/some-dir/some-point) "),
        ("<a>no link</a>", " [no link]() "),
        ("<p>This is some link:<a href=\"somewhere1\">link1</a>and also:<a href=\"somewhere2\">link2</a>.</p>", "This is some link: [link1](somewhere1) and also: [link2](somewhere2) .\n\n"),
    ];
    assert(cases);
}

#[test]
fn media() {
    let cases = vec![
        (
            "<img src=\"/some-dir/some-file.ext\">",
            "![](/some-dir/some-file.ext)\n",
        ),
        ("<img alt=\"awesome image\">", "![awesome image]()\n"),
        (
            "<img src=\"/some-dir/some-file.ext\" alt=\"awesome image\">",
            "![awesome image](/some-dir/some-file.ext)\n",
        ),
        (
            "<img alt=\"awesome image\" src=\"/some-dir/some-file.ext1\">",
            "![awesome image](/some-dir/some-file.ext1)\n",
        ),
        (
            "<audio src=\"/some-dir/some-file.ext2\" alt=\"awesome audio\">",
            "![awesome audio](/some-dir/some-file.ext2)\n",
        ),
        (
            "<video src=\"/some-dir/some-file.ext3\" alt=\"awesome video\">",
            "![awesome video](/some-dir/some-file.ext3)\n",
        ),
    ];
    assert(cases);
}

#[test]
fn new_line() {
    let cases = vec![
        ("1<br>2", "1    \n2"),
        ("1<br><br>2", "1    \n    \n2"),
        ("1\n2", "1\n2"),
        (
            r#"1\n2
        "#,
            "1\\n2\n        ",
        ),
    ];
    assert(cases);
}

#[test]
fn divider() {
    let cases = vec![("<hr>", "\n---\n"), ("1<hr>2", "1\n---\n2")];
    assert(cases);
}

#[test]
fn document() {
    let cases = vec![
        ("<html>lorem</html>", "lorem"),
        ("<body>lorem</body>", "lorem"),
    ];
    assert(cases);
}

#[test]
fn semantic() {
    let cases = vec![
        ("<main>lorem</main>", "lorem"),
        ("<main><div>lorem</div></main>", "lorem\n"),
        ("<header>lorem</header>", "lorem"),
        ("<footer>lorem</footer>", "lorem"),
        ("<nav>lorem</nav>", "lorem"),
        (
            "<nav><a href=\"href_str\">caption</a></nav>",
            " [caption](href_str) ",
        ),
        ("<section>lorem</section>", "lorem"),
        ("<section><p>lorem</p></section>", "lorem\n\n"),
        ("<article>lorem</article>", "lorem"),
        ("<aside>lorem</aside>", "lorem"),
        ("<time>lorem</time>", "lorem"),
        ("<address>lorem</address>", "lorem"),
        ("<figure><img src=\"src_str\"></figure>", "![](src_str)\n"),
        ("<figcaption>lorem</figcaption>", "lorem"),
        (
            "<figure><img src=\"src_str\"><figcaption>lorem</figcaption></figure>",
            "![](src_str)\nlorem",
        ),
    ];
    assert(cases);
}

#[test]
fn attrs() {
    let cases = vec![
        (
            "<h1 style=\"color: orange;\">1</h1>",
            "\n<span style=\"color: orange;\">\n# 1\n\n</span>\n",
        ),
        (
            "<h1 id=\"myid\" style=\"color: orange;\">1</h1>",
            "\n<span id=\"myid\"></span>\n<span style=\"color: orange;\">\n# 1\n\n</span>\n",
        ),
    ];
    assert(cases);
}

#[test]
fn empty_element() {
    let cases = vec![
        ("<h1></h1>", ""),
        ("<h2></h2>", ""),
        ("<h3></h3>", ""),
        ("<h4></h4>", ""),
        ("<h5></h5>", ""),
        ("<h6></h6>", ""),
        ("<div></div>", ""),
        ("<p></p>", ""),
        ("<span></span>", ""),
        ("<b></b>", ""),
        ("<strong></strong>", ""),
        ("<i></i>", ""),
        ("<i class=\"some-icon\"></i>", ""),
        ("<em></em>", ""),
        ("<ul></ul>", ""),
        ("<ol></ol>", ""),
        ("<ul><li></ul>", "- \n\n"),
        ("<table></table>", ""),
        (
            "<table><tbody><tr><td></td></tr></tbody></table>",
            "|  |\n| --- |\n\n",
        ),
        (
            "<table><thead><tr><th></th></tr></thead><tbody><tr><td></td></tr></tbody></table>",
            "|  |\n| --- |\n|  |\n\n",
        ),
        ("<code></code>", ""),
        ("<pre></pre>", ""),
        ("<pre><code></code></pre>", ""),
        ("<blockquote></blockquote>", ""),
        ("<a></a>", ""),
        ("<a href></a>", ""),
        ("<a href=\"\"></a>", ""),
        ("<a href=\"href_str\"></a>", " [](href_str) "),
        ("<a>caption</a>", " [caption]() "),
        ("<img></img>", ""),
        ("<img src></img>", ""),
        ("<img src=\"\"></img>", ""),
        ("<img alt></img>", ""),
        ("<img alt=\"\"></img>", ""),
        ("<img src=\"src_str\">", "![](src_str)\n"),
        ("<img id=\"myid\"></img>", "<span id=\"myid\"></span>"),
        ("<audio></audio>", ""),
        ("<audio src=\"src_str\">", "![](src_str)\n"),
        ("<audio alt=\"alt_str\">", "![alt_str]()\n"),
        ("<video></video>", ""),
        ("<video src=\"src_str\">", "![](src_str)\n"),
        ("<video alt=\"alt_str\">", "![alt_str]()\n"),
    ];
    assert(cases);
}

#[test]
fn unicode() {
    let cases = vec![
        ("<div>あいうえお</div>", "あいうえお\n"),
        ("<div>春夏秋冬</div>", "春夏秋冬\n"),
        (
            r#"
<p>文字のテスト
    <ul>
        <li>გამარჯობა
        <li>السلام عليكم
        <li>وعلیکم السلام
    </ul>
    <table>
        <tr>
            <td>Název knihy</td>
        </tr>
        <tr>
            <td>Blod på snø</td>
        </tr>
    </table>
</p>
        "#,
            r#"文字のテスト

- გამარჯობა
- السلام عليكم
- وعلیکم السلام

| Název knihy |
| --- |
| Blod på snø |


        "#,
        ),
    ];
    assert(cases);
}

#[test]
fn empty_document() {
    let cases = vec![("<html></html>", ""), ("<body></body>", "")];
    assert(cases);
}

#[test]
fn empty_semantic() {
    let cases = vec![
        ("<main></main>", ""),
        ("<header></header>", ""),
        ("<footer></footer>", ""),
        ("<nav></nav>", ""),
        ("<section></section>", ""),
        ("<article></article>", ""),
        ("<aside></aside>", ""),
        ("<time></time>", ""),
        ("<address></address>", ""),
        ("<figure></figure>", ""),
        ("<figcaption></figcaption>", ""),
    ];
    assert(cases);
}

#[test]
fn empty_enclosed() {
    let cases = vec![
        ("<h1 id=\"myid\"></h1>", "<span id=\"myid\"></span>"),
        (
            "<h1 style=\"color: pink;\"></h1>",
            "<span style=\"color: pink;\"></span>",
        ),
        ("<div id=\"myid\"></div>", "<span id=\"myid\"></span>"),
        ("<span id=\"myid\"></span>", "<span id=\"myid\"></span>"),
        ("<ul id=\"myid\"></ul>", "<span id=\"myid\"></span>"),
        ("<table id=\"myid\"></table>", "<span id=\"myid\"></span>"),
        ("<code id=\"myid\"></code>", "<span id=\"myid\"></span>"),
        ("<pre id=\"myid\"></pre>", "<span id=\"myid\"></span>"),
        (
            "<blockquote id=\"myid\"></blockquote>",
            "<span id=\"myid\"></span>",
        ),
        ("<a id=\"myid\"></a>", "<span id=\"myid\"></span>"),
        ("<img id=\"myid\"></img>", "<span id=\"myid\"></span>"),
        ("<audio id=\"myid\"></audio>", "<span id=\"myid\"></span>"),
        ("<video id=\"myid\"></video>", "<span id=\"myid\"></span>"),
    ];
    assert(cases);
}

#[test]
fn contenteditable_element() {
    let cases = vec![
        ("<h1 contenteditable=\"true\">lorem</h1>", "# lorem\n\n"),
        ("<h2 contenteditable=\"true\">lorem</h2>", "## lorem\n\n"),
        ("<h3 contenteditable=\"true\">lorem</h3>", "### lorem\n\n"),
        ("<h4 contenteditable=\"true\">lorem</h4>", "#### lorem\n\n"),
        ("<h5 contenteditable=\"true\">lorem</h5>", "##### lorem\n\n"),
        (
            "<h6 contenteditable=\"true\">lorem</h6>",
            "###### lorem\n\n",
        ),
        ("<div contenteditable=\"true\">lorem</div>", "lorem\n"),
        ("<p contenteditable=\"true\">lorem</p>", "lorem\n\n"),
        ("<span contenteditable=\"true\">lorem</span>", "lorem"),
        ("<b contenteditable=\"true\">lorem</b>", " **lorem** "),
        (
            "<strong contenteditable=\"true\">lorem</strong>",
            " **lorem** ",
        ),
        ("<i contenteditable=\"true\">lorem</i>", " _lorem_ "),
        ("<em contenteditable=\"true\">lorem</i>", " _lorem_ "),
        (
            "<ul><li contenteditable=\"true\">lorem</li></ul>",
            "- lorem\n\n",
        ),
        (
            r#"
<ul>
    <li contenteditable=\"true\">lorem</li>
    <li contenteditable=\"true\"><ul>
        <li contenteditable=\"true\">ipsum</li>
    </ul>
    <li contenteditable=\"true\">dolor</li>
</ul>
        "#,
            "- lorem\n- \n    - ipsum\n- dolor\n\n\n        ",
        ),
        (
            "<ol><li contenteditable=\"true\">lorem</li></ol>",
            "1. lorem\n\n",
        ),
        (
            r#"
<ol>
    <li contenteditable=\"true\">lorem</li>
    <li contenteditable=\"true\"><ol>
        <li contenteditable=\"true\">ipsum</li>
    </ol>
    <li contenteditable=\"true\">dolor</li>
</ol>
        "#,
            "1. lorem\n1. \n    1. ipsum\n1. dolor\n\n\n        ",
        ),
        (
            r#"
<ol>
    <li contenteditable=\"true\">lorem-1</li>
    <li contenteditable=\"true\">lorem-2<ul>
        <li contenteditable=\"true\">ipsum-1</li>
        <li contenteditable=\"true\">ipsum-2</li>
        <li contenteditable=\"true\">ipsum-3<ol>
            <li contenteditable=\"true\">dolor-1</li>
            <li contenteditable=\"true\">dolor-2</li>
            <li contenteditable=\"true\">dolor-3</li>
        </li></ol>
        <li contenteditable=\"true\">ipsum-4</li>
    </ul>
    <li contenteditable=\"true\">lorem-3</li>
    <li contenteditable=\"true\">lorem-4</li>
</ol>
        "#,
            r#"1. lorem-1
1. lorem-2
    - ipsum-1
    - ipsum-2
    - ipsum-3
        1. dolor-1
        1. dolor-2
        1. dolor-3
    - ipsum-4
1. lorem-3
1. lorem-4


        "#,
        ),
        (
            r#"
<table>
    <thead>
        <tr>
            <th contenteditable=\"true\">h1</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td contenteditable=\"true\">d1</td>
        </tr>
    </tbody>
</table>
        "#,
            "| h1 |\n| --- |\n| d1 |\n\n\n        ",
        ),
        (
            r#"
<table contenteditable=\"true\">
    <thead contenteditable=\"true\">
        <tr contenteditable=\"true\">
            <th contenteditable=\"true\">h1</th>
            <th contenteditable=\"true\">h2</th>
        </tr>
    </thead>
    <tbody contenteditable=\"true\">
        <tr contenteditable=\"true\">
            <td contenteditable=\"true\">d1</td>
            <td contenteditable=\"true\">d2</td>
        </tr>
        <tr contenteditable=\"true\">
            <td contenteditable=\"true\">d3</td>
            <td contenteditable=\"true\">d4</td>
        </tr>
    </tbody>
</table>
        "#,
            "| h1 | h2 |\n| --- | --- |\n| d1 | d2 |\n| d3 | d4 |\n\n\n        ",
        ),
        ("<code contenteditable=\"true\">lorem</code>", " `lorem` "),
        (
            "<pre contenteditable=\"true\">lorem</pre>",
            "```\nlorem\n```\n\n",
        ),
        (
            "<pre contenteditable=\"true\"><code lang=\"rust\">println!(\"lorem\");</code></pre>",
            "```rust\nprintln!(\"lorem\");\n```\n\n",
        ),
        (
            "<blockquote contenteditable=\"true\">lorem</blockquote>",
            "> lorem\n\n",
        ),
        (
            "<a href=\"href_str\" contenteditable=\"true\">caption</a>",
            " [caption](href_str) ",
        ),
    ];
    assert(cases);
}

#[test]
fn unsupported() {
    let cases = vec![
        (
            "<!doctype html><html lang=\"en\"><head>1</head></html>",
            "1",
        ), // treated as inline
        ("<script>1</script>", ""),
        ("<script lang=\"ts\">console.log('wow')</script>", ""),
        ("<style>* { color: orange; }></style>", ""),
        (
            "<h1>1</h1><style>* { color: orange; }></style><h2>2</h2>",
            "# 1\n\n## 2\n\n",
        ),
        ("<span><!-- 1 -->2</span>", "2"),
        ("<span class=\"b\">1</span>", "1"),
        (
            r#"
<svg width="100" height="100">
  <circle cx="50" cy="50" r="40" stroke="green" stroke-width="4" fill="yellow" />
</svg>
        "#,
            "\n        ",
        ),
        ("<customtag></customtag>", ""),
        ("<div customattr=\"some\">lorem</div>", "lorem\n"),
        ("<div data-id=\"some\">lorem</div>", "lorem\n"),
        ("<CustomComponent id=\"myid\" style=\"mystyle\" />", ""),
        ("<CustomComponent :id=\"myid\" :style=\"mystyle\" />", ""),
    ];
    assert(cases);
}

#[test]
fn readme_usage() {
    let cases = vec![(
        "<h1>heading 1</h1>\n<p>Hello, world.</p>",
        "# heading 1\n\nHello, world.\n\n",
    )];
    assert(cases);
}

/// general purpose assertion
fn assert(cases: Vec<(&str, &str)>) {
    for (i, (input, expect)) in cases.into_iter().enumerate() {
        let output = from_html(input);
        assert_eq!(output, expect, "\ncase #{}", i + 1);
    }
}
