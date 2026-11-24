use crate::core::models::{ReadNodeArgs, SliceSymbolArgs, SliceSymbolsArgs, SliceStats, ToolOutput, ParsedFile};
use crate::core::parser::parse_file_with_imports;
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use std::path::Path;
use tree_sitter::{Parser, Node};

pub fn slice_symbol(args: &SliceSymbolArgs) -> Result<ToolOutput> {
    slice_symbols_internal(&SliceSymbolsArgs {
        root: args.root.clone(),
        symbols: vec![args.symbol.clone()],
        context_lines: args.context_lines,
        max_chars: args.max_chars,
        stats: args.stats,
    })
}

pub fn slice_symbols(args: &SliceSymbolsArgs) -> Result<ToolOutput> {
    slice_symbols_internal(args)
}

fn slice_symbols_internal(args: &SliceSymbolsArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let symbols = &args.symbols;
    let context_lines = args.context_lines.unwrap_or(2);
    let max_chars = args.max_chars.unwrap_or(16000);
    let track_stats = args.stats.unwrap_or(false);
    
    // Deduplicate symbols
    let unique_symbols: Vec<String> = symbols.iter().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
    
    let mut out = String::new();
    let mut total_files_scanned = 0;
    let mut total_files_matched = 0;
    let mut total_full_bytes = 0;
    let mut total_sliced_bytes = 0;
    
    for p in walk_files(root_path) {
        let lang = match language_for(&p) { Some(l) => l, None => continue };
        let source = match std::fs::read_to_string(&p) { Ok(s) => s, Err(_) => continue };
        
        if track_stats {
            total_files_scanned += 1;
            total_full_bytes += source.len();
        }
        
        let parsed: ParsedFile = parse_file_with_imports(&p, &source);
        
        // Build local name targets for ALL symbols
        let mut symbol_to_locals: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for sym in &unique_symbols {
            let mut local_names = vec![sym.clone()];
            for imp in &parsed.imports {
                if imp.original == *sym {
                    local_names.push(imp.alias.clone());
                }
            }
            symbol_to_locals.insert(sym.clone(), local_names);
        }
        
        // Parse the file with tree-sitter to find identifier references
        let ts_lang = match crate::core::parser::lang_for(lang) { Some(l) => l, None => continue };
        let mut parser = Parser::new();
        if parser.set_language(&ts_lang).is_err() { continue; }
        let tree = match parser.parse(&source, None) { Some(t) => t, None => continue };
        
        // Track which line ranges we've already output to avoid duplicates
        let mut emitted_ranges: Vec<(usize, usize)> = Vec::new();
        let mut file_matched = false;
        let mut file_sliced_bytes = 0;
        
        // Check symbol definitions
        for s in &parsed.symbols {
            for sym in &unique_symbols {
                let local_names = symbol_to_locals.get(sym).unwrap();
                if local_names.contains(&s.name) {
                    let lines: Vec<&str> = source.lines().collect();
                    let start = s.start_line.saturating_sub(context_lines).max(1);
                    let end = (s.end_line + context_lines).min(lines.len());
                    if !ranges_overlap(&emitted_ranges, start, end) {
                        let via_alias = if s.name != *sym {
                            parsed.imports.iter()
                                .find(|imp| imp.alias == s.name && imp.original == *sym)
                                .map(|imp| format!(" (via alias {})", imp.alias))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        let header = format!("--- {}:{}-{} {} ---\n", p.display(), s.start_line, s.end_line, via_alias);
                        let content: String = (start..=end).map(|i| format!("{:>4}: {}\n", i, lines[i-1])).collect();
                        let block = header + &content;
                        
                        file_sliced_bytes += block.len();
                        out.push_str(&block);
                        emitted_ranges.push((start, end));
                        file_matched = true;
                    }
                }
            }
        }
        
        // Find identifier references (call sites) matching local names for each symbol
        for sym in &unique_symbols {
            let local_names = symbol_to_locals.get(sym).unwrap();
            let refs = find_identifier_references(&tree.root_node(), &source, local_names, &parsed.imports, sym);
            for (line, via_alias) in refs {
                let start = line.saturating_sub(context_lines).max(1);
                let end = (line + context_lines).min(source.lines().count());
                // Deduplicate: skip if exact same context range already emitted, or if fully contained in existing range
                let is_duplicate = emitted_ranges.iter().any(|&(s, e)| {
                    (s == start && e == end) || (s <= start && e >= end)
                });
                if !is_duplicate {
                    let lines: Vec<&str> = source.lines().collect();
                    let header = format!("--- {}:{}-{} {} ---\n", p.display(), line, line, via_alias);
                    let content: String = (start..=end).map(|i| format!("{:>4}: {}\n", i, lines[i-1])).collect();
                    let block = header + &content;
                    
                    file_sliced_bytes += block.len();
                    out.push_str(&block);
                    emitted_ranges.push((start, end));
                    file_matched = true;
                }
            }
        }
        
        if file_matched && track_stats {
            total_files_matched += 1;
            total_sliced_bytes += file_sliced_bytes;
        }
    }
    
    if out.len() > max_chars { out.truncate(max_chars); out.push_str("\n...[truncated]"); }
    
    let stats = if track_stats {
        let full = total_full_bytes.max(1);
        let saved = full.saturating_sub(total_sliced_bytes);
        let reduction = (saved as f32 / full as f32) * 100.0;
        let estimated_tokens = saved / 4; // ~4 chars per token
        Some(SliceStats {
            files_scanned: total_files_scanned,
            files_matched: total_files_matched,
            full_files_bytes: total_full_bytes,
            sliced_bytes: total_sliced_bytes,
            estimated_tokens_saved: estimated_tokens,
            reduction_percent: reduction,
        })
    } else {
        None
    };
    
    Ok(ToolOutput { text: out, stats })
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
        if local_names.iter().any(|n| n == &text) {
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
    Ok(ToolOutput { text: out, stats: None })
}
