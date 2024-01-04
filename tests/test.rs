use mdka::from_html;

#[test]
fn heading() {
    let cases = vec![
        ("<h1>1</h1>", "# 1\n"),
        ("<h2>2</h2>", "## 2\n"),
        ("<h3>3</h3>", "### 3\n"),
        ("<h4>4</h4>", "#### 4\n"),
        ("<h5>5</h5>", "##### 5\n"),
        ("<h6>6</h6>", "###### 6\n"),
        ("<h1>1</h1>\n<h2>2</h2>\n<h3>3</h3>", "# 1\n## 2\n### 3\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn block() {
    let cases = vec![
        ("<span>1</span><span>2</span>", "12"),
        ("<div>1</div><div>2</div>", "1\n2\n"),
        ("<p>1</p><p>2</p>", "1\n\n2\n\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
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
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

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
"#, "| h1 | h2 |\n| --- | --- |\n| d1-1 | d1-2 |\n| d2-1 | d2-2 |\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn preformatted() {
    let cases = vec![
        ("<pre>1</pre>", "```\n1\n```\n"),
        ("<code>1</code>", "`1`"),
        ("<pre><code>1</code></pre>", "```\n1\n```\n"),
        ("<pre><code lang=\"rust\">1</code></pre>", "```rust\n1\n```\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

// todo
// #[test]
// fn blockquote() {
//     let cases = vec![
//         ("<b>1</b>", " **1** "),
//         ("<strong>2</strong>", " **2** "),
//     ];
//     for (input, expect) in cases {
//         let output = from_html(input);
//         assert_eq!(output, expect);
//     }
// }

// #[test]
// fn link() {
//     let cases = vec![
//         ("<b>1</b>", " **1** "),
//         ("<strong>2</strong>", " **2** "),
//     ];
//     for (input, expect) in cases {
//         let output = from_html(input);
//         assert_eq!(output, expect);
//     }
// }

// #[test]
// fn media() {
//     let cases = vec![
//         ("<b>1</b>", " **1** "),
//         ("<strong>2</strong>", " **2** "),
//     ];
//     for (input, expect) in cases {
//         let output = from_html(input);
//         assert_eq!(output, expect);
//     }
// }

#[test]
fn bold() {
    let cases = vec![
        ("<b>1</b>", " **1** "),
        ("<strong>2</strong>", " **2** "),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn italic() {
    let cases = vec![
        ("<i>1</i>", " *1* "),
        ("<em>2</em>", " *2* "),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn new_line() {
    let cases = vec![
        ("1<br>2", "1    \n2"),
        ("1<br><br>2", "1    \n    \n2"),
        ("1\n2", "1\n2"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn devider() {
    let cases = vec![
        ("<hr>", "\n---\n"),
        ("1<hr>2", "1\n---\n2"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn attrs() {
    let cases = vec![
        ("<h1 style=\"color: orange;\">1</h1>", "<span style=\"color: orange;\">\n# 1\n</span>\n"),
        ("<h1 id=\"myid\" style=\"color: orange;\">1</h1>", "<span id=\"myid\" style=\"color: orange;\">\n# 1\n</span>\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn style() {
    let cases = vec![
        ("<style>* { color: orange; }></style>", ""),
        ("<h1>1</h1><style>* { color: orange; }></style><h2>2</h2>", "# 1\n## 2\n"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}

#[test]
fn unsupported() {
    let cases = vec![
        ("<html>1</html>", "1"),
        ("<head>1</head>", "1"),
        ("<body>1</body>", "1"),
        ("<span><!-- 1 -->2</span>", "2"),
        ("<span class=\"b\">1</span>", "1"),
    ];
    for (input, expect) in cases {
        let output = from_html(input);
        assert_eq!(output, expect);
    }
}
