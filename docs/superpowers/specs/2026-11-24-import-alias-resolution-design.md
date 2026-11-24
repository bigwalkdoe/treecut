# Import Alias Resolution Design

## Intent
Enable `slice_symbol` to find call sites that use aliased imports (e.g., `use crate::auth::verify_token as check_tok;` or `import { verify_token as checkTok } from './auth'`).

## Goals
- Extract import aliases during AST parsing for Rust, TypeScript/JavaScript, Python, Go, C#
- Store aliases in SQLite cache alongside symbols
- Expand match targets per file to include local aliases
- Annotate aliased matches in output with `(via alias <name>)`

## Non-Goals
- Full type resolution or cross-module analysis beyond import aliases
- Export alias resolution (only import aliases)
- Dynamic imports or runtime aliasing

## Constraints
- Must maintain backward compatibility with v0.1.0 cache format
- Cache schema version bump to invalidate stale caches
- Zero additional dependencies

## Approach
1. Add `ImportAlias` struct to `models.rs`
2. Extend `parse_file` to return `(Vec<SymbolInfo>, Vec<ImportAlias>)` or create a `ParsedFile` struct
3. Update cache schema to store imports JSON
4. Implement alias extraction in `parser.rs` for 5 languages
5. Update `skeleton.rs` to cache imports
6. Update `slicer.rs` to build per-file alias map and match against both original and alias names
7. Add integration tests for Rust, TypeScript, Python

## Tree-sitter Node Patterns
| Language | Import Node | Original Name | Alias Name |
|----------|-------------|---------------|------------|
| Rust | `use_as_clause` | path segment | alias identifier |
| TypeScript/JS | `import_specifier` | name | alias (if `as`) |
| Python | `aliased_import` | name (last segment) | alias |
| Go | `import_spec` | package path (last segment) | alias identifier |
| C# | `using_directive` with `name_equals` | type name | alias name |

## Cache Schema Change
```sql
-- Add imports_json column, bump version
ALTER TABLE file_index ADD COLUMN imports_json TEXT NOT NULL DEFAULT '[]';
-- Or recreate table with new schema and version tracking
```

## Testing
- Cross-file aliased import tests for Rust, TypeScript, Python
- Verify `(via alias <name>)` annotation appears in output