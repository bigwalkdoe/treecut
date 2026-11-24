use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn walk_files(root: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn language_for(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "go" => Some("go"),
        "java" => Some("java"),
        "c" | "h" => Some("c"),
        "cpp" | "hpp" | "cc" | "cxx" => Some("cpp"),
        // New Tier-2 languages
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "cs" => Some("csharp"),
        "rb" | "rake" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "sh" | "bash" => Some("bash"),
        _ => None,
    }
}
