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

```console
$ ./mdka <html-text>
converted-to-markdown-text will be printed
```

#### Help

```console
$ ./mdka -h
Usage:
  -h, --help             : Help is shown.
  <html_text>            : Direct parameter is taken as HTML text to be converted. Either this or <html_filepath> is required.
  -i <html_filepath>     : Read HTML text from it. Optional.
  -o <markdown_filepath> : Write Markdown result to it. Optional.
  --overwrites           : Overwrite if Markdown file exists. Optional.

Examples:
  ./mdka "<p>Hello, world.</p>"
  ./mdka -i input.html
  ./mdka -o output.md "<p>Hello, world.</p>"
  ./mdka -i input.html -o output.md --overwrites
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

Binding for Python is supported. Python scripts can import this Rust library to use the functions.

Install:

```console
$ pip install mdka
```

`awesome.py`

#### Convert from HTML text

```python
from mdka import md_from_html

print(md_from_html("<p>Hello, world.</p>"))
# Hello, world.
# 
```

##### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_text |

##### Return

String

##### Error(s)

(None)

---

#### Convert from HTML file

```python
from mdka import md_from_file

print(md_from_file("tests/fixtures/simple-01.html"))
# Hello, world.
# 
```

##### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_filepath |

##### Return

String

##### Error(s)

File I/O

---

#### Convert from HTML text and write the result to file

```python
from mdka import md_from_html_to_file

md_from_html_to_file("<p>Hello, world.</p>", "tests/tmp/out.md", False)
```

##### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_text |
| 2 | markdown_filepath |
| 3 | overwrites : Overwrite if Markdown file exists. |

##### Return

(None)

##### Error(s)

File I/O

---

#### Convert from HTML file and write the result to file

```python
from mdka import md_from_file_to_file

md_from_file_to_file("tests/fixtures/simple-01.html", "tests/tmp/out.md", False)
```

##### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_filepath |
| 2 | markdown_filepath |
| 3 | overwrites : Overwrite if Markdown file exists. |

---

##### Return

(None)

##### Error(s)

File I/O

## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python.
