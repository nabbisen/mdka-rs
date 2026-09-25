"""Type stubs for the compiled extension module (RFC 045).

These describe what the module *is*, and are checked against the built
extension in CI (`python -m mypy.stubtest mdka`): a stub that drifts from the
runtime fails there. Write changes to the Rust source and to this file
together; do not rely on the documentation for the shape of a signature.
"""

from collections.abc import Sequence
from typing import ClassVar, final

# The extension declares its own `__all__` (PyO3 emits it), so the stub must too.
__all__ = [
    "MdkaError",
    "ConversionMode",
    "ConvertResult",
    "BulkConvertResult",
    "html_to_markdown",
    "html_to_markdown_with",
    "html_to_markdown_many",
    "html_to_markdown_many_with",
    "html_file_to_markdown",
    "html_file_to_markdown_with",
    "html_files_to_markdown",
    "html_files_to_markdown_with",
    "version",
]

@final
class ConversionMode:
    """How aggressively HTML is pre-processed: `Balanced` (the default) or
    `Minimal`. The former aliases `Strict`, `Semantic` and `Preserve` were
    removed in 3.0."""

    Balanced: ClassVar[ConversionMode]
    Minimal: ClassVar[ConversionMode]
    def __int__(self) -> int: ...

@final
class ConvertResult:
    """Result of a file conversion."""

    @property
    def src(self) -> str: ...
    @property
    def dest(self) -> str: ...

@final
class BulkConvertResult:
    """Result for one file in a bulk conversion, successful or not."""

    @property
    def src(self) -> str: ...
    @property
    def dest(self) -> str | None: ...
    @property
    def error(self) -> str | None: ...
    @property
    def ok(self) -> bool: ...

class MdkaError(Exception):
    """Raised when reading or writing a file fails."""

def version() -> str: ...
def html_to_markdown(html: str) -> str: ...

# The `_with` functions share one keyword tail: `mode`, `preserve_ids` and
# `drop_interactive_shell`. The keyword arguments removed in 3.0
# (`preserve_classes`, `preserve_data_attrs`, `preserve_aria_attrs`,
# `unwrap_unknown_wrappers`) now raise TypeError like any unknown keyword.
def html_to_markdown_with(
    html: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> str: ...
def html_to_markdown_many(html_list: Sequence[str]) -> list[str]: ...
def html_to_markdown_many_with(
    html_list: Sequence[str],
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> list[str]: ...
def html_file_to_markdown(
    path: str,
    out_dir: str | None = None,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> ConvertResult: ...
def html_file_to_markdown_with(
    path: str,
    out_dir: str | None = None,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> ConvertResult: ...
def html_files_to_markdown(
    paths: Sequence[str],
    out_dir: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> list[BulkConvertResult]: ...
def html_files_to_markdown_with(
    paths: Sequence[str],
    out_dir: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    drop_interactive_shell: bool | None = None,
) -> list[BulkConvertResult]: ...
