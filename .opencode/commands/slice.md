---
description: Trace symbol definitions and call graphs with treecut v0.4.0 depth/direction/scope slicing
---

You have access to the `treecut` MCP server (`repo_skeleton`, `slice_symbol`, `slice_symbols`, `slice_changed_symbols`, `read_node`). Use `treecut` instead of full-file reads or broad `grep` scans.

Target input: `$ARGUMENTS`

## Goal

Produce a minimal, high-signal context slice for the requested symbol(s), bug trace, or refactor.

## Execution Workflow

1. **Parse intent from `$ARGUMENTS`**:
   - Identify symbol names (one or many), and any hints like "callers", "callees", "both", "depth N", "scope <path>", or "exclude/include tests".
   - If no symbol is given but a bug/refactor is described, start with `treecut_repo_skeleton` (`max_depth: 3`, `max_chars: 12000`) scoped if a path is obvious, then pick the most relevant symbol(s).

2. **Slice with v0.4.0 defaults**:
   - **Single symbol**: call `treecut_slice_symbol`.
   - **Multiple symbols**: call `treecut_slice_symbols` in one batch call.
   - Default args unless user specified otherwise:
     - `depth: 2`
     - `direction: "both"`
     - `context_lines: 3`
     - `max_chars: 16000`
     - `stats: true`
   - Use `scope` when the target is clearly inside a monorepo package/subdirectory.
   - Use `include` glob patterns to exclude noisy files when helpful (e.g. tests/generated files), only when it won't hide relevant call sites.

3. **Refine only if needed**:
   - If output is too broad/noisy, narrow with `scope`, reduce `depth` to 1, or set `direction` to `callers`/`callees`.
   - If output misses downstream/upstream context, increase `depth` (max 4) or switch `direction` to `both`.
   - Use `treecut_read_node` only for exact spans that need full inspection before editing.

4. **Report**:
   - Summarize the seed symbol(s), hop direction/depth used, and key definition/call-site spans (`path:start_line-end_line`).
   - For refactors/bugs, list affected callers/callees by hop and call out risky edges.
   - If `stats` are available, include token/byte reduction.

## Rules

- **Never dump whole files** when `slice_symbol`/`slice_symbols`/`read_node` can fetch exact context.
- **Prefer batch slicing** (`slice_symbols`) over multiple single-symbol calls when tracing related symbols.
- **Cite exact `path:start_line-end_line` spans** from treecut outputs.
- **Keep scope explicit** in monorepos; don't scan the whole repo when a workspace scope is known.
