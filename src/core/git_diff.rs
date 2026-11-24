use crate::core::models::{SliceChangedArgs, SliceSymbolsArgs, ToolOutput, SliceStats};
use crate::core::parser::parse_file;
use crate::core::slicer::slice_symbols;
use crate::core::walker::language_for;
use anyhow::Result;
use git2::Repository;
use std::path::Path;

pub fn slice_changed_symbols(args: &SliceChangedArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let _ = args.include_callers; // TODO: implement caller inclusion
    let max_chars = args.max_chars.unwrap_or(16000);
    let track_stats = args.stats.unwrap_or(false);
    let repo = Repository::discover(root_path)?;
    // diff
    let (old_tree, new_tree) = if let Some(base) = &args.base_ref {
        let base_obj = repo.revparse_single(base)?;
        let base_commit = base_obj.peel_to_commit()?;
        let base_tree = base_commit.tree()?;
        let head_commit = repo.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        (Some(base_tree), Some(head_tree))
    } else {
        let head_commit = repo.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        (Some(head_tree), None)
    };
    let diff = repo.diff_tree_to_tree(old_tree.as_ref(), new_tree.as_ref(), None)?;
    let diff = if new_tree.is_none() {
        repo.diff_tree_to_workdir_with_index(old_tree.as_ref(), None)?
    } else { diff };
    // collect file paths
    let mut files: Vec<String> = Vec::new();
    diff.foreach(&mut |delta, _| {
        if let Some(p) = delta.new_file().path() {
            files.push(p.to_string_lossy().to_string());
        }
        true
    }, None, None, None)?;
    // collect hunks per file
    let mut hunks_map: std::collections::HashMap<String, Vec<(usize,usize)>> = std::collections::HashMap::new();
    for f in &files { hunks_map.insert(f.clone(), Vec::new()); }
    diff.foreach(
        &mut |_, _| true,
        None,
        Some(&mut |delta, hunk| {
            if let Some(p) = delta.new_file().path() {
                let path_str = p.to_string_lossy().to_string();
                if let Some(v) = hunks_map.get_mut(&path_str) {
                    v.push((hunk.new_start() as usize, (hunk.new_start() + hunk.new_lines()) as usize));
                }
            }
            true
        }),
        None,
    )?;
    let changed_files: Vec<(String, Vec<(usize,usize)>)> = files.into_iter().map(|f| (f.clone(), hunks_map.get(&f).cloned().unwrap_or_default())).collect();
    // map to symbols
    let mut modified_symbols: Vec<String> = Vec::new();
    for (rel, ranges) in &changed_files {
        let full = root_path.join(rel);
        if !full.exists() { continue; }
        if language_for(&full).is_none() { continue; }
        let source = match std::fs::read_to_string(&full) { Ok(s) => s, Err(_) => continue };
        let syms = parse_file(&full, &source);
        for s in &syms {
            for (a, b) in ranges {
                if s.start_line <= *b && s.end_line >= *a && !modified_symbols.iter().any(|n| n == &s.name) {
                    modified_symbols.push(s.name.clone());
                }
            }
        }
    }
    
    let mut out = String::new();
    let mut total_files_scanned = 0;
    let mut total_files_matched = 0;
    let mut total_full_bytes = 0;
    let mut total_sliced_bytes = 0;
    
    for sym in &modified_symbols {
        out.push_str(&format!("=== Modified symbol: {} ===\n", sym));
        let r = slice_symbols(&SliceSymbolsArgs {
            root: Some(root.clone()),
            symbols: vec![sym.clone()],
            context_lines: Some(2),
            max_chars: Some(max_chars),
            stats: Some(track_stats),
        })?;
        out.push_str(&r.text);
        out.push('\n');
        
        if let Some(stats) = r.stats.filter(|_| track_stats) {
            total_files_scanned += stats.files_scanned;
            total_files_matched += stats.files_matched;
            total_full_bytes += stats.full_files_bytes;
            total_sliced_bytes += stats.sliced_bytes;
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
