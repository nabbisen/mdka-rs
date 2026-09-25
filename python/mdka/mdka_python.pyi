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
    """How aggressively HTML is pre-processed. Only `Balanced` and `Minimal`
    convert differently. `Strict`, `Semantic` and `Preserve` are aliases of
    `Balanced`: deprecated since 2.8.0 and removed in 3.0. Naming one emits a
    `DeprecationWarning`; the output is unchanged."""

    Balanced: ClassVar[ConversionMode]
    Strict: ClassVar[ConversionMode]
    Minimal: ClassVar[ConversionMode]
    Semantic: ClassVar[ConversionMode]
    Preserve: ClassVar[ConversionMode]
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

# The `_with` functions share one keyword tail. `preserve_classes`,
# `preserve_data_attrs`, `preserve_aria_attrs` and (since 2.9.0)
# `unwrap_unknown_wrappers` are deprecated, have no effect and emit a
# DeprecationWarning when passed, but are still accepted, so they are part of
# the signature. There is deliberately no
# `preserve_unknown_attrs`: passing it raises TypeError.
def html_to_markdown_with(
    html: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> str: ...
def html_to_markdown_many(html_list: Sequence[str]) -> list[str]: ...
def html_to_markdown_many_with(
    html_list: Sequence[str],
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> list[str]: ...
def html_file_to_markdown(
    path: str,
    out_dir: str | None = None,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> ConvertResult: ...
def html_file_to_markdown_with(
    path: str,
    out_dir: str | None = None,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> ConvertResult: ...
def html_files_to_markdown(
    paths: Sequence[str],
    out_dir: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> list[BulkConvertResult]: ...
def html_files_to_markdown_with(
    paths: Sequence[str],
    out_dir: str,
    mode: ConversionMode = ...,
    preserve_ids: bool | None = None,
    preserve_classes: bool | None = None,
    preserve_data_attrs: bool | None = None,
    preserve_aria_attrs: bool | None = None,
    drop_interactive_shell: bool | None = None,
    unwrap_unknown_wrappers: bool | None = None,
) -> list[BulkConvertResult]: ...
