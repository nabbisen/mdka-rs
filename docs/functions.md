# mdka: Functions on bindings

## Functions

### `from_html`

Convert from HTML text.

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_text |

#### Return

String

#### Error(s)

(None)

---

### `from_file`

Convert from HTML file.

#### Paramter(s)

| position | name / description |
| --- | --- |
| 1 | html_filepath |

#### Return

String

#### Error(s)

File I/O

---

### `from_html_to_file`

Convert from HTML text and write the result to file.

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

### `from_file_to_file`

Convert from HTML file and write the result to file.

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
