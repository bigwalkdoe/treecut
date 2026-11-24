use crate::core::models::SymbolInfo;
use std::path::Path;
use tree_sitter::{Language, Parser};

fn lang_for(language: &str) -> Option<Language> {
    match language {
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "typescript" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
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
        _ => &[],
    }
}

pub fn parse_file(path: &Path, source: &str) -> Vec<SymbolInfo> {
    let language = match crate::core::walker::language_for(path) { Some(l) => l, None => return Vec::new() };
    let ts_lang = match lang_for(language) { Some(l) => l, None => return Vec::new() };
    let mut parser = Parser::new();
    if parser.set_language(&ts_lang).is_err() { return Vec::new(); }
    let tree = match parser.parse(source, None) { Some(t) => t, None => return Vec::new() };
    let kinds = symbol_node_kinds(language);
    let mut out = Vec::new();
    collect(&tree.root_node(), source, path, kinds, &mut out);
    out
}

fn collect(node: &tree_sitter::Node, source: &str, path: &Path, kinds: &[&str], out: &mut Vec<SymbolInfo>) {
    if kinds.contains(&node.kind()) {
        let name = node.child_by_field_name("name")
            .or_else(|| node.child_by_field_name("type"))
            .map(|n| source[n.byte_range()].to_string())
            .unwrap_or_else(|| node.kind().to_string());
        out.push(SymbolInfo {
            name,
            kind: node.kind().to_string(),
            file: path.to_string_lossy().to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(&child, source, path, kinds, out);
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
