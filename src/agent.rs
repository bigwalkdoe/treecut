#[cfg(feature = "cli")]
use reqwest::Client;
use serde_json::{Value, json};
use crate::core;

pub async fn run_ask(prompt: &str, path: &str, model: &str, base_url: &str) {
    let client = Client::new();
    let mut messages = vec![json!({"role":"user","content":prompt})];
    for _ in 0..6 {
        let body = json!({
            "model": model,
            "messages": messages,
            "tools": [
                {"type":"function","function":{"name":"repo_skeleton","parameters":{"type":"object","properties":{"root":{"type":"string"}}}}},
                {"type":"function","function":{"name":"slice_symbol","parameters":{"type":"object","properties":{"root":{"type":"string"},"symbol":{"type":"string"}},"required":["symbol"]}}},
                {"type":"function","function":{"name":"slice_symbols","parameters":{"type":"object","properties":{"root":{"type":"string"},"symbols":{"type":"array","items":{"type":"string"}}},"required":["symbols"]}}},
                {"type":"function","function":{"name":"slice_changed_symbols","parameters":{"type":"object","properties":{"root":{"type":"string"}}}}},
                {"type":"function","function":{"name":"read_node","parameters":{"type":"object","properties":{"path":{"type":"string"},"start_line":{"type":"integer"},"end_line":{"type":"integer"}},"required":["path","start_line","end_line"]}}}
            ]
        });
        let resp = client
            .post(format!(
                "{}/chat/completions",
                base_url.trim_end_matches('/')
            ))
            .json(&body)
            .send()
            .await;
        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                eprintln!("err {}", e);
                return;
            }
        };
        let v: Value = resp.json().await.unwrap_or(json!({}));
        let msg = v["choices"][0]["message"].clone();
        messages.push(msg.clone());
        if let Some(calls) = msg.get("tool_calls") {
            for call in calls.as_array().cloned().unwrap_or_default() {
                let name = call["function"]["name"].as_str().unwrap_or("");
                let args: Value =
                    serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}"))
                        .unwrap_or(json!({}));
                let result = execute_tool(name, &args, path);
                messages.push(json!({"role":"tool","tool_call_id":call["id"],"content":result}));
            }
            continue;
        }
        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
            println!("{}", content);
        }
        return;
    }
}

fn execute_tool(name: &str, args: &Value, default_root: &str) -> String {
    let root = args
        .get("root")
        .and_then(|v| v.as_str())
        .unwrap_or(default_root)
        .to_string();
    match name {
        "repo_skeleton" => {
            let a = core::RepoSkeletonArgs {
                root: Some(root),
                ..Default::default()
            };
            core::repo_skeleton(&a).map(|r| r.text).unwrap_or_default()
        }
        "slice_symbol" => {
            let sym = args
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let a = core::SliceSymbolArgs {
                root: Some(root),
                symbol: sym,
                ..Default::default()
            };
            core::slice_symbol(&a).map(|r| r.text).unwrap_or_default()
        }
        "slice_symbols" => {
            let symbols = args
                .get("symbols")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let a = core::SliceSymbolsArgs {
                root: Some(root),
                symbols,
                ..Default::default()
            };
            core::slice_symbols(&a).map(|r| r.text).unwrap_or_default()
        }
        "slice_changed_symbols" => {
            let a = core::SliceChangedArgs {
                root: Some(root),
                ..Default::default()
            };
            core::slice_changed_symbols(&a)
                .map(|r| r.text)
                .unwrap_or_default()
        }
        "read_node" => {
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
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
    }
}
