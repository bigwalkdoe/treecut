use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::fmt;
use treecut::core;

#[derive(Parser)]
#[command(name = "treecut", version, about = "AST & Dependency Context Slicer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mcp,
    Map {
        path: Option<String>,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        stats: bool,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        include: Vec<String>,
    },
    Slice {
        #[arg(required = true, num_args = 1..)]
        symbols: Vec<String>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        context_lines: Option<usize>,
        #[arg(long)]
        stats: bool,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long, value_enum)]
        direction: Option<TraversalDirection>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        include: Vec<String>,
    },
    DiffSlice {
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        stats: bool,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long, value_enum)]
        direction: Option<TraversalDirection>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        include: Vec<String>,
    },
    Ask { prompt: String },
    Init {
        #[arg(long)]
        opencode: bool,
        #[arg(long)]
        claude: bool,
        #[arg(long)]
        cursor: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        path: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum TraversalDirection {
    Callers,
    Callees,
    Both,
}

impl From<TraversalDirection> for core::TraversalDirection {
    fn from(d: TraversalDirection) -> Self {
        match d {
            TraversalDirection::Callers => core::TraversalDirection::Callers,
            TraversalDirection::Callees => core::TraversalDirection::Callees,
            TraversalDirection::Both => core::TraversalDirection::Both,
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = fmt().with_writer(std::io::stderr).with_ansi(false).try_init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Mcp => treecut::mcp::run().await,
        Commands::Map { path, depth, max_chars, stats, scope, include } => {
            let args = core::RepoSkeletonArgs { root: path, max_depth: depth, max_chars, stats: Some(stats), no_cache: Some(false), scope, include: if include.is_empty() { None } else { Some(include) } };
            let r = core::repo_skeleton(&args).unwrap();
            println!("{}", r.text);
            if stats && r.stats.is_some() {
                print_stats(r.stats.as_ref().unwrap());
            }
        }
        Commands::Slice { symbols, path, max_chars, context_lines, stats, depth, direction, scope, include } => {
            let direction = direction.map(Into::into);
            let include = if include.is_empty() { None } else { Some(include) };
            if symbols.len() == 1 {
                let args = core::SliceSymbolArgs { root: path, symbol: symbols[0].clone(), max_chars, context_lines, stats: Some(stats), depth, direction, scope, include };
                let r = core::slice_symbol(&args).unwrap();
                println!("{}", r.text);
                if stats && r.stats.is_some() {
                    print_stats(r.stats.as_ref().unwrap());
                }
            } else {
                let args = core::SliceSymbolsArgs { root: path, symbols, max_chars, context_lines, stats: Some(stats), depth, direction, scope, include };
                let r = core::slice_symbols(&args).unwrap();
                println!("{}", r.text);
                if stats && r.stats.is_some() {
                    print_stats(r.stats.as_ref().unwrap());
                }
            }
        }
        Commands::DiffSlice { path, max_chars, base, stats, depth, direction, scope, include } => {
            let direction = direction.map(Into::into);
            let include = if include.is_empty() { None } else { Some(include) };
            let args = core::SliceChangedArgs { root: path, max_chars, base_ref: base, stats: Some(stats), include_callers: Some(true), depth, direction, scope, include };
            let r = core::slice_changed_symbols(&args).unwrap();
            println!("{}", r.text);
            if stats && r.stats.is_some() {
                print_stats(r.stats.as_ref().unwrap());
            }
        }
        Commands::Ask { prompt } => {
            #[cfg(feature = "cli")]
            {
                let model = std::env::var("TREECT_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
                let base_url = std::env::var("TREECT_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
                treecut::agent::run_ask(&prompt, ".", &model, &base_url).await;
            }
            #[cfg(not(feature = "cli"))]
            { let _ = prompt; eprintln!("ask requires cli feature"); }
        }
        Commands::Init { opencode, claude, cursor, all, path } => {
            let root = std::path::PathBuf::from(path.unwrap_or_else(|| ".".to_string()));
            let targets = resolve_init_targets(opencode, claude, cursor, all);
            if targets.is_empty() {
                eprintln!("No targets selected. Use --opencode, --claude, --cursor, or --all.");
                std::process::exit(2);
            }
            for t in &targets {
                if let Err(e) = init_target(&root, *t) {
                    eprintln!("Failed to initialize {:?}: {}", t, e);
                    std::process::exit(1);
                }
            }
            println!("Initialized treecut integration for: {}", targets.iter().map(|t| t.as_str()).collect::<Vec<_>>().join(", "));
        }
    }
}

fn print_stats(stats: &core::SliceStats) {
    eprintln!("\n=== Slice Statistics ===");
    eprintln!("Files scanned:       {}", stats.files_scanned);
    eprintln!("Files matched:       {}", stats.files_matched);
    eprintln!("Full files (bytes):  {}", stats.full_files_bytes);
    eprintln!("Sliced output (bytes): {}", stats.sliced_bytes);
    eprintln!("Est. tokens saved:   {}", stats.estimated_tokens_saved);
    eprintln!("Reduction:           {:.1}%", stats.reduction_percent);
}

#[derive(Clone, Copy, Debug)]
enum InitTarget {
    OpenCode,
    Claude,
    Cursor,
}

impl InitTarget {
    fn as_str(&self) -> &'static str {
        match self {
            InitTarget::OpenCode => "opencode",
            InitTarget::Claude => "claude",
            InitTarget::Cursor => "cursor",
        }
    }
}

fn resolve_init_targets(opencode: bool, claude: bool, cursor: bool, all: bool) -> Vec<InitTarget> {
    if all || (!opencode && !claude && !cursor) {
        return vec![InitTarget::OpenCode, InitTarget::Claude, InitTarget::Cursor];
    }
    let mut out = Vec::new();
    if opencode { out.push(InitTarget::OpenCode); }
    if claude { out.push(InitTarget::Claude); }
    if cursor { out.push(InitTarget::Cursor); }
    out
}

fn init_target(root: &std::path::Path, target: InitTarget) -> anyhow::Result<()> {
    match target {
        InitTarget::OpenCode => init_opencode(root),
        InitTarget::Claude => init_claude(root),
        InitTarget::Cursor => init_cursor(root),
    }
}

fn init_opencode(root: &std::path::Path) -> anyhow::Result<()> {
    let cmd_dir = root.join(".opencode/commands");
    std::fs::create_dir_all(&cmd_dir)?;
    std::fs::write(cmd_dir.join("slice.md"), SLICE_MD_OPENCODE)?;

    let cfg_path = root.join("opencode.json");
    let mut cfg = read_json_or_default(&cfg_path)?;
    let mcp = cfg.as_object_mut().unwrap().entry("mcp").or_insert_with(|| serde_json::json!({}));
    let mcp_obj = mcp.as_object_mut().ok_or_else(|| anyhow::anyhow!("opencode.json: mcp must be an object"))?;
    mcp_obj.entry("treecut").or_insert_with(|| serde_json::json!({
        "type": "local",
        "command": ["treecut", "mcp"],
        "enabled": true
    }));
    write_json_pretty(&cfg_path, &cfg)?;
    Ok(())
}

fn init_claude(root: &std::path::Path) -> anyhow::Result<()> {
    let cmd_dir = root.join(".claude/commands");
    std::fs::create_dir_all(&cmd_dir)?;
    std::fs::write(cmd_dir.join("slice.md"), SLICE_MD_CLAUDE)?;

    let cfg_path = root.join(".mcp.json");
    let mut cfg = read_json_or_default(&cfg_path)?;
    let servers = cfg.as_object_mut().unwrap().entry("mcpServers").or_insert_with(|| serde_json::json!({}));
    let servers_obj = servers.as_object_mut().ok_or_else(|| anyhow::anyhow!(".mcp.json: mcpServers must be an object"))?;
    servers_obj.entry("treecut").or_insert_with(|| serde_json::json!({
        "command": "treecut",
        "args": ["mcp"]
    }));
    write_json_pretty(&cfg_path, &cfg)?;
    Ok(())
}

fn init_cursor(root: &std::path::Path) -> anyhow::Result<()> {
    let cmd_dir = root.join(".cursor/commands");
    std::fs::create_dir_all(&cmd_dir)?;
    std::fs::write(cmd_dir.join("slice.md"), SLICE_MD_CURSOR)?;

    let cfg_path = root.join(".cursor/mcp.json");
    if let Some(parent) = cfg_path.parent() { std::fs::create_dir_all(parent)?; }
    let mut cfg = read_json_or_default(&cfg_path)?;
    let servers = cfg.as_object_mut().unwrap().entry("mcpServers").or_insert_with(|| serde_json::json!({}));
    let servers_obj = servers.as_object_mut().ok_or_else(|| anyhow::anyhow!(".cursor/mcp.json: mcpServers must be an object"))?;
    servers_obj.entry("treecut").or_insert_with(|| serde_json::json!({
        "command": "treecut",
        "args": ["mcp"]
    }));
    write_json_pretty(&cfg_path, &cfg)?;
    Ok(())
}

fn read_json_or_default(path: &std::path::Path) -> anyhow::Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let text = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
    if !v.is_object() {
        anyhow::bail!("{} must be a JSON object", path.display());
    }
    Ok(v)
}

fn write_json_pretty(path: &std::path::Path, value: &serde_json::Value) -> anyhow::Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    std::fs::write(path, text)?;
    Ok(())
}

const SLICE_MD_OPENCODE: &str = r#"---
description: Trace symbol definitions and call graphs with treecut depth/direction/scope slicing
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
"#;

const SLICE_MD_CLAUDE: &str = r#"---
description: Trace symbol definitions and call graphs with treecut depth/direction/scope slicing
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
"#;

const SLICE_MD_CURSOR: &str = r#"# AST Blast-Radius Review & Symbol Refactoring (`treecut`)

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
"#;
