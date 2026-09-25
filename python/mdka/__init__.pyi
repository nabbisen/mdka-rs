"""Type stubs for the `mdka` package (RFC 045).

Mirrors the runtime `__init__.py`: the public names are the ones in `__all__`,
re-exported from the compiled extension, whose own stub is `mdka_python.pyi`.
Checked against the built module in CI (`python -m mypy.stubtest mdka`).
"""

from .mdka_python import (
    ConversionMode as ConversionMode,
    FileOutcome as FileOutcome,
    MdkaError as MdkaError,
    html_file_to_markdown as html_file_to_markdown,
    html_file_to_markdown_with as html_file_to_markdown_with,
    html_files_to_markdown as html_files_to_markdown,
    html_files_to_markdown_with as html_files_to_markdown_with,
    html_to_markdown as html_to_markdown,
    html_to_markdown_many as html_to_markdown_many,
    html_to_markdown_many_with as html_to_markdown_many_with,
    html_to_markdown_with as html_to_markdown_with,
    version as version,
)

__version__: str
__all__ = [
    "html_to_markdown",
    "html_to_markdown_with",
    "html_to_markdown_many",
    "html_to_markdown_many_with",
    "html_file_to_markdown",
    "html_file_to_markdown_with",
    "html_files_to_markdown",
    "html_files_to_markdown_with",
    "ConversionMode",
    "FileOutcome",
    "MdkaError",
    "version",
]
