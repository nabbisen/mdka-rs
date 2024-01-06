use mdka::from_html;

const TRIAL_HTML: &str = r#"
<section>
    <p>
        HTML string has been converted to Markdown (MD) !!
    </p>
</section>
"#;

/// available for trial run and the result will be printed out
/// 
/// ```
/// cargo run --example trial
/// ```
/// 
fn main() {
    let ret = from_html(TRIAL_HTML);
    println!("{}", ret);
}
