use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::tempdir;
use treecut::core;

#[test]
fn sqlite_mtime_cache_invalidation() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.rs"), "pub fn foo() {}\n").unwrap();
    let args = core::RepoSkeletonArgs { root: Some(root.to_str().unwrap().into()), ..Default::default() };
    let _ = core::repo_skeleton(&args).unwrap();
    // modify file mtime by rewriting
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(root.join("a.rs"), "pub fn foo() { let x = 1; }\n").unwrap();
    let r = core::repo_skeleton(&args).unwrap();
    assert!(r.text.contains("foo"));
}

#[test]
fn git2_diff_slice() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    // init git repo
    let repo = git2::Repository::init(root).unwrap();
    std::fs::write(root.join("lib.rs"), "pub fn add(a:i32,b:i32)->i32 { a+b }\n").unwrap();
    // stage & commit
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("lib.rs")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("t","t@t.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
    // modify
    std::fs::write(root.join("lib.rs"), "pub fn add(a:i32,b:i32)->i32 { a*b }\n").unwrap();
    let args = core::SliceChangedArgs { root: Some(root.to_str().unwrap().into()), ..Default::default() };
    let r = core::slice_changed_symbols(&args).unwrap();
    assert!(r.text.contains("add"));
}

#[test]
fn jsonrpc_stdio_handshake() {
    let bin = env!("CARGO_BIN_EXE_treecut");
    let mut child = Command::new(bin)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{}}}}"#).unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("repo_skeleton"));
    assert!(stdout.contains("slice_symbol"));
    assert!(stdout.contains("slice_changed_symbols"));
    assert!(stdout.contains("read_node"));
}

// New Tier-2 language tests

fn fixture_path(lang: &str, file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(lang)
        .join(file)
}

#[test]
fn repo_skeleton_javascript() {
    let path = fixture_path("javascript", "main.js");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("greet"));
    assert!(skeleton.contains("Animal"));
    assert!(skeleton.contains("arrowFn"));
}

#[test]
fn repo_skeleton_csharp() {
    let path = fixture_path("csharp", "Main.cs");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("Animal"));
    assert!(skeleton.contains("IShape"));
    assert!(skeleton.contains("Point"));
    assert!(skeleton.contains("Color"));
    assert!(skeleton.contains("Person"));
    assert!(skeleton.contains("Add"));
}

#[test]
fn repo_skeleton_ruby() {
    let path = fixture_path("ruby", "main.rb");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("greet"));
    assert!(skeleton.contains("Animal"));
    assert!(skeleton.contains("Walkable"));
    assert!(skeleton.contains("Person"));
}

#[test]
fn repo_skeleton_php() {
    let path = fixture_path("php", "main.php");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("greet"));
    assert!(skeleton.contains("Animal"));
    assert!(skeleton.contains("Shape"));
    assert!(skeleton.contains("Walkable"));
    assert!(skeleton.contains("Status"));
}

#[test]
fn repo_skeleton_swift() {
    let path = fixture_path("swift", "main.swift");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("greet"));
    assert!(skeleton.contains("Animal"));
    assert!(skeleton.contains("Point"));
    assert!(skeleton.contains("Shape"));
    assert!(skeleton.contains("Color"));
}

#[test]
fn repo_skeleton_bash() {
    let path = fixture_path("bash", "main.sh");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let skeleton = core::parser::skeleton_text(&path, &source, &symbols);
    assert!(skeleton.contains("greet"));
    assert!(skeleton.contains("main"));
}

#[test]
fn slice_symbol_javascript() {
    let path = fixture_path("javascript", "main.js");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let greet = symbols.iter().find(|s| s.name == "greet").unwrap();
    let args = core::SliceSymbolArgs { root: Some(path.parent().unwrap().to_str().unwrap().to_string()), symbol: greet.name.clone(), ..Default::default() };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.text.contains("greet"));
}

#[test]
fn slice_symbol_csharp() {
    let path = fixture_path("csharp", "Main.cs");
    let source = std::fs::read_to_string(&path).unwrap();
    let symbols = core::parser::parse_file(&path, &source);
    let animal = symbols.iter().find(|s| s.name == "Animal").unwrap();
    let args = core::SliceSymbolArgs { root: Some(path.parent().unwrap().to_str().unwrap().to_string()), symbol: animal.name.clone(), ..Default::default() };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.text.contains("Animal"));
}
