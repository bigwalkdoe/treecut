use crate::core::cache::Cache;
use crate::core::models::{RepoSkeletonArgs, ToolOutput};
use crate::core::parser::{parse_file, skeleton_text};
use crate::core::walker::{language_for, walk_files};
use anyhow::Result;
use serde_json;
use std::path::Path;

pub fn repo_skeleton(args: &RepoSkeletonArgs) -> Result<ToolOutput> {
    let root = args.root.clone().unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);
    let max_chars = args.max_chars.unwrap_or(12000);
    let no_cache = args.no_cache.unwrap_or(false);
    let cache = if no_cache { None } else { Cache::open(root_path).ok() };
    let max_depth = args.max_depth.unwrap_or(4);
    let files = walk_files(root_path);
    let files: Vec<_> = files.into_iter().filter(|p| {
        p.strip_prefix(root_path).ok().map(|r| r.components().count() <= max_depth).unwrap_or(true)
    }).collect();
    let rels: Vec<String> = files.iter().filter_map(|p| p.strip_prefix(root_path).ok().map(|r| r.to_string_lossy().to_string())).collect();
    if let Some(c) = &cache { let _ = c.prune(&rels); }
    let mut entries: Vec<String> = Vec::new();
    let results: Vec<String> = files.iter().map(|p| {
        let lang = match language_for(p) { Some(l) => l, None => return String::new() };
        let source = match std::fs::read_to_string(p) { Ok(s) => s, Err(_) => return String::new() };
        let rel = p.strip_prefix(root_path).unwrap_or(p).to_string_lossy().to_string();
        let mtime = crate::core::cache::mtime_ns(p);
        let byte_len = source.len() as i64;
        let (skel, _syms) = if let Some(c) = &cache {
            if let Some((s, j)) = c.get(&rel, mtime, byte_len) {
                (s, j)
            } else {
                let syms = parse_file(p, &source);
                let s = skeleton_text(p, &source, &syms);
                let j = serde_json::to_string(&syms).unwrap_or_default();
                let _ = c.put(&rel, mtime, byte_len, lang, &s, &j);
                (s, j)
            }
        } else {
            let syms = parse_file(p, &source);
            let s = skeleton_text(p, &source, &syms);
            let j = serde_json::to_string(&syms).unwrap_or_default();
            (s, j)
        };
        skel
    }).collect();
    for r in results { if !r.is_empty() { entries.push(r); } }
    let mut text = entries.join("\n");
    if text.len() > max_chars { text.truncate(max_chars); text.push_str("\n...[truncated]"); }
    Ok(ToolOutput { text })
}
