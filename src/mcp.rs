use serde_json::{json, Value};
use std::io::{BufRead, Write};
use crate::core;

pub async fn run() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        if method == "notifications/initialized" {
            continue;
        }
        let resp = if method == "initialize" {
            json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"treecut","version":"0.3.0"}}})
        } else if method == "tools/list" {
            json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
                {"name":"repo_skeleton","description":"repo skeleton","inputSchema":{"type":"object"}},
                {"name":"slice_symbol","description":"slice symbol","inputSchema":{"type":"object"}},
                {"name":"slice_symbols","description":"slice multiple symbols in batch","inputSchema":{"type":"object"}},
                {"name":"slice_changed_symbols","description":"slice changed","inputSchema":{"type":"object"}},
                {"name":"read_node","description":"read node","inputSchema":{"type":"object"}}
            ]}})
        } else if method == "tools/call" {
            let tool_name = req.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
            let args = req.get("params").and_then(|p| p.get("arguments")).cloned().unwrap_or(json!({}));
            let result = match tool_name {
                "repo_skeleton" => {
                    let a = core::RepoSkeletonArgs {
                        root: args.get("root").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        max_depth: args.get("max_depth").and_then(|v| v.as_u64()).map(|n| n as usize),
                        max_chars: args.get("max_chars").and_then(|v| v.as_u64()).map(|n| n as usize),
                        no_cache: args.get("no_cache").and_then(|v| v.as_bool()),
                        stats: args.get("stats").and_then(|v| v.as_bool()),
                        scope: args.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        include: args.get("include").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
                    };
                    core::repo_skeleton(&a).map(|r| r.text).unwrap_or_default()
                }
                "slice_symbol" => {
                    let sym = args.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let a = core::SliceSymbolArgs {
                        root: args.get("root").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        symbol: sym,
                        context_lines: args.get("context_lines").and_then(|v| v.as_u64()).map(|n| n as usize),
                        max_chars: args.get("max_chars").and_then(|v| v.as_u64()).map(|n| n as usize),
                        stats: args.get("stats").and_then(|v| v.as_bool()),
                        depth: args.get("depth").and_then(|v| v.as_u64()).map(|n| n as usize),
                        direction: args.get("direction").and_then(|v| v.as_str()).and_then(|s| match s.to_lowercase().as_str() { "callers" => Some(core::models::TraversalDirection::Callers), "callees" => Some(core::models::TraversalDirection::Callees), "both" => Some(core::models::TraversalDirection::Both), _ => None }),
                        scope: args.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        include: args.get("include").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
                    };
                    core::slice_symbol(&a).map(|r| r.text).unwrap_or_default()
                }
                "slice_symbols" => {
                    let symbols = args.get("symbols").and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                        .unwrap_or_default();
                    let a = core::SliceSymbolsArgs {
                        root: args.get("root").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        symbols,
                        context_lines: args.get("context_lines").and_then(|v| v.as_u64()).map(|n| n as usize),
                        max_chars: args.get("max_chars").and_then(|v| v.as_u64()).map(|n| n as usize),
                        stats: args.get("stats").and_then(|v| v.as_bool()),
                        depth: args.get("depth").and_then(|v| v.as_u64()).map(|n| n as usize),
                        direction: args.get("direction").and_then(|v| v.as_str()).and_then(|s| match s.to_lowercase().as_str() { "callers" => Some(core::models::TraversalDirection::Callers), "callees" => Some(core::models::TraversalDirection::Callees), "both" => Some(core::models::TraversalDirection::Both), _ => None }),
                        scope: args.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        include: args.get("include").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
                    };
                    core::slice_symbols(&a).map(|r| r.text).unwrap_or_default()
                }
                "slice_changed_symbols" => {
                    let a = core::SliceChangedArgs {
                        root: args.get("root").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        base_ref: args.get("base_ref").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        include_callers: args.get("include_callers").and_then(|v| v.as_bool()),
                        max_chars: args.get("max_chars").and_then(|v| v.as_u64()).map(|n| n as usize),
                        stats: args.get("stats").and_then(|v| v.as_bool()),
                        depth: args.get("depth").and_then(|v| v.as_u64()).map(|n| n as usize),
                        direction: args.get("direction").and_then(|v| v.as_str()).and_then(|s| match s.to_lowercase().as_str() { "callers" => Some(core::models::TraversalDirection::Callers), "callees" => Some(core::models::TraversalDirection::Callees), "both" => Some(core::models::TraversalDirection::Both), _ => None }),
                        scope: args.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        include: args.get("include").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
                    };
                    core::slice_changed_symbols(&a).map(|r| r.text).unwrap_or_default()
                }
                "read_node" => {
                    let p = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let s = args.get("start_line").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                    let e = args.get("end_line").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                    let a = core::ReadNodeArgs {
                        path: p,
                        start_line: s,
                        end_line: e,
                    };
                    core::read_node(&a).map(|r| r.text).unwrap_or_default()
                }
                _ => String::new(),
            };
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":result}]}})
        } else {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}})
        };
        let _ = writeln!(stdout, "{}", resp);
        let _ = stdout.flush();
    }
}
