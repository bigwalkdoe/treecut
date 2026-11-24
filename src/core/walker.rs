use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkBuilder::new(root).hidden(true).git_ignore(true).build() {
        if let Ok(e) = entry {
            if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                out.push(e.path().to_path_buf());
            }
        }
    }
    out
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
        _ => None,
    }
}
