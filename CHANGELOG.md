# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-11-25

### Changed
- Added GitHub release notes across all tagged releases and updated changelog maintenance.
- Untracked machine-specific `opencode.json` from the repository and added it to `.gitignore`.

## [0.4.0] - 2026-11-25

### Added
- **Transitive call-graph traversal** for symbol slicing with `--depth` and `--direction` (`callers`, `callees`, `both`), enabling hop-grouped context beyond direct call sites.
- **Monorepo scope/include filtering** for map/slice workflows via `--scope` and `--include`, making targeted slices easier in large repositories.

### Changed
- Extended CLI and MCP interfaces to expose v0.4.0 traversal and filtering arguments.
- Improved slicing behavior around call-site headers and symbol/file handling for traversal workflows.

### Fixed
- Repo hygiene and build/test fixes for the v0.4.0 state (license/changelog metadata, feature/dependency cleanup, and parser/slicer correctness fixes).
- Clippy/quality fixes across parser, slicer, and skeleton code paths.

## [0.3.0] - 2026-11-24

### Added
- **Multi-Symbol Batch Slicing (`slice_symbols`)**: Trace multiple functions, structs, or classes in a single MCP tool call or CLI invocation (`treecut slice SymbolA SymbolB SymbolC`). Automatically merges overlapping line spans when multiple target symbols share callers, eliminating duplicate context blocks.
- **Token-Savings Telemetry (`--stats`)**: Every slice and skeleton operation calculates exact byte and estimated token savings compared to reading full matched files. Pass `--stats` to `treecut map`, `treecut slice`, or `treecut diff-slice` to print compression metrics to `stderr` without polluting `stdout` pipes.
- **Import Alias Resolution**: `slice_symbol` and `slice_changed_symbols` resolve aliased imports across files (Rust `use foo as bar`, TypeScript `import { foo as bar }`, Python `from foo import bar as baz`, Go `import bar "foo"`, C# `using Bar = Foo;`) and annotate call sites with `(via alias <name>)`.
- **6 New Tier-2 Language Support**: JavaScript/JSX, C#, Ruby, PHP, Swift, and Bash/Shell (total 12 languages now supported).

### Changed
- CLI now accepts multiple symbols for `treecut slice` command.
- MCP `slice_symbols` tool for batch slicing multiple symbols.
- Improved deduplication logic to handle overlapping call sites across multiple target symbols.

### Fixed
- SQLite cache migration for v0.1.0 caches (adds `imports_json` column).

## [0.2.0] - 2026-11-15

### Added
- Import alias resolution for Rust, TypeScript, Python, Go, and C#.
- Cross-file call site detection using local aliases.
- Annotated output showing `(via alias <name>)` for aliased call sites.

### Changed
- Enhanced parser to extract import alias information per file.
- Updated slicer to track local name mappings for each target symbol.

## [0.1.0] - 2026-11-01

### Added
- Initial release.
- Surgical Symbol Slicing (`slice_symbol`): Extracts a target function, struct, or class definition alongside every cross-file call site and enclosing signature.
- Git-Diff Blast Radius (`slice_changed_symbols`): Intersects `git diff` hunks with parsed AST spans to isolate modified symbols and automatically pull in their callers.
- Compressed Repo Maps (`repo_skeleton`): Generates token-budgeted architectural outlines with function bodies stripped out.
- Incremental SQLite `mtime` Cache: Indexes unchanged files in `.treecut/cache.db` so repeated slices across large repositories run in single-digit milliseconds.
- 6 Tier-1 Languages: Rust, Python, TypeScript/TSX, Go, Java, C/C++.
- Dual-Mode Binary: Runs as headless `stdio` MCP Server or standalone Agentic CLI.
- Zero `stdout` Pollution: All diagnostics and `tracing` logs route strictly to `stderr`.
- Automatic Cache Hygiene: Creates `.treecut/cache.db` alongside `.treecut/.gitignore` (`*`).