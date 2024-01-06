use mdka::from_html;

const TRIAL_HTML: &str = r#"
<section>
    <p>
        HTML string has been converted to Markdown (MD) !!
    </p>
</section>
"#;

/// Available for trial run and the result will be printed out.
/// This file is untracked so locally editable.
///
/// How to use:
/// 
/// ```
/// cargo run --example trial
/// ```
/// 
fn main() {
    let ret = from_html(TRIAL_HTML);
    println!("{}", ret);
}
