# mdka: Bindings for Python

## Code examples

`awesome.py`

### Convert from HTML text

```python
from mdka import md_from_html

print(md_from_html("<p>Hello, world.</p>"))
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
from mdka import md_from_file

print(md_from_file("tests/fixtures/simple-01.html"))
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
from mdka import md_from_html_to_file

md_from_html_to_file("<p>Hello, world.</p>", "tests/tmp/out.md", False)
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
from mdka import md_from_file_to_file

md_from_file_to_file("tests/fixtures/simple-01.html", "tests/tmp/out.md", False)
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
