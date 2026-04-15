# API Reference

mdka exposes a small, focused public API. The table below shows the complete
surface — every function and type you need, nothing you don't.

## Functions

| Function | Language | Description |
|---|---|---|
| [`html_to_markdown`](./core.md#html_to_markdown) | Rust | Convert HTML string → Markdown (default mode) |
| [`html_to_markdown_with`](./core.md#html_to_markdown_with) | Rust | Convert with explicit `ConversionOptions` |
| [`html_file_to_markdown`](./core.md#html_file_to_markdown) | Rust | Convert one file; output alongside input or to `out_dir` |
| [`html_file_to_markdown_with`](./core.md#html_file_to_markdown_with) | Rust | Single file with options |
| [`html_files_to_markdown`](./core.md#html_files_to_markdown) | Rust | Parallel bulk conversion (rayon) |
| [`html_files_to_markdown_with`](./core.md#html_files_to_markdown_with) | Rust | Bulk with options |

## Types

| Type | Description |
|---|---|
| [`ConversionMode`](./modes.md) | Enum: `Balanced` · `Strict` · `Minimal` · `Semantic` · `Preserve` |
| [`ConversionOptions`](./options.md) | Controls pre-processing per-call; built via `for_mode()` |
| [`ConvertResult`](./core.md#convertresult) | Returned by single-file functions: `src` + `dest` paths |
| [`MdkaError`](./errors.md) | The only error type: wraps `std::io::Error` |

## Guarantees

- **`html_to_markdown` and `html_to_markdown_with` never panic.** They accept
  any `&str`, including empty strings, binary garbage, or deeply nested HTML.
- **File functions propagate IO errors** via `Result<_, MdkaError>`.
- **Output is always valid UTF-8.**
- **Output always ends with a single newline** when the input produces any content.
