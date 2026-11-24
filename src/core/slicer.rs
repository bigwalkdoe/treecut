use crate::core::models::{ReadNodeArgs, SliceSymbolArgs, ToolOutput, ParsedFile};
use crate::core::parser::parse_file_with_imports;
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use std::path::Path;
use tree_sitter::{Parser, Node};

pub fn slice_symbol(args: &SliceSymbolArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let symbol = &args.symbol;
    let context_lines = args.context_lines.unwrap_or(2);
    let max_chars = args.max_chars.unwrap_or(16000);
    let mut out = String::new();
    for p in walk_files(root_path) {
        let lang = match language_for(&p) { Some(l) => l, None => continue };
        let source = match std::fs::read_to_string(&p) { Ok(s) => s, Err(_) => continue };
        let parsed: ParsedFile = parse_file_with_imports(&p, &source);
        
        // Build local name targets: original symbol + any aliases that map to it
        let mut local_names = vec![symbol.clone()];
        for imp in &parsed.imports {
            if imp.original == *symbol {
                local_names.push(imp.alias.clone());
            }
        }
        
        // Parse the file with tree-sitter to find identifier references
        let ts_lang = match crate::core::parser::lang_for(lang) { Some(l) => l, None => continue };
        let mut parser = Parser::new();
        if parser.set_language(&ts_lang).is_err() { continue; }
        let tree = match parser.parse(&source, None) { Some(t) => t, None => continue };
        
        // Track which lines we've already output to avoid duplicates
        let mut emitted_ranges: Vec<(usize, usize)> = Vec::new();
        
        // Check symbol definitions
        for s in &parsed.symbols {
            if local_names.iter().any(|name| s.name == *name) {
                let lines: Vec<&str> = source.lines().collect();
                let start = s.start_line.saturating_sub(context_lines).max(1);
                let end = (s.end_line + context_lines).min(lines.len());
                if !ranges_overlap(&emitted_ranges, start, end) {
                    let via_alias = if s.name != *symbol {
                        parsed.imports.iter()
                            .find(|imp| imp.alias == s.name && imp.original == *symbol)
                            .map(|imp| format!(" (via alias {})", imp.alias))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    out.push_str(&format!("--- {}:{}-{} {} ---\n", p.display(), s.start_line, s.end_line, via_alias));
                    for i in start..=end {
                        out.push_str(&format!("{:>4}: {}\n", i, lines[i-1]));
                    }
                    emitted_ranges.push((start, end));
                }
            }
        }
        
        // Find identifier references (call sites) matching local names
        let refs = find_identifier_references(&tree.root_node(), &source, &local_names, &parsed.imports, symbol);
        for (line, via_alias) in refs {
            let start = line.saturating_sub(context_lines).max(1);
            let end = (line + context_lines).min(source.lines().count());
            // Only deduplicate exact same line matches, not overlapping context
            if !emitted_ranges.iter().any(|&(s, e)| s == start && e == end) {
                let lines: Vec<&str> = source.lines().collect();
                out.push_str(&format!("--- {}:{}-{} {} ---\n", p.display(), line, line, via_alias));
                for i in start..=end {
                    out.push_str(&format!("{:>4}: {}\n", i, lines[i-1]));
                }
                emitted_ranges.push((start, end));
            }
        }
    }
    if out.len() > max_chars { out.truncate(max_chars); out.push_str("\n...[truncated]"); }
    Ok(ToolOutput { text: out })
}

fn ranges_overlap(emitted: &[(usize, usize)], start: usize, end: usize) -> bool {
    for &(s, e) in emitted {
        if start <= e && end >= s {
            return true;
        }
    }
    false
}

fn find_identifier_references(node: &Node, source: &str, local_names: &[String], imports: &[crate::core::models::ImportAlias], target_symbol: &str) -> Vec<(usize, String)> {
    let mut refs = Vec::new();
    if node.kind() == "identifier" {
        let text = source[node.byte_range()].to_string();
        if local_names.contains(&text) {
            let via_alias = if text != *target_symbol {
                imports.iter()
                    .find(|imp| imp.alias == text && imp.original == *target_symbol)
                    .map(|imp| format!(" (via alias {})", imp.alias))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            refs.push((node.start_position().row + 1, via_alias));
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        refs.extend(find_identifier_references(&child, source, local_names, imports, target_symbol));
    }
    refs
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
