use crate::core::models::{SliceChangedArgs, SliceSymbolArgs, ToolOutput};
use crate::core::parser::parse_file;
use crate::core::slicer::slice_symbol;
use crate::core::walker::{walk_files, language_for};
use anyhow::Result;
use git2::Repository;
use std::path::Path;

pub fn slice_changed_symbols(args: &SliceChangedArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let include_callers = args.include_callers.unwrap_or(true);
    let max_chars = args.max_chars.unwrap_or(16000);
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
                if s.start_line <= *b && s.end_line >= *a {
                    if !modified_symbols.contains(&s.name) { modified_symbols.push(s.name.clone()); }
                }
            }
        }
    }
    let mut out = String::new();
    for sym in &modified_symbols {
        out.push_str(&format!("=== Modified symbol: {} ===\n", sym));
        let r = slice_symbol(&SliceSymbolArgs {
            root: Some(root.clone()),
            symbol: sym.clone(),
            context_lines: Some(2),
            max_chars: Some(max_chars),
        })?;
        out.push_str(&r.text);
        out.push('\n');
        if include_callers {
            // callers already included via slice_symbol cross-file? our slice_symbol only defs+context; add note
        }
    }
    if out.len() > max_chars { out.truncate(max_chars); out.push_str("\n...[truncated]"); }
    Ok(ToolOutput { text: out })
}
