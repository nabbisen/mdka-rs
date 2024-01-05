# mdka
**HTML to Markdown** converter - Lightweight library written in Rust.

"ka" means "化" aka conversion.

[![crates.io](https://img.shields.io/crates/v/mdka?label=latest)](https://crates.io/crates/mdka)
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka/latest)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka/latest)

## Usage
`Cargo.toml`

```toml
[dependencies]
mdka = "*" # or specific version
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
