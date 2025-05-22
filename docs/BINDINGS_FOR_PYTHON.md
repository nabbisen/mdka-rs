# mdka: Bindings for Python

## Summary

**HTML to Markdown (MD)** converter written in [Rust](https://www.rust-lang.org/).

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    

Bindings for Python are supported. Python scripts can import this Rust library to use the functions.

## Install

```console
$ pip install mdka
```

---

## Code examples

`awesome.py`

### Convert from HTML text

```python
from mdka import from_html

print(from_html("<p>Hello, world.</p>"))
# Hello, world.
# 
```

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_text |

#### Return

String

#### Error(s)

(None)

---

### Convert from HTML file

```python
from mdka import from_file

print(from_file("tests/fixtures/simple-01.html"))
# Hello, world.
# 
```

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_filepath |

#### Return

String

#### Error(s)

File I/O

---

### Convert from HTML text and write the result to file

```python
from mdka import from_html_to_file

from_html_to_file("<p>Hello, world.</p>", "tests/tmp/out.md", False)
```

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_text |
| 2 | markdown_filepath |
| 3 | overwrites : Overwrite if Markdown file exists. |

#### Return

(None)

#### Error(s)

File I/O

---

### Convert from HTML file and write the result to file

```python
from mdka import from_file_to_file

from_file_to_file("tests/fixtures/simple-01.html", "tests/tmp/out.md", False)
```

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_filepath |
| 2 | markdown_filepath |
| 3 | overwrites : Overwrite if Markdown file exists. |

---

#### Return

(None)

#### Error(s)

File I/O

---

## 🤝 Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python. [napi-rs](https://github.com/napi-rs/napi-rs) for binding for [Node.js](https://nodejs.org/).
