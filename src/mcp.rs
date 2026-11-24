use serde_json::{json, Value};
use std::io::{BufRead, Write};

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
            json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"treecut","version":"0.1.0"}}})
        } else if method == "tools/list" {
            json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
                {"name":"repo_skeleton","description":"repo skeleton","inputSchema":{"type":"object"}},
                {"name":"slice_symbol","description":"slice symbol","inputSchema":{"type":"object"}},
                {"name":"slice_changed_symbols","description":"slice changed","inputSchema":{"type":"object"}},
                {"name":"read_node","description":"read node","inputSchema":{"type":"object"}}
            ]}})
        } else {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}})
        };
        let _ = writeln!(stdout, "{}", resp);
        let _ = stdout.flush();
    }
}
