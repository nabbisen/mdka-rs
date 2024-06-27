use mdka::from_html;

const TRIAL_HTML: &str = r#"
<ul>
    <li contenteditable=\"true\">lorem</li>
    <li contenteditable=\"true\">let see...<ul>
        <li contenteditable=\"true\">ipsum</li>
    </ul>
    <li contenteditable=\"true\">dolor</li>
</ul>
"#;

/// available for trial run and the result will be printed out
///
/// usage:
///
/// ```
/// cargo run --example trial
/// ```
///
fn main() {
    let ret = from_html(TRIAL_HTML);
    println!("{}", ret);
}
