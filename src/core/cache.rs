use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

pub struct CacheEntry {
    pub rel: String,
    pub mtime_ns: i64,
    pub byte_len: i64,
    pub language: String,
    pub skeleton: String,
    pub symbols_json: String,
    pub imports_json: String,
}

pub struct Cache {
    conn: Connection,
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
                symbols_json TEXT NOT NULL,
                imports_json TEXT NOT NULL DEFAULT '[]',
                schema_version INTEGER NOT NULL DEFAULT 3
            )",
            [],
        )?;
        // Migration: add imports_json column if it doesn't exist (for v0.1.0 caches)
        let _ = conn.execute(
            "ALTER TABLE file_index ADD COLUMN imports_json TEXT NOT NULL DEFAULT '[]'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE file_index ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 2",
            [],
        );
        // Migration: bump schema_version to 3 for v0.4.0 (new symbol fields)
        let _ = conn.execute(
            "ALTER TABLE file_index ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 3",
            [],
        );
        Ok(Self { conn })
    }

    pub fn get(&self, rel: &str, mtime_ns: i64, byte_len: i64) -> Option<(String, String, String)> {
        let mut stmt = self.conn.prepare("SELECT mtime_ns, byte_len, skeleton_text, symbols_json, imports_json FROM file_index WHERE rel_path=?1").ok()?;
        let row = stmt.query_row(params![rel], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?))
        }).ok()?;
        if row.0 == mtime_ns && row.1 == byte_len {
            Some((row.2, row.3, row.4))
        } else {
            None
        }
    }

    pub fn put(&self, entry: &CacheEntry) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO file_index (rel_path, mtime_ns, byte_len, language, skeleton_text, symbols_json, imports_json, schema_version) VALUES (?1,?2,?3,?4,?5,?6,?7,3)",
            params![
                entry.rel, entry.mtime_ns, entry.byte_len, entry.language,
                entry.skeleton, entry.symbols_json, entry.imports_json
            ],
        )?;
        Ok(())
    }

    pub fn prune(&self, existing: &[String]) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT rel_path FROM file_index")?;
        let rows: Vec<String> = stmt.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();
        let existing_set: std::collections::HashSet<_> = existing.iter().collect();
        for r in rows {
            if !existing_set.contains(&r) {
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
