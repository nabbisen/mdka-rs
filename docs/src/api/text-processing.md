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

## Markdown Escaping

Text is escaped by the **context** it is written in, so that the Markdown
parses back — under CommonMark and GitHub Flavored Markdown — to exactly the
text the HTML contained, with as few backslashes as possible. No character is
escaped everywhere: a character gets a backslash only where, without one, it
would be read as Markdown.

### Text

In paragraphs, headings, list items, quotes and link text:

| Text | Escaped | Not escaped |
|---|---|---|
| `*` and `~` | where they could open or close emphasis or strikethrough: `*bold*` → `\*bold\*` | with a space on both sides: `2 * 3` |
| `_` | the same, except inside a word | `snake_case_here` |
| `` ` `` | always | |
| `[` | always | |
| `]` | inside link text and image alt text | elsewhere |
| `!` | before `[`, where it would start an image | `Wow! Really!` |
| `<` | before a letter, `/`, `!` or `?` — a tag, a comment or an autolink: `<div>` → `\<div>` | `a < b`, `<3` |
| `&` | before a letter, a digit or `#` — a possible entity: `&copy;` → `\&copy;` | `a & b` |
| `\` | before punctuation or at the end of a line | `path\to\file` |

A few characters need a backslash only at the **start of a block's text** —
after any `- `, `1. ` or `> ` that a list item or quote puts in front of it —
or at the start of a line after a `<br>`:

| Text begins with | Would become | Written as |
|---|---|---|
| a number, then `.` or `)` | an ordered list | `1986\. A great year` — the delimiter is escaped, not the digit |
| `-` or `+`, then a space | a bullet list | `\- not a list` |
| `#`, then a space | a heading | `\# not a heading` |
| `>` | a quote | `\> not a quote` |
| `~~~` | a code block | `\~\~\~` |
| `---` | a thematic break | `\---` |

After a `<br>`, a line of `=` or `-` would turn the paragraph into a heading,
and a line such as `| --- | --- |` under a line containing `|` would turn it
into a GFM table: its first character is escaped. In a heading, a trailing `#`
is escaped, since it would be read as the heading's closing sequence.

### Code

Code spans and code blocks hold their text **verbatim**, with no escapes. The
delimiter is sized to the content instead:

```html
<p><code>snake_case</code> and <code>a`b</code></p>
```

```
`snake_case` and ``a`b``
```

A code block's fence is one backtick longer than the longest run of backticks
that starts a line inside it, and never shorter than three. A language hint
containing a backtick is dropped, since a fence cannot hold one.

### Links and images

A destination with a space, a control character or unbalanced parentheses is
written in angle brackets; a line break in it is written as `&#10;`. A title is
written in `"…"`, or in `'…'` or `(…)` when that avoids escaping its quotes:

```html
<a href="/a b.html" title='say "hi"'>x</a>
```

```
[x](</a b.html> 'say "hi"')
```

### Adjacent emphasis

Two emphasized runs of the same kind that touch would merge into one run of
`*`, which CommonMark reads differently. One of them is written with `_`:

```html
<p><em>a</em><em>b</em></p>
```

```
_a_*b*
```

Between two letters or digits (`x<em>a</em><em>b</em>y`) neither `*` nor `_`
can express this, and the output is left as it is.

## HTML Entity Decoding

HTML entities in text nodes are decoded by the HTML parser (scraper / html5ever)
before mdka processes them. The result is already Unicode text:

| HTML entity | After parsing | In Markdown |
|---|---|---|
| `&amp;` | `&` | `&` (`\&` before a letter, a digit or `#`) |
| `&lt;` | `<` | `<` (`\<` before a letter, `/`, `!` or `?`) |
| `&gt;` | `>` | `>` |
| `&nbsp;` | non-breaking space | preserved as space |

## Output Boundaries

- Output always ends with **exactly one newline** (`\n`) when the input
  produces any content; the output is empty for empty input.
- Leading blank lines that scraper adds when wrapping content in `<html><body>`
  are trimmed before the final string is returned.
- Block elements (paragraphs, headings, lists, etc.) are separated by blank lines.
