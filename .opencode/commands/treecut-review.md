---
description: Perform AST blast-radius code review or targeted symbol refactoring using treecut
---

You have access to the `treecut` MCP server (`repo_skeleton`, `slice_symbol`, `slice_changed_symbols`, and `read_node`). Use `treecut` strictly instead of full-file reads or broad `grep` scans to keep context minimal and surgical.

Target input: `$ARGUMENTS`

## Execution Workflow

Determine the mode from `$ARGUMENTS` and execute the matching tool chain:

### Mode A: Git Diff / PR Blast-Radius Review
*(Trigger when `$ARGUMENTS` is empty, `"diff"`, `"review"`, or a Git ref like `"main"` or `"HEAD~1"`)*

1. **Map Modified Symbols & Callers**: Call `treecut_slice_changed_symbols` with `include_callers: true` (and `base_ref` set if a branch/commit ref was provided).
2. **Inspect Structural Context**: If any modified symbol touches core architecture or unfamiliar modules, call `treecut_repo_skeleton` with `max_depth: 3` to verify module boundaries.
3. **Deep-Read Only Narrow Spans**: Only call `treecut_read_node` on specific line ranges where caller invariants or error handling look suspect.
4. **Deliver Review**:
   * Summarize every modified symbol and its cross-file blast radius.
   * Flag breaking signature changes, unhandled edge cases, or stale callers.
   * Keep feedback concrete with `file:line` references.

### Mode B: Symbol Trace or Refactoring Plan
*(Trigger when `$ARGUMENTS` names a function, struct, class, method, or refactoring goal)*

1. **Slice the Symbol**: Extract the exact identifier from `$ARGUMENTS` and call `treecut_slice_symbol` (`context_lines: 3`).
2. **Trace Upstream/Downstream Dependencies**: If the symbol wraps or delegates to a second critical type/function, run one follow-up `treecut_slice_symbol` on that dependency.
3. **Read Exact Edit Spans**: Call `treecut_read_node` only for the exact `start_line..end_line` spans you intend to modify.
4. **Execute or Propose Changes**:
   * List every definition and call site affected by the change.
   * Apply clean, atomic edits across the definition and all impacted callers, or present a step-by-step refactoring plan if the user asked for analysis only.

## Rules
* **Never dump whole files** when `slice_symbol` or `read_node` can fetch the exact AST node.
* **Cite exact `path:start_line-end_line`** spans from `treecut` outputs in your final response.
