# mdka
**HTML to Markdown (MD)** converter written in [Rust](https://www.rust-lang.org/).

[![crates.io](https://img.shields.io/crates/v/mdka?label=latest)](https://crates.io/crates/mdka)
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka/latest)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka/latest)

## Summary

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

## Usage

### Executable

[Releases](https://github.com/nabbisen/mdka-rs/releases)' **Assets** offer executables for multiple platforms.

```
$ ./mdka <html-text> # not html file path
```

### Development with Rust and cargo

`Cargo.toml`

```toml
[dependencies]
mdka = "1"
```

`awesome.rs`

```rust
use mdka::from_html

fn awesome_fn() {
    let input = r#"
<h1>heading 1</h1>
<p>Hello, world.</p>"#;
    let ret = from_html(input);
    println!("{}", ret);
    // # heading 1
    // 
    // Hello, world.
    // 
}
```

### Python integration

As of 1.3.1, this library can be called from Python script.
[Releases](https://github.com/nabbisen/mdka-rs/releases)' **Assets** offer `.whl` for multiple platforms.

Install:

```console
$ pip install ./${RELEASE_ASSET_FOR_PLATFORM}.whl
```

`awesome.py`

```python
from mdka import md_from_html

print(md_from_html("<p>Hello, world.</p>"))
# Hello, world.
# 
```


## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
