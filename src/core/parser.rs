use crate::core::models::{SymbolInfo, ImportAlias, ParsedFile};
use std::path::Path;
use tree_sitter::{Language, Node, Parser};

pub fn lang_for(language: &str) -> Option<Language> {
    match language {
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "typescript" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        #[cfg(feature = "extra-langs")]
        "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
        #[cfg(feature = "extra-langs")]
        "csharp" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        #[cfg(feature = "extra-langs")]
        "ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        #[cfg(feature = "extra-langs")]
        "php" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        #[cfg(feature = "extra-langs")]
        "swift" => Some(tree_sitter_swift::LANGUAGE.into()),
        #[cfg(feature = "extra-langs")]
        "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
        _ => None,
    }
}

fn symbol_node_kinds(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["function_item", "struct_item", "enum_item", "trait_item", "impl_item", "type_item"],
        "python" => &["function_definition", "class_definition"],
        "typescript" | "tsx" => &["function_declaration", "class_declaration", "method_definition", "interface_declaration", "type_alias_declaration"],
        "go" => &["function_declaration", "method_declaration", "type_declaration"],
        "java" => &["class_declaration", "interface_declaration", "enum_declaration", "method_declaration", "constructor_declaration"],
        "c" | "cpp" => &["function_definition", "struct_specifier", "class_specifier", "enum_specifier"],
        #[cfg(feature = "extra-langs")]
        "javascript" => &["function_declaration", "class_declaration", "method_definition", "lexical_declaration"],
        #[cfg(feature = "extra-langs")]
        "csharp" => &["class_declaration", "struct_declaration", "interface_declaration", "enum_declaration", "record_declaration", "method_declaration", "constructor_declaration"],
        #[cfg(feature = "extra-langs")]
        "ruby" => &["class", "module", "method", "singleton_method"],
        #[cfg(feature = "extra-langs")]
        "php" => &["class_declaration", "interface_declaration", "trait_declaration", "enum_declaration", "function_definition", "method_declaration"],
        #[cfg(feature = "extra-langs")]
        "swift" => &["class_declaration", "struct_declaration", "protocol_declaration", "enum_declaration", "function_declaration", "init_declaration"],
        #[cfg(feature = "extra-langs")]
        "bash" => &["function_definition"],
        _ => &[],
    }
}

fn import_node_kinds(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["use_declaration"],
        "python" => &["import_statement", "import_from_statement"],
        "typescript" | "tsx" => &["import_statement"],
        #[cfg(feature = "extra-langs")]
        "javascript" => &["import_statement"],
        "go" => &["import_declaration"],
        #[cfg(feature = "extra-langs")]
        "csharp" => &["using_directive"],
        _ => &[],
    }
}

pub fn parse_file(path: &Path, source: &str) -> Vec<SymbolInfo> {
    let parsed = parse_file_with_imports(path, source);
    parsed.symbols
}

pub fn parse_file_with_imports(path: &Path, source: &str) -> ParsedFile {
    let language = match crate::core::walker::language_for(path) { Some(l) => l, None => return ParsedFile::default() };
    let ts_lang = match lang_for(language) { Some(l) => l, None => return ParsedFile::default() };
    let mut parser = Parser::new();
    if parser.set_language(&ts_lang).is_err() { return ParsedFile::default(); }
    let tree = match parser.parse(source, None) { Some(t) => t, None => return ParsedFile::default() };
    let sym_kinds = symbol_node_kinds(language);
    let imp_kinds = import_node_kinds(language);
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    collect_symbols(&tree.root_node(), source, path, sym_kinds, &mut symbols);
    collect_imports(&tree.root_node(), source, path, language, imp_kinds, &mut imports);
    ParsedFile { symbols, imports }
}

fn collect_symbols(node: &Node, source: &str, path: &Path, kinds: &[&str], out: &mut Vec<SymbolInfo>) {
    if kinds.iter().any(|k| *k == node.kind()) {
        let name = node.child_by_field_name("name")
            .or_else(|| node.child_by_field_name("type"))
            .map(|n| source[n.byte_range()].to_string())
            .unwrap_or_else(|| node.kind().to_string());
        
        // Find enclosing symbol (parent function/class)
        let enclosing_symbol = find_enclosing_symbol(&node, source, kinds);
        
        // Extract callees from call expressions within this symbol
        let callees = extract_callees(&node, source);
        
        out.push(SymbolInfo {
            name,
            kind: node.kind().to_string(),
            file: path.to_string_lossy().to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            enclosing_symbol,
            callees,
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_symbols(&child, source, path, kinds, out);
    }
}

// Find the nearest enclosing symbol (function/class) that contains this node
fn find_enclosing_symbol(node: &Node, source: &str, kinds: &[&str]) -> Option<String> {
    let mut parent = node.parent();
    while let Some(p) = parent {
        if kinds.iter().any(|k| *k == p.kind()) {
            if let Some(name_node) = p.child_by_field_name("name").or_else(|| p.child_by_field_name("type")) {
                return Some(source[name_node.byte_range()].to_string());
            }
        }
        parent = p.parent();
    }
    None
}

// Extract callees (function calls) from within a symbol's body
fn extract_callees(node: &Node, source: &str) -> Vec<String> {
    let mut callees = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call_expression" || child.kind() == "function_call" || child.kind() == "call" {
            if let Some(func_node) = child.child_by_field_name("function").or_else(|| child.child_by_field_name("name")) {
                let callee = source[func_node.byte_range()].to_string();
                if !callees.contains(&callee) {
                    callees.push(callee);
                }
            }
        }
        // Recursively check children
        callees.extend(extract_callees(&child, source));
    }
    callees
}

fn collect_imports(node: &Node, source: &str, path: &Path, language: &str, kinds: &[&str], out: &mut Vec<ImportAlias>) {
    if kinds.iter().any(|k| *k == node.kind()) {
        extract_imports(node, source, path, language, out);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_imports(&child, source, path, language, kinds, out);
    }
}

fn extract_imports(node: &Node, source: &str, _path: &Path, language: &str, out: &mut Vec<ImportAlias>) {
    match language {
        "rust" => extract_rust_imports(node, source, out),
        "python" => extract_python_imports(node, source, out),
        "typescript" | "tsx" => extract_ts_imports(node, source, out),
        "go" => extract_go_imports(node, source, out),
        #[cfg(feature = "extra-langs")]
        "javascript" => extract_ts_imports(node, source, out),
        #[cfg(feature = "extra-langs")]
        "csharp" => extract_csharp_imports(node, source, out),
        _ => {}
    }
}

fn extract_rust_imports(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
    // use crate::auth::verify_token as check_tok;
    // Matches use_as_clause anywhere in the use_declaration (handles grouped imports)
    fn walk_use(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
        if node.kind() == "use_as_clause" {
            let path_node = node.child_by_field_name("path");
            let alias_node = node.child_by_field_name("alias");
            if let (Some(p), Some(a)) = (path_node, alias_node) {
                let path_text = source[p.byte_range()].to_string();
                let alias_text = source[a.byte_range()].to_string();
                let original = path_text.split("::").last().unwrap_or(&path_text).to_string();
                out.push(ImportAlias {
                    original,
                    alias: alias_text,
                    source_module: Some(path_text),
                });
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk_use(&child, source, out);
        }
    }
    walk_use(node, source, out);
}

fn extract_python_imports(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
    // from foo import bar as baz
    // import foo as bar
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "aliased_import" {
            let name_node = child.child_by_field_name("name");
            let alias_node = child.child_by_field_name("alias");
            if let (Some(n), Some(a)) = (name_node, alias_node) {
                let name_text = source[n.byte_range()].to_string();
                let alias_text = source[a.byte_range()].to_string();
                let original = name_text.split('.').next_back().unwrap_or(&name_text).to_string();
                // Try to get module from parent
                let source_module = if node.kind() == "import_from_statement" {
                    node.child_by_field_name("module_name")
                        .map(|m| source[m.byte_range()].to_string())
                } else {
                    None
                };
                out.push(ImportAlias {
                    original,
                    alias: alias_text,
                    source_module,
                });
            }
        } else if child.kind() == "dotted_as_names" || child.kind() == "import_prefix" {
            // Handle import foo.bar as baz pattern
            let mut inner = child.walk();
            for c in child.children(&mut inner) {
                if c.kind() == "aliased_import" {
                    let name_node = c.child_by_field_name("name");
                    let alias_node = c.child_by_field_name("alias");
                    if let (Some(n), Some(a)) = (name_node, alias_node) {
                        let name_text = source[n.byte_range()].to_string();
                        let alias_text = source[a.byte_range()].to_string();
                        let original = name_text.split('.').next_back().unwrap_or(&name_text).to_string();
                        out.push(ImportAlias {
                            original,
                            alias: alias_text,
                            source_module: None,
                        });
                    }
                }
            }
        }
    }
}

fn extract_ts_imports(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
    // import { verify_token as checkTok } from './auth'
    // import verify_token from './auth' (default import)
    // Recursively search for import_specifier nodes
    fn visit_import_specifier(node: &Node, source: &str, source_module: Option<String>, out: &mut Vec<ImportAlias>) {
        if node.kind() == "import_specifier" {
            let mut spec_cursor = node.walk();
            let children: Vec<Node> = node.children(&mut spec_cursor).collect();
            if children.len() >= 3 && children[1].kind() == "as" {
                let name_text = source[children[0].byte_range()].to_string();
                let alias_text = source[children[2].byte_range()].to_string();
                out.push(ImportAlias {
                    original: name_text,
                    alias: alias_text,
                    source_module: source_module.clone(),
                });
            } else if children.len() == 1 {
                let name_text = source[children[0].byte_range()].to_string();
                out.push(ImportAlias {
                    original: name_text.clone(),
                    alias: name_text,
                    source_module: source_module.clone(),
                });
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit_import_specifier(&child, source, source_module.clone(), out);
        }
    }
    
    let source_module = node.child_by_field_name("source")
        .map(|s| source[s.byte_range()].to_string().trim_matches(&['\"', '\''][..]).to_string());
    visit_import_specifier(node, source, source_module, out);
}

fn extract_go_imports(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
    // import check_tok "github.com/user/auth"
    // or import ( "fmt" "github.com/user/auth" )
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import_spec" {
            // Check for alias (identifier before path)
            let mut spec_cursor = child.walk();
            let children: Vec<Node> = child.children(&mut spec_cursor).collect();
            if children.len() >= 2 {
                // First might be alias, last is path
                let path_node = children.last().unwrap();
                let alias_node = &children[0];
                let path_text = source[path_node.byte_range()].to_string().trim_matches('"').to_string();
                let alias_text = source[alias_node.byte_range()].to_string();
                if alias_text != path_text && !path_text.contains(&alias_text) {
                    // This is an aliased import
                    let original = path_text.split('/').next_back().unwrap_or(&path_text).to_string();
                    out.push(ImportAlias {
                        original,
                        alias: alias_text,
                        source_module: Some(path_text),
                    });
                }
            }
        }
    }
}

#[cfg(feature = "extra-langs")]
fn extract_csharp_imports(node: &Node, source: &str, out: &mut Vec<ImportAlias>) {
    // using CheckTok = Auth.VerifyToken;
    // Matches using_directive with name_equals
    if node.kind() == "using_directive" {
        let alias_node = node.child_by_field_name("alias");
        let target_node = node.child_by_field_name("target");
        if let (Some(a), Some(t)) = (alias_node, target_node) {
            let alias_text = source[a.byte_range()].to_string();
            let target_text = source[t.byte_range()].to_string();
            // Get last segment as original
            let original = target_text.split('.').next_back().unwrap_or(&target_text).to_string();
            out.push(ImportAlias {
                original,
                alias: alias_text,
                source_module: Some(target_text),
            });
        }
    }
}

pub fn skeleton_text(path: &Path, source: &str, symbols: &[SymbolInfo]) -> String {
    let mut s = String::new();
    s.push_str(&format!("// {}\n", path.display()));
    for sym in symbols {
        // extract first line signature
        let lines: Vec<&str> = source.lines().collect();
        if sym.start_line <= lines.len() {
            s.push_str(&format!("{}: {}\n", sym.start_line, lines[sym.start_line-1].trim()));
        }
    }
    s
}