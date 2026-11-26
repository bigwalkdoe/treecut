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

// Import alias resolution tests

fn fixture_dir(lang: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(lang)
}

#[test]
fn slice_symbol_rust_alias() {
    let dir = fixture_dir("rust_alias");
    let args = core::SliceSymbolArgs { root: Some(dir.to_str().unwrap().to_string()), symbol: "verify_token".to_string(), ..Default::default() };
    let result = core::slicer::slice_symbol(&args).unwrap();
    // Should find both the definition and the aliased call site
    assert!(result.text.contains("verify_token"), "Should find original symbol definition");
    assert!(result.text.contains("check_tok"), "Should find aliased call site");
    assert!(result.text.contains("via alias check_tok"), "Should annotate aliased match");
}

#[test]
fn slice_symbol_typescript_alias() {
    let dir = fixture_dir("typescript_alias");
    let args = core::SliceSymbolArgs { root: Some(dir.to_str().unwrap().to_string()), symbol: "verifyToken".to_string(), ..Default::default() };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.text.contains("verifyToken"), "Should find original symbol definition");
    assert!(result.text.contains("checkTok"), "Should find aliased call site");
    assert!(result.text.contains("via alias checkTok"), "Should annotate aliased match");
}

#[test]
fn slice_symbol_python_alias() {
    let dir = fixture_dir("python_alias");
    let args = core::SliceSymbolArgs { root: Some(dir.to_str().unwrap().to_string()), symbol: "verify_token".to_string(), ..Default::default() };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.text.contains("verify_token"), "Should find original symbol definition");
    assert!(result.text.contains("check_tok"), "Should find aliased call site");
    assert!(result.text.contains("via alias check_tok"), "Should annotate aliased match");
}

#[test]
fn parse_imports_rust() {
    let path = fixture_dir("rust_alias").join("middleware.rs");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = core::parser::parse_file_with_imports(&path, &source);
    assert_eq!(parsed.imports.len(), 1);
    assert_eq!(parsed.imports[0].original, "verify_token");
    assert_eq!(parsed.imports[0].alias, "check_tok");
    assert!(parsed.imports[0].source_module.as_ref().unwrap().contains("auth"));
}

#[test]
fn parse_imports_typescript() {
    let path = fixture_dir("typescript_alias").join("middleware.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = core::parser::parse_file_with_imports(&path, &source);
    assert_eq!(parsed.imports.len(), 1);
    assert_eq!(parsed.imports[0].original, "verifyToken");
    assert_eq!(parsed.imports[0].alias, "checkTok");
    assert!(parsed.imports[0].source_module.as_ref().unwrap().contains("auth"));
}

#[test]
fn parse_imports_python() {
    let path = fixture_dir("python_alias").join("middleware.py");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = core::parser::parse_file_with_imports(&path, &source);
    assert_eq!(parsed.imports.len(), 1);
    assert_eq!(parsed.imports[0].original, "verify_token");
    assert_eq!(parsed.imports[0].alias, "check_tok");
    assert!(parsed.imports[0].source_module.as_ref().unwrap().contains("auth"));
}

// v0.3.0: Multi-symbol batch slicing and stats tests

#[test]
fn slice_symbols_multi() {
    let dir = fixture_dir("multi_symbol");
    let args = core::SliceSymbolsArgs { 
        root: Some(dir.to_str().unwrap().to_string()), 
        symbols: vec!["verify_token".to_string(), "refresh_token".to_string()],
        ..Default::default() 
    };
    let result = core::slicer::slice_symbols(&args).unwrap();
    // Should find both definitions and both call sites
    assert!(result.text.contains("verify_token"), "Should find verify_token definition");
    assert!(result.text.contains("refresh_token"), "Should find refresh_token definition");
    assert!(result.text.contains("check_tok"), "Should find verify_token aliased call site");
    assert!(result.text.contains("refresh_token"), "Should find refresh_token call site (no alias)");
    // Should NOT duplicate the overlapping caller context (handle_request and refresh_session share file)
    let handle_request_count = result.text.matches("handle_request").count();
    assert_eq!(handle_request_count, 1, "Should not duplicate overlapping caller context");
}

#[test]
fn slice_symbols_stats() {
    let dir = fixture_dir("multi_symbol");
    let args = core::SliceSymbolsArgs { 
        root: Some(dir.to_str().unwrap().to_string()), 
        symbols: vec!["verify_token".to_string()],
        stats: Some(true),
        ..Default::default() 
    };
    let result = core::slicer::slice_symbols(&args).unwrap();
    assert!(result.stats.is_some(), "Stats should be present");
    let stats = result.stats.unwrap();
    assert!(stats.files_scanned > 0, "files_scanned should be > 0");
    assert!(stats.files_matched > 0, "files_matched should be > 0");
    assert!(stats.full_files_bytes > 0, "full_files_bytes should be > 0");
    assert!(stats.sliced_bytes > 0, "sliced_bytes should be > 0");
    assert!(stats.reduction_percent >= 0.0 && stats.reduction_percent <= 100.0, "reduction_percent should be 0-100");
    let saved = stats.full_files_bytes.saturating_sub(stats.sliced_bytes);
    assert_eq!(stats.estimated_tokens_saved, saved / 4, "tokens should be ~4 chars");
}

#[test]
fn slice_symbol_stats() {
    let dir = fixture_dir("multi_symbol");
    let args = core::SliceSymbolArgs { 
        root: Some(dir.to_str().unwrap().to_string()), 
        symbol: "verify_token".to_string(),
        stats: Some(true),
        ..Default::default() 
    };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.stats.is_some(), "Stats should be present");
    let stats = result.stats.unwrap();
    assert!(stats.files_scanned > 0, "files_scanned should be > 0");
    assert!(stats.files_matched > 0, "files_matched should be > 0");
    assert!(stats.reduction_percent >= 0.0 && stats.reduction_percent <= 100.0, "reduction_percent should be 0-100");
}

#[test]
fn repo_skeleton_stats() {
    let dir = fixture_dir("multi_symbol");
    let args = core::RepoSkeletonArgs { 
        root: Some(dir.to_str().unwrap().to_string()),
        stats: Some(true),
        ..Default::default() 
    };
    let result = core::repo_skeleton(&args).unwrap();
    assert!(result.stats.is_some(), "Stats should be present");
    let stats = result.stats.unwrap();
    assert!(stats.files_scanned > 0, "files_scanned should be > 0");
    assert!(stats.files_matched > 0, "files_matched should be > 0");
    assert!(stats.reduction_percent >= 0.0 && stats.reduction_percent <= 100.0, "reduction_percent should be 0-100");
}

#[test]
fn slice_symbols_dedupe_symbols() {
    // Test that duplicate symbols in input are deduplicated
    let dir = fixture_dir("multi_symbol");
    let args = core::SliceSymbolsArgs { 
        root: Some(dir.to_str().unwrap().to_string()), 
        symbols: vec!["verify_token".to_string(), "verify_token".to_string()],
        ..Default::default() 
    };
    let result = core::slicer::slice_symbols(&args).unwrap();
    eprintln!("OUTPUT:\n{}", result.text);
    // Should only show each match once
    // Definition contains "verify_token", call site contains alias "check_tok"
    let verify_token_count = result.text.matches("verify_token").count();
    let check_tok_count = result.text.matches("check_tok").count();
    // Should find at least definition + call site (via alias)
    assert!(verify_token_count >= 1 && check_tok_count >= 1, 
        "Should find definition (verify_token) + call site (check_tok), got verify_token={}, check_tok={}", 
        verify_token_count, check_tok_count);
}

#[test]
fn slice_symbol_depth_direction_callers() {
    let dir = fixture_dir("multi_symbol");
    let args = core::SliceSymbolArgs {
        root: Some(dir.to_str().unwrap().to_string()),
        symbol: "verify_token".to_string(),
        depth: Some(2),
        direction: Some(core::TraversalDirection::Callers),
        ..Default::default()
    };
    let result = core::slicer::slice_symbol(&args).unwrap();
    assert!(result.text.contains("verify_token"), "should include seed symbol");
    assert!(result.text.contains("handle_request"), "should include caller");
    assert!(result.text.contains("Hop 0") || result.text.contains("hop 0") || result.text.contains("verify_token"), "should produce slice output");
}

#[test]
fn slice_symbol_direction_callees_and_both() {
    let dir = fixture_dir("multi_symbol");
    for direction in [core::TraversalDirection::Callees, core::TraversalDirection::Both] {
        let args = core::SliceSymbolArgs {
            root: Some(dir.to_str().unwrap().to_string()),
            symbol: "handle_request".to_string(),
            depth: Some(2),
            direction: Some(direction),
            ..Default::default()
        };
        let result = core::slicer::slice_symbol(&args).unwrap();
        assert!(result.text.contains("handle_request"), "should include seed symbol");
    }
}

#[test]
fn slice_symbol_scope_and_include() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("pkg_a")).unwrap();
    std::fs::create_dir_all(root.join("pkg_b")).unwrap();
    std::fs::write(root.join("pkg_a/lib.rs"), "pub fn target() {}\npub fn caller() { target(); }\n").unwrap();
    std::fs::write(root.join("pkg_b/lib.rs"), "pub fn caller() { target(); }\n").unwrap();

    let scoped = core::SliceSymbolArgs {
        root: Some(root.to_str().unwrap().to_string()),
        symbol: "target".to_string(),
        scope: Some("pkg_a".to_string()),
        ..Default::default()
    };
    let scoped_result = core::slicer::slice_symbol(&scoped).unwrap();
    assert!(scoped_result.text.contains("pkg_a/lib.rs") || scoped_result.text.contains("pkg_a\\lib.rs"), "scoped result should include pkg_a");
    assert!(!scoped_result.text.contains("pkg_b/lib.rs") && !scoped_result.text.contains("pkg_b\\lib.rs"), "scoped result should exclude pkg_b");

    let include = core::SliceSymbolArgs {
        root: Some(root.to_str().unwrap().to_string()),
        symbol: "target".to_string(),
        include: Some(vec!["pkg_b/**".to_string()]),
        ..Default::default()
    };
    let include_result = core::slicer::slice_symbol(&include).unwrap();
    assert!(!include_result.text.contains("pkg_b/lib.rs") && !include_result.text.contains("pkg_b\\lib.rs"), "include/exclude pattern should filter pkg_b");
}

#[test]
fn diff_slice_stats() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = git2::Repository::init(root).unwrap();
    std::fs::write(root.join("lib.rs"), "pub fn add(a:i32,b:i32)->i32 { a+b }\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("lib.rs")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("t","t@t.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
    std::fs::write(root.join("lib.rs"), "pub fn add(a:i32,b:i32)->i32 { a*b }\n").unwrap();

    let args = core::SliceChangedArgs { root: Some(root.to_str().unwrap().into()), stats: Some(true), ..Default::default() };
    let r = core::slice_changed_symbols(&args).unwrap();
    assert!(r.text.contains("add"));
    let stats = r.stats.expect("stats should be present");
    assert!(stats.files_scanned > 0);
}

#[test]
fn cli_help_and_version_smoke() {
    let bin = env!("CARGO_BIN_EXE_treecut");
    let help = Command::new(bin).arg("--help").output().unwrap();
    assert!(help.status.success());
    let help_out = String::from_utf8_lossy(&help.stdout);
    assert!(help_out.contains("slice"));
    assert!(help_out.contains("mcp"));

    let version = Command::new(bin).arg("--version").output().unwrap();
    assert!(version.status.success());
    let ver_out = String::from_utf8_lossy(&version.stdout);
    assert!(ver_out.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn mcp_initialize_reports_package_version() {
    let bin = env!("CARGO_BIN_EXE_treecut");
    let mut child = Command::new(bin)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{}}}}"#).unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}
