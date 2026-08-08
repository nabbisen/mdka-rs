//! Integration tests: RFC 017 — `<pre>` closing fence newline reset regression
//! coverage
//! Covers: the trailing-newline severity range from the RFC 017 handoff §6,
//! guarding against the state-desync defect where the closing fence swallowed
//! the newline before following content (fixed by switching the "pre" arm in
//! `leave_element` from `output.push_str` to `push_raw`).

mod common;
use common::conv;

#[test]
fn pre_no_trailing_newline() {
    assert_eq!(
        conv("<pre><code>fn main(){}</code></pre><p>B</p>"),
        "```\nfn main(){}\n```\n\nB\n"
    );
}

#[test]
fn pre_one_trailing_newline() {
    assert_eq!(
        conv("<pre><code>fn main(){}\n</code></pre><p>B</p>"),
        "```\nfn main(){}\n```\n\nB\n"
    );
}

#[test]
fn pre_two_trailing_newlines() {
    assert_eq!(
        conv("<pre><code>x\n\n</code></pre><p>B</p>"),
        "```\nx\n\n```\n\nB\n"
    );
}

#[test]
fn pre_then_hr() {
    assert_eq!(
        conv("<pre><code>x\n</code></pre><hr><p>B</p>"),
        "```\nx\n```\n\n---\n\nB\n"
    );
}

#[test]
fn pre_alone() {
    assert_eq!(conv("<pre><code>x</code></pre>"), "```\nx\n```\n");
}

#[test]
fn pre_language_class() {
    assert_eq!(
        conv("<pre><code class=\"language-rust\">fn f(){}\n</code></pre><p>B</p>"),
        "```rust\nfn f(){}\n```\n\nB\n"
    );
}
