# mdka
**HTML to Markdown** converter - Lightweight library written in Rust.

"ka" means "化 (か)" aka conversion.

[![crates.io](https://img.shields.io/crates/v/mdka?label=latest)](https://crates.io/crates/mdka)
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka/latest)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka/latest)

## Usage
`Cargo.toml`

```toml
[dependencies]
mdka = "^1.0.2"
```

`awesome.rs`

```rust
use mdka::from_html

fn awesome_fn() {
    let input = "<h1>heading 1</h1>\n<p>Hello, world.</p>";
    let ret = from_html(input);
    println!("{}", ret);
    // # heading 1
    // 
    // Hello, world.
    // 
    // 
}
```
