# Text Processing Rules

mdka applies a small set of deterministic rules to produce consistent,
readable Markdown from any HTML text content.

## Whitespace Normalisation

HTML text nodes are normalised according to the HTML whitespace collapsing rules:

- Leading and trailing whitespace is trimmed from block-level context.
- Consecutive whitespace characters (spaces, tabs, newlines) within a text
  node are collapsed to a single space.
- A single space is preserved between adjacent inline elements.
- `<br>` produces a hard line break (`  \n`).
- `<pre>` blocks are **exempt** — whitespace inside `<pre>` is reproduced exactly.

This is done in a single pass without regular expressions, which keeps
allocation overhead low.

## Markdown Character Escaping

To prevent accidental Markdown formatting, the following characters are
escaped with a backslash when they appear in text content that is not
inside a code span or code block:

| Character | Escaped as | Context |
|---|---|---|
| `*` | `\*` | Would create emphasis |
| `_` | `\_` | Would create emphasis |
| `` ` `` | `` \` `` | Would start a code span |
| `#` | `\#` | At the start of a line, would create a heading |
| `[` | `\[` | Would start a link |
| `!` | `\!` | Before `[`, would start an image |
| `\` | `\\` | The escape character itself |

Escaping is context-aware: a `#` in the middle of a line is **not** escaped,
only at the start of a line where it would be interpreted as an ATX heading.

## HTML Entity Decoding

HTML entities in text nodes are decoded by the HTML parser (scraper / html5ever)
before mdka processes them. The result is already Unicode text:

| HTML entity | After parsing | In Markdown |
|---|---|---|
| `&amp;` | `&` | `&` |
| `&lt;` | `<` | `<` |
| `&gt;` | `>` | `>` |
| `&nbsp;` | non-breaking space | preserved as space |

## Output Boundaries

- Output always ends with **exactly one newline** (`\n`) when the input
  produces any content; the output is empty for empty input.
- Leading blank lines that scraper adds when wrapping content in `<html><body>`
  are trimmed before the final string is returned.
- Block elements (paragraphs, headings, lists, etc.) are separated by blank lines.
