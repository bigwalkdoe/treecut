use crate::core::models::{ReadNodeArgs, SliceSymbolArgs, SliceSymbolsArgs, SliceStats, ToolOutput, ParsedFile, TraversalDirection, SymbolInfo};
use crate::core::parser::{parse_file_with_imports, lang_for};
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tree_sitter::{Parser, Node};

pub fn slice_symbol(args: &SliceSymbolArgs) -> Result<ToolOutput> {
    slice_symbols_internal(&SliceSymbolsArgs {
        root: args.root.clone(),
        symbols: vec![args.symbol.clone()],
        context_lines: args.context_lines,
        max_chars: args.max_chars,
        stats: args.stats,
        depth: args.depth,
        direction: args.direction,
        scope: args.scope.clone(),
        include: args.include.clone(),
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
    let max_depth = args.depth.unwrap_or(1).clamp(1, 4);
    let direction = args.direction.unwrap_or_default();
    
    // Stats tracking
    let mut total_files_scanned: usize = 0;
    let mut total_files_matched: usize = 0;
    let mut total_full_bytes: usize = 0;
    let mut total_sliced_bytes: usize = 0;
    
    // Build scope and include filters
    let scope_filter = args.scope.as_ref().map(|s| Path::new(s));
    let include_filter = args.include.as_ref().and_then(|incs| {
        let mut builder = globset::GlobSetBuilder::new();
        for inc in incs {
            if let Ok(glob) = globset::Glob::new(inc) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    });

    // Deduplicate initial symbols
    let initial_symbols: Vec<String> = symbols.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    
    // Phase 1: Collect all parsed files and build symbol index
    let mut file_cache: HashMap<String, (String, ParsedFile)> = HashMap::new();
    let mut symbol_index: HashMap<String, Vec<SymbolInfo>> = HashMap::new(); // symbol_name -> list of SymbolInfo (across files)
    
    for p in walk_files(root_path) {
        // Apply scope filter
        if let Some(scope) = scope_filter {
            if let Ok(rel) = p.strip_prefix(root_path) {
                if !rel.starts_with(scope) {
                    continue;
                }
            }
        }
        
        // Apply include filter (exclude patterns)
        if let Some(ref filter) = include_filter {
            if let Ok(rel) = p.strip_prefix(root_path) {
                let rel_str = rel.to_string_lossy();
                if filter.is_match(rel_str.as_ref()) {
                    continue; // exclude if matches include pattern (treat as exclude)
                }
            }
        }
        
        let _lang = match language_for(&p) { Some(l) => l, None => continue };
        let source = match std::fs::read_to_string(&p) { Ok(s) => s, Err(_) => continue };
        
        if track_stats {
            total_files_scanned += 1;
            total_full_bytes += source.len();
        }
        
        let parsed = parse_file_with_imports(&p, &source);
        let rel = p.strip_prefix(root_path).unwrap_or(&p).to_string_lossy().to_string();
        
        // Build symbol index
        for sym in &parsed.symbols {
            symbol_index.entry(sym.name.clone()).or_default().push(sym.clone());
        }
        
        file_cache.insert(rel, (source, parsed));
    }
    
    // Phase 2: BFS traversal to find all symbols within depth
    let mut visited_symbols: HashSet<String> = HashSet::new();
    let mut hop_symbols: Vec<Vec<String>> = Vec::new(); // symbols at each hop level
    let mut current_hop: Vec<String> = initial_symbols.clone();
    
    for _hop in 0..max_depth {
        if current_hop.is_empty() { break; }
        
        // Filter out already visited
        let new_symbols: Vec<String> = current_hop.iter()
            .filter(|s| visited_symbols.insert(s.to_string()))
            .cloned()
            .collect();
        
        if new_symbols.is_empty() { break; }
        hop_symbols.push(new_symbols.clone());
        
        // Find next hop symbols based on direction
        let mut next_hop = Vec::new();
        for sym_name in &new_symbols {
            if let Some(sym_infos) = symbol_index.get(sym_name) {
                for sym_info in sym_infos {
                    match direction {
                        TraversalDirection::Callers => {
                            // Find symbols that call this symbol (via callees field)
                            for (name, infos) in &symbol_index {
                                for info in infos {
                                    if info.callees.contains(sym_name) && !visited_symbols.contains(name) {
                                        next_hop.push(name.clone());
                                    }
                                }
                            }
                        }
                        TraversalDirection::Callees => {
                            // Find symbols this symbol calls
                            for callee in &sym_info.callees {
                                if !visited_symbols.contains(callee) {
                                    next_hop.push(callee.clone());
                                }
                            }
                        }
                        TraversalDirection::Both => {
                            // Both callers and callees
                            for callee in &sym_info.callees {
                                if !visited_symbols.contains(callee) {
                                    next_hop.push(callee.clone());
                                }
                            }
                            for (name, infos) in &symbol_index {
                                for info in infos {
                                    if info.callees.contains(sym_name) && !visited_symbols.contains(name) {
                                        next_hop.push(name.clone());
                                    }
                                }
                            }
                        }
                }
            }
        }
        
        next_hop.dedup();
        current_hop = next_hop.clone();
    }
}
    
    // Collect all symbols to include
    let _all_symbols: HashSet<String> = hop_symbols.iter().flatten().cloned().collect();
    
    // Phase 3: Output results grouped by hop
    let mut out = String::new();
    // Global deduplication across all symbols and hops
    let mut global_emitted: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    
    for (hop, symbols_at_hop) in hop_symbols.iter().enumerate() {
        if out.len() >= max_chars { break; }
        
        out.push_str(&format!("\n=== [Hop {}] ===\n", hop));
        
        for sym_name in symbols_at_hop {
            if out.len() >= max_chars { break; }
            
            let sym_infos = match symbol_index.get(sym_name) {
                Some(v) => v,
                None => continue,
            };
            
            // Group by file (normalize to rel path for file_cache lookup)
            let mut by_file: HashMap<String, Vec<&SymbolInfo>> = HashMap::new();
            for info in sym_infos {
                let rel = Path::new(&info.file).strip_prefix(root_path).unwrap_or(Path::new(&info.file)).to_string_lossy().to_string();
                by_file.entry(rel).or_default().push(info);
            }
            
            for (file, infos) in by_file {
                if out.len() >= max_chars { break; }
                
                let (source, _) = match file_cache.get(&file) {
                    Some(v) => v,
                    None => continue,
                };
                
                let lines: Vec<&str> = source.lines().collect();
                let mut file_sliced_bytes = 0;
                let mut file_matched = false;
                
                for info in infos {
                    if out.len() >= max_chars { break; }
                    
                    let start = info.start_line.saturating_sub(context_lines).max(1);
                    let end = (info.end_line + context_lines).min(lines.len());
                    
                    // Check global deduplication
                    let global_ranges = global_emitted.entry(file.clone()).or_default();
                    if ranges_overlap(global_ranges, start, end) {
                        continue;
                    }
                    
                    let header = format!("--- {}:{}-{} (hop {}) ---\n", file, info.start_line, info.end_line, hop);
                    let content: String = (start..=end).map(|i| format!("{:>4}: {}\n", i, lines[i-1])).collect();
                    let block = header + &content;
                    
                    file_sliced_bytes += block.len();
                    out.push_str(&block);
                    global_ranges.push((start, end));
                    file_matched = true;
                }
                
                if file_matched && track_stats {
                    total_files_matched += 1;
                    total_sliced_bytes += file_sliced_bytes;
                }
            }
        }
    }
    
    // Also find call sites for the initial symbols (hop 0)
    if direction == TraversalDirection::Callers || direction == TraversalDirection::Both {
        // Build a map of all local names for all initial symbols per file
        let mut file_to_symbol_locals: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        
        for (file, (_source, parsed)) in &file_cache {
            let mut symbol_to_locals: HashMap<String, Vec<String>> = HashMap::new();
            for sym_name in &initial_symbols {
                let mut local_names = vec![sym_name.clone()];
                for imp in &parsed.imports {
                    if imp.original == *sym_name {
                        local_names.push(imp.alias.clone());
                    }
                }
                symbol_to_locals.insert(sym_name.clone(), local_names);
            }
            file_to_symbol_locals.insert(file.clone(), symbol_to_locals);
        }
        
        // Print header for call sites section
        out.push_str("\n=== [Call Sites] ===\n");
        
        // Process each file once, finding call sites for all symbols
        for (file, (source, parsed)) in &file_cache {
            if out.len() >= max_chars { break; }
            
            let symbol_to_locals = match file_to_symbol_locals.get(file) {
                Some(m) => m,
                None => continue,
            };
            
            // Re-parse to get tree for identifier references
            let lang_str = language_for(Path::new(file)).unwrap_or("rust");
            let ts_lang = lang_for(lang_str).unwrap();
            let mut parser = Parser::new();
            parser.set_language(&ts_lang).unwrap();
            let tree = parser.parse(source, None).unwrap();
            
            let lines: Vec<&str> = source.lines().collect();
            let mut file_sliced_bytes = 0;
            let mut file_matched = false;
            
            // Find all call sites for all initial symbols in this file
            for sym_name in &initial_symbols {
                if out.len() >= max_chars { break; }
                
                let local_names = symbol_to_locals.get(sym_name).unwrap();
                let refs = find_identifier_references(&tree.root_node(), source, local_names, &parsed.imports, sym_name);
                
                for (line, via_alias) in refs {
                    if out.len() >= max_chars { break; }
                    
                    let start = line.saturating_sub(context_lines).max(1);
                    let end = (line + context_lines).min(lines.len());
                    
                    // Check global deduplication
                    let global_ranges = global_emitted.entry(file.clone()).or_default();
                    if ranges_overlap(global_ranges, start, end) {
                        continue;
                    }
                    
                    let header = format!("--- {}:{}-{} [{}]{} ---\n", file, line, line, sym_name, via_alias);
                    let content: String = (start..=end).map(|i| format!("{:>4}: {}\n", i, lines[i-1])).collect();
                    let block = header + &content;
                    
                    file_sliced_bytes += block.len();
                    out.push_str(&block);
                    global_ranges.push((start, end));
                    file_matched = true;
                }
            }
            
            if file_matched && track_stats {
                total_files_matched += 1;
                total_sliced_bytes += file_sliced_bytes;
            }
        }
    }
    
    if out.len() > max_chars { out.truncate(max_chars); out.push_str("\n...[truncated]"); }
    
    let stats = if track_stats {
        let full = total_full_bytes.max(1);
        let saved = full.saturating_sub(total_sliced_bytes);
        let reduction = (saved as f32 / full as f32) * 100.0;
        let estimated_tokens = saved / 4;
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
        // Only skip if exact duplicate or new range is completely contained in existing
        if (start == s && end == e) || (start >= s && end <= e) {
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
            // Skip identifiers that are part of import/use declarations (they're definitions, not references)
            if is_in_import_declaration(node) {
                return refs;
            }
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

fn is_in_import_declaration(node: &Node) -> bool {
    let mut parent = node.parent();
    while let Some(p) = parent {
        let kind = p.kind();
        if kind == "use_declaration" || kind == "import_statement" || kind == "import_from_statement" 
            || kind == "use_declaration" || kind == "using_directive" {
            return true;
        }
        parent = p.parent();
    }
    false
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