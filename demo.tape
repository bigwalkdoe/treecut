# Demo recording script for treecut
# Install vhs: https://github.com/charmbracelet/vhs
# Run: vhs demo.tape

Output demo.gif
Set Theme "dracula"
Set FontSize 14
Set Width 1000
Set Height 600

# Setup a sample repo
Type "mkdir -p /tmp/treecut-demo && cd /tmp/treecut-demo"
Sleep 500ms
Type "git init -q"
Sleep 200ms
Type "cat > main.rs << 'EOF'\npub fn validate_token(token: &str) -> bool {\n    token.len() > 8\n}\n\npub fn authenticate(user: &str, token: &str) -> Result<String, String> {\n    if validate_token(token) {\n        Ok(format!(\"Welcome, {}\", user))\n    } else {\n        Err(\"Invalid token\".into())\n    }\n}\n\npub fn main() {\n    let result = authenticate(\"alice\", \"secret123\");\n    println!(\"{:?}\", result);\n}\nEOF"
Sleep 500ms
Type "cat > lib.rs << 'EOF'\nuse crate::main::{validate_token, authenticate};\n\npub fn login(user: &str, token: &str) -> bool {\n    authenticate(user, token).is_ok()\n}\nEOF"
Sleep 500ms
Type "git add . && git commit -m 'initial' -q"
Sleep 500ms

# Demo treecut map
Type "treecut map --depth 3 --max-chars 8000"
Sleep 2000ms
Enter

# Demo treecut slice
Type "treecut slice validate_token"
Sleep 2000ms
Enter

# Modify a file and demo diff-slice
Type "sed -i 's/token.len() > 8/token.len() >= 10/' main.rs"
Sleep 300ms
Type "treecut diff-slice"
Sleep 2000ms
Enter

# Demo slice_changed_symbols with base ref
Type "treecut diff-slice --base HEAD~1"
Sleep 2000ms
Enter