# ast_analyser

Dart AST analysis CLI built on tree-sitter.

## What it does

Scans Dart source files for null-assertion operator (`!`) usage and reports locations as structured JSON. Designed to run in CI pipelines as a code quality gate — currently used in production at my employer to enforce null-safety discipline across the codebase.

## Tech Stack

- **Rust** — core language
- **tree-sitter** — incremental parsing framework for AST construction
- **tree-sitter-dart** — Dart grammar for tree-sitter
- **clap** — CLI argument parsing
- **walkdir** — recursive directory traversal
- **serde** — JSON serialization of analysis results

## Running locally

```bash
# Analyse a single file
cargo run -- file -f path/to/file.dart

# Analyse all files in a directory recursively
cargo run -- directory -d path/to/dart/project

# Analyse multiple specific files
cargo run -- files -f file1.dart,file2.dart
```

Output is JSON with transgression locations, counts, and affected files.

## Design Decisions

- **tree-sitter over regex**: AST-level analysis catches bang operators in all syntactic positions without false positives from comments or strings.
- **Structured JSON output**: Enables integration with CI tools, editors, and reporting dashboards without parsing human-readable text.
- **Subcommand-based CLI**: Separate `file`, `files`, and `directory` modes keep each path simple and explicit rather than overloading a single interface.
