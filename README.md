# mdka
**HTML to Markdown (MD)** converter written in Rust.

[![crates.io](https://img.shields.io/crates/v/mdka?label=latest)](https://crates.io/crates/mdka)
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka/latest)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka/latest)

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

## Usage
`Cargo.toml`

```toml
[dependencies]
mdka = "1.2"
```

`awesome.rs`

```rust
use mdka::from_html

fn awesome_fn() {
    let input = r#"
<h1>heading 1</h1>
<p>Hello, world.</p>
"#;
    let ret = from_html(input);
    println!("{}", ret);
    // # heading 1
    // 
    // Hello, world.
    // 
}
```
