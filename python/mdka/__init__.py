"""
mdka — 省メモリ・高速な HTML to Markdown コンバータ (Rust製)
"""

from .mdka_python import (  # noqa: F401
    html_to_markdown,
    html_to_markdown_with,
    html_to_markdown_many,
    html_file_to_markdown,
    html_files_to_markdown,
    ConversionMode,
    ConvertResult,
    BulkConvertResult,
    MdkaError,
    version,
)

__version__ = version()
__all__ = [
    "html_to_markdown",
    "html_to_markdown_with",
    "html_to_markdown_many",
    "html_file_to_markdown",
    "html_files_to_markdown",
    "ConversionMode",
    "ConvertResult",
    "BulkConvertResult",
    "MdkaError",
    "version",
]
