use crate::core::cache::{Cache, CacheEntry};
use crate::core::models::{RepoSkeletonArgs, ToolOutput, SliceStats};
use crate::core::parser::{parse_file_with_imports, skeleton_text};
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use serde_json;
use std::path::Path;

pub fn repo_skeleton(args: &RepoSkeletonArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let max_chars = args.max_chars.unwrap_or(12000);
    let no_cache = args.no_cache.unwrap_or(false);
    let track_stats = args.stats.unwrap_or(false);
    let cache = if no_cache { None } else { Cache::open(root_path).ok() };
    let max_depth = args.max_depth.unwrap_or(4);
    let files = walk_files(root_path);
    let files: Vec<_> = files.into_iter().filter(|p| {
        p.strip_prefix(root_path).ok().map(|r| r.components().count() <= max_depth).unwrap_or(true)
    }).collect();
    let rels: Vec<String> = files.iter().filter_map(|p| p.strip_prefix(root_path).ok().map(|r| r.to_string_lossy().to_string())).collect();
    if let Some(c) = &cache { let _ = c.prune(&rels); }
    let mut entries: Vec<String> = Vec::new();
    let mut total_files_scanned = 0;
    let mut total_full_bytes = 0;
    let mut total_sliced_bytes = 0;
    let results: Vec<String> = files.iter().map(|p| {
        let lang = match language_for(p) { Some(l) => l, None => return String::new() };
        let source = match std::fs::read_to_string(p) { Ok(s) => s, Err(_) => return String::new() };
        if track_stats {
            total_files_scanned += 1;
            total_full_bytes += source.len();
        }
        let rel = p.strip_prefix(root_path).unwrap_or(p).to_string_lossy().to_string();
        let mtime = crate::core::cache::mtime_ns(p);
        let byte_len = source.len() as i64;
        let skel = if let Some(c) = &cache {
            if let Some((s, _, _)) = c.get(&rel, mtime, byte_len) {
                s
            } else {
                let parsed = parse_file_with_imports(p, &source);
                let s = skeleton_text(p, &source, &parsed.symbols);
                let j = serde_json::to_string(&parsed.symbols).unwrap_or_default();
                let imp = serde_json::to_string(&parsed.imports).unwrap_or_default();
                let _ = c.put(&CacheEntry {
                rel: rel.clone(),
                mtime_ns: mtime,
                byte_len,
                language: lang.to_string(),
                skeleton: s.clone(),
                symbols_json: j,
                imports_json: imp,
            });
                s
            }
        } else {
            let parsed = parse_file_with_imports(p, &source);
            skeleton_text(p, &source, &parsed.symbols)
        };
        if track_stats {
            total_sliced_bytes += skel.len();
        }
        skel
    }).collect();
    for r in results { if !r.is_empty() { entries.push(r); } }
    let mut text = entries.join("\n");
    if text.len() > max_chars { text.truncate(max_chars); text.push_str("\n...[truncated]"); }
    
    let stats = if track_stats {
        let full = total_full_bytes.max(1);
        let saved = full.saturating_sub(total_sliced_bytes);
        let reduction = (saved as f32 / full as f32) * 100.0;
        let estimated_tokens = saved / 4;
        Some(SliceStats {
            files_scanned: total_files_scanned,
            files_matched: entries.len(),
            full_files_bytes: total_full_bytes,
            sliced_bytes: total_sliced_bytes,
            estimated_tokens_saved: estimated_tokens,
            reduction_percent: reduction,
        })
    } else {
        None
    };
    
    Ok(ToolOutput { text, stats })
}
