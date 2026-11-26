# AST Blast-Radius Review & Symbol Refactoring (`treecut`)

Use the `treecut` MCP tools (`repo_skeleton`, `slice_symbol`, `slice_symbols`, `slice_changed_symbols`, and `read_node`) instead of full-file reads or broad `grep` scans to keep context minimal and surgical.

Target input: $ARGUMENTS

## Workflow

1. **If `$ARGUMENTS` is empty, `"diff"`, `"review"`, or a Git ref (e.g. `main`, `HEAD~1`)**:
   - Call `treecut_slice_changed_symbols` with `include_callers: true` (and `base_ref` if specified).
   - Call `treecut_repo_skeleton` (`max_depth: 3`) if architectural context is needed.
   - Review every modified symbol, flag breaking changes or stale callers, and cite exact `file:line` spans.

2. **If `$ARGUMENTS` names one or more symbols or a refactoring goal**:
   - Call `treecut_slice_symbol` (or `treecut_slice_symbols` for multiple targets) with `depth: 2` and `direction: "both"`.
   - Use `treecut_read_node` only on the narrow `start_line..end_line` spans you plan to edit.
   - Apply clean edits across the definition and all impacted callers, or present a step-by-step refactoring plan if asked for analysis only.

## Rules

- Never dump whole files when `slice_symbol`/`slice_symbols`/`read_node` can fetch exact context.
- Prefer batch slicing (`slice_symbols`) for multiple related symbols.
- Cite exact `path:start_line-end_line` spans from treecut outputs.
