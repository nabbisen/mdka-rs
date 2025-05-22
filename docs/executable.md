# mdka: Executable

## Usage

Download the executable for your platform in [**Assets**](https://github.com/nabbisen/mdka-rs/releases/latest) in Releases. Then run:

```console
$ ./mdka <html-text>
converted-to-markdown-text will be printed
```

### Executable help

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