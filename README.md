# mdka

**HTML to Markdown (MD)** converter written in [Rust](https://www.rust-lang.org/).

[![crates.io](https://img.shields.io/crates/v/mdka?label=latest)](https://crates.io/crates/mdka)
[![Documentation](https://docs.rs/mdka/badge.svg?version=latest)](https://docs.rs/mdka)
[![Dependency Status](https://deps.rs/crate/mdka/latest/status.svg)](https://deps.rs/crate/mdka)
[![Executable](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-executable.yaml)
[![PyPi](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml/badge.svg)](https://github.com/nabbisen/mdka-rs/actions/workflows/release-pypi.yaml)
[![License](https://img.shields.io/github/license/nabbisen/mdka-rs)](https://github.com/nabbisen/mdka-rs/blob/main/LICENSE)

## Summary

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

## Usage

### Rust with cargo

```toml
# Cargo.toml
[dependencies]
mdka = "1"
```

```rust
// awesome.rs
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

For more details about functions, the docs live [here](docs/functions.md).

### Executable

[**Assets**](https://github.com/nabbisen/mdka-rs/releases/latest) in Releases offer executables for multiple platforms. → [For usage](docs/executable.md)

### Python integration

Bindings for Python are supported. → [For more examples](docs/BINDINGS_FOR_PYTHON.md)

```console
$ pip install mdka
```

```python
# awesome.py
from mdka import md_from_html

print(md_from_html("<p>Hello, world.</p>"))
# Hello, world.
# 
```

### Node.js integration

Bindings for Node.js are supported. → [For more examples](napi/README.md)

```console
$ npm install mdka
```

```js
// awesome.js
const { fromHtml } = require("mdka")

console.log(fromHtml("<p>Hello, world.</p>"))
// Hello, world.
// 
```

## 🤝 Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python. [napi-rs](https://github.com/napi-rs/napi-rs) for binding for [Node.js](https://nodejs.org/).
