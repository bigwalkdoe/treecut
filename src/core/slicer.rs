use crate::core::models::{ReadNodeArgs, SliceSymbolArgs, ToolOutput};
use crate::core::parser::parse_file;
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use std::path::Path;

pub fn slice_symbol(args: &SliceSymbolArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let symbol = &args.symbol;
    let context_lines = args.context_lines.unwrap_or(2);
    let max_chars = args.max_chars.unwrap_or(16000);
    let mut out = String::new();
    for p in walk_files(root_path) {
        if language_for(&p).is_none() { continue; }
        let source = match std::fs::read_to_string(&p) { Ok(s) => s, Err(_) => continue };
        let syms = parse_file(&p, &source);
        for s in &syms {
            if s.name == *symbol {
                let lines: Vec<&str> = source.lines().collect();
                let start = s.start_line.saturating_sub(context_lines).max(1);
                let end = (s.end_line + context_lines).min(lines.len());
                out.push_str(&format!("--- {}:{}-{} ---\n", p.display(), s.start_line, s.end_line));
                for i in start..=end {
                    out.push_str(&format!("{:>4}: {}\n", i, lines[i-1]));
                }
            }
        }
    }
    if out.len() > max_chars { out.truncate(max_chars); out.push_str("\n...[truncated]"); }
    Ok(ToolOutput { text: out })
}

pub fn read_node(args: &ReadNodeArgs) -> Result<ToolOutput> {
    let path = Path::new(&args.path);
    let source = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = source.lines().collect();
    let start = args.start_line.max(1).min(lines.len());
    let end = args.end_line.min(lines.len());
    let mut out = String::new();
    for i in start..=end {
        out.push_str(&format!("{:>4}: {}\n", i, lines[i-1]));
    }
    Ok(ToolOutput { text: out })
}
