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

Bindings for Python are supported. Python scripts can import this Rust library to use the function(s).

Install:

```console
$ pip install mdka
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
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python.
