"""
mdka - a memory-efficient, fast HTML to Markdown converter, written in Rust
"""

from .mdka_python import (  # noqa: F401
    html_to_markdown,
    html_to_markdown_with,
    html_to_markdown_many,
    html_to_markdown_many_with,
    html_file_to_markdown,
    html_file_to_markdown_with,
    html_files_to_markdown,
    html_files_to_markdown_with,
    ConversionMode,
    FileOutcome,
    MdkaError,
    version,
)

__version__ = version()
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
