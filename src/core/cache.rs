use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

pub struct Cache {
    conn: Connection,
    root: PathBuf,
}

impl Cache {
    pub fn open(root: &Path) -> Result<Self> {
        let treecut = root.join(".treecut");
        std::fs::create_dir_all(&treecut)?;
        let gitignore = treecut.join(".gitignore");
        if !gitignore.exists() {
            std::fs::write(&gitignore, "*\n")?;
        }
        let db = treecut.join("cache.db");
        let conn = Connection::open(&db)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_index (
                rel_path TEXT PRIMARY KEY,
                mtime_ns INTEGER NOT NULL,
                byte_len INTEGER NOT NULL,
                language TEXT NOT NULL,
                skeleton_text TEXT NOT NULL,
                symbols_json TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn, root: root.to_path_buf() })
    }

    pub fn get(&self, rel: &str, mtime_ns: i64, byte_len: i64) -> Option<(String, String)> {
        let mut stmt = self.conn.prepare("SELECT mtime_ns, byte_len, skeleton_text, symbols_json FROM file_index WHERE rel_path=?1").ok()?;
        let row = stmt.query_row(params![rel], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }).ok()?;
        if row.0 == mtime_ns && row.1 == byte_len {
            Some((row.2, row.3))
        } else {
            None
        }
    }

    pub fn put(&self, rel: &str, mtime_ns: i64, byte_len: i64, language: &str, skeleton: &str, symbols_json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO file_index (rel_path, mtime_ns, byte_len, language, skeleton_text, symbols_json) VALUES (?1,?2,?3,?4,?5,?6)",
            params![rel, mtime_ns, byte_len, language, skeleton, symbols_json],
        )?;
        Ok(())
    }

    pub fn prune(&self, existing: &[String]) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT rel_path FROM file_index")?;
        let rows: Vec<String> = stmt.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();
        for r in rows {
            if !existing.contains(&r) {
                self.conn.execute("DELETE FROM file_index WHERE rel_path=?1", params![r])?;
            }
        }
        Ok(())
    }
}

pub fn mtime_ns(path: &Path) -> i64 {
    std::fs::metadata(path).ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}
