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
        ("<h1>1</h1>\n<h2>2</h2>\n<h3>3</h3>", "# 1\n\n## 2\n\n### 3\n\n"),
        ("<h1>1</h1>\n\n\n<h2>2</h2>\n\n\n<h3>3</h3>", "# 1\n\n## 2\n\n### 3\n\n"),
    ];
    assert(cases);
}

#[test]
fn block() {
    let cases = vec![
        ("<span>1</span><span>2</span>", "12"),
        ("<div>1</div><div>2</div>", "1\n2\n"),
        ("<p>1</p><p>2</p>", "\n1\n\n\n2\n\n"),
    ];
    assert(cases);
}

#[test]
fn list() {
    let cases = vec![
        ("<ul><li>1<li>2</ul>", "- 1\n- 2\n"),
        ("<ul><li>1</li><li>2</li></ul>", "- 1\n- 2\n"),
        ("<ol><li>1<li>2</ol>", "1. 1\n1. 2\n"),
        ("<ol><li>1</li><li>2</li></ol>", "1. 1\n1. 2\n"),
        ("<ul><li>1<ul><li>1-1<li>1-2</ul><li>2</ul>", "- 1\n    - 1-1\n    - 1-2\n- 2\n"),
        ("<ul><li>1<ul><li>1-1<ul><li>1-1-1<li>1-1-2</ul><li>1-2</ul><li>2</ul>", "- 1\n    - 1-1\n        - 1-1-1\n        - 1-1-2\n    - 1-2\n- 2\n"),
        ("<ul><li><ul><li><ul><li>1-1-1<li>1-1-2</ul><li>1-2</ul><li>2</ul>", "- \n    - \n        - 1-1-1\n        - 1-1-2\n    - 1-2\n- 2\n"),
        ("<ul><li>1<ol><li>1-1<li>1-2</ol><li>2</ul>", "- 1\n    1. 1-1\n    1. 1-2\n- 2\n"),
        ("<ol><li>1<ul><li>1-1<li>1-2</ul><li>2</ol>", "1. 1\n    - 1-1\n    - 1-2\n1. 2\n"),
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
"#, "\n| h1 | h2 |\n| --- | --- |\n| d1-1 | d1-2 |\n| d2-1 | d2-2 |\n\n"),
    ];
    assert(cases);
}

#[test]
fn preformatted() {
    let cases = vec![
        ("<pre>1</pre>", "\n```\n1\n```\n\n"),
        ("<code>1</code>", "`1`"),
        ("<pre><code>1</code></pre>", "\n```\n1\n```\n\n"),
        ("<pre><code lang=\"rust\">1</code></pre>", "\n```rust\n1\n```\n\n"),
        ("<pre><div>1</div></pre>", "\n```\n<div>1</div>\n```\n\n"),
        ("<code><div>1</div></code>", "`<div>1</div>`"),
        ("<ul><li>a<ol><li><pre><div>1</div>2\n</pre></ol><li>b</ul>", "- a\n    1. \n        ```\n        <div>1</div>2        \n        ```\n        \n        \n- b\n"),
    ];
    assert(cases);
}

// more test: nested elements
#[test]
fn blockquote() {
    let cases = vec![
        ("<blockquote>a\nbc\ndef</blockquote>", "> a\n> bc\n> def"),
        ("<blockquote>a\nbc<br>\ndef<hr></blockquote>", "> a\n> bc    \n> def\n> ---\n> "),
    ];
    assert(cases);
}

#[test]
fn link() {
    let cases = vec![
        ("<a href=\"https://some-fqdn/some-dir/some-point\">Click me</a>", "[Click me](https://some-fqdn/some-dir/some-point)"),
        ("<a>no link</a>", "[no link]()"),
    ];
    assert(cases);
}

#[test]
fn media() {
    let cases = vec![
        ("<img src=\"/some-dir/some-file.ext\">", "\n![](/some-dir/some-file.ext)\n"),
        ("<img alt=\"awesome image\">", "\n![awesome image]()\n"),
        ("<img src=\"/some-dir/some-file.ext\" alt=\"awesome image\">", "\n![awesome image](/some-dir/some-file.ext)\n"),
        ("<img alt=\"awesome image\" src=\"/some-dir/some-file.ext\">", "\n![awesome image](/some-dir/some-file.ext)\n"),
        ("<video src=\"/some-dir/some-file.ext2\" alt=\"awesome video\">", "\n![awesome video](/some-dir/some-file.ext2)\n"),
    ];
    assert(cases);
}

#[test]
fn bold() {
    let cases = vec![
        ("<b>1</b>", " **1** "),
        ("<strong>2</strong>", " **2** "),
    ];
    assert(cases);
}

#[test]
fn italic() {
    let cases = vec![
        ("<i>1</i>", " *1* "),
        ("<em>2</em>", " *2* "),
    ];
    assert(cases);
}

#[test]
fn new_line() {
    let cases = vec![
        ("1<br>2", "1    \n2"),
        ("1<br><br>2", "1    \n    \n2"),
        ("1\n2", "1\n2"),
    ];
    assert(cases);
}

#[test]
fn devider() {
    let cases = vec![
        ("<hr>", "\n---\n"),
        ("1<hr>2", "1\n---\n2"),
    ];
    assert(cases);
}

#[test]
fn text() {
    let cases = vec![
        ("<html>1</html>", "1"),
        ("<body>1</body>", "1"),
    ];
    assert(cases);
}

#[test]
fn attrs() {
    let cases = vec![
        ("<h1 style=\"color: orange;\">1</h1>", "\n<span style=\"color: orange;\">\n# 1\n\n</span>\n"),
        ("<h1 id=\"myid\" style=\"color: orange;\">1</h1>", "\n<span id=\"myid\" style=\"color: orange;\">\n# 1\n\n</span>\n"),
    ];
    assert(cases);
}

#[test]
fn unsupported() {
    let cases = vec![
        ("<!doctype html><html lang=\"en\"><head>1</head></html>", "1"), // treated as inline
        ("<script>1</script>", ""),
        ("<script lang=\"ts\">console.log('wow')</script>", ""),
        ("<style>* { color: orange; }></style>", ""),
        ("<h1>1</h1><style>* { color: orange; }></style><h2>2</h2>", "# 1\n\n## 2\n\n"),
        ("<span><!-- 1 -->2</span>", "2"),
        ("<span class=\"b\">1</span>", "1"),
    ];
    assert(cases);
}

#[test]
fn readme_usage() {
    let cases = vec![
        ("<h1>heading 1</h1>\n<p>Hello, world.</p>", "# heading 1\n\n\nHello, world.\n\n"),
    ];
    assert(cases);
}

fn assert(cases: Vec<(&str, &str)>) {
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}