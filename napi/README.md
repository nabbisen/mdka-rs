# mdka

**HTML to Markdown (MD)** converter written in [Rust](https://www.rust-lang.org/).

## Summary

A kind of text manipulator named mdka. "ka" means "化 (か)" pointing to conversion.    
Designed with in mind:

- Fast speed
- Low memory consumption
- Easy usage

Bindings for Node.js are supported. Functions are available Node.js scripts can import. For more details about functions, check out [the docs](https://github.com/nabbisen/mdka-rs/blob/main/docs/functions.md).

## Install

```console
$ npm install mdka
```

---

## Usage

### Convert from HTML text

```js
const { fromHtml } = require("mdka")

console.log(fromHtml("<p>Hello, world.</p>"))
# Hello, world.
# 
```

### Convert from HTML file

```js
const { fromFile } = require("mdka")

console.log(fromFile("tests/fixtures/simple-01.html"))
# Hello, world.
# 
```

### Convert from HTML text and write the result to file

```js
const { fromHtmlToFile } = require("mdka")

fromHtmlToFile("<p>Hello, world.</p>", "tests/tmp/out.md", false)
```

### Convert from HTML file and write the result to file

```js
const { fromFileToFile } = require("mdka")

fromFileToFile("tests/fixtures/simple-01.html", "tests/tmp/out.md", false)
```

---

## 🤝 Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [Servo](https://servo.org/)'s [html5ever](https://github.com/servo/html5ever) / markup5ever.
Also, on [PyO3](https://github.com/PyO3)'s [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) on bindings for Python. [napi-rs](https://github.com/napi-rs/napi-rs) for binding for [Node.js](https://nodejs.org/).
