# mdka: Bindings for Python

**HTML to Markdown (MD)** converter written in [Rust](https://www.rust-lang.org/).

## Summary

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

Bindings for Python are supported. Functions are available Python scripts can import. For more details about functions, check out [the docs](https://github.com/nabbisen/mdka-rs/blob/main/docs/functions.md).

## Install

```console
$ pip install mdka
```

---

## Usage

### Convert from HTML text

```python
from mdka import from_html

print(from_html("<p>Hello, world.</p>"))
# Hello, world.
# 
```

### Convert from HTML file

```python
from mdka import from_file

print(from_file("tests/fixtures/simple-01.html"))
# Hello, world.
# 
```

### Convert from HTML text and write the result to file

```python
from mdka import from_html_to_file

from_html_to_file("<p>Hello, world.</p>", "tests/tmp/out.md", False)
```

### Convert from HTML file and write the result to file

```python
from mdka import from_file_to_file

from_file_to_file("tests/fixtures/simple-01.html", "tests/tmp/out.md", False)
```

---

## 🤝 Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python. [napi-rs](https://github.com/napi-rs/napi-rs) for binding for [Node.js](https://nodejs.org/).
