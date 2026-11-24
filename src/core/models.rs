use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct RepoSkeletonArgs {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub max_chars: Option<usize>,
    #[serde(default)]
    pub no_cache: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ImportAlias {
    /// Original exported symbol name (e.g., "verify_token")
    pub original: String,
    /// Local alias used in this file (e.g., "check_tok")
    pub alias: String,
    /// Optional module path hint (e.g., "crate::auth" or "./auth")
    pub source_module: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ParsedFile {
    pub symbols: Vec<SymbolInfo>,
    pub imports: Vec<ImportAlias>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct SliceSymbolArgs {
    #[serde(default)]
    pub root: Option<String>,
    pub symbol: String,
    #[serde(default)]
    pub context_lines: Option<usize>,
    #[serde(default)]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct SliceChangedArgs {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub base_ref: Option<String>,
    #[serde(default)]
    pub include_callers: Option<bool>,
    #[serde(default)]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReadNodeArgs {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub text: String,
}
