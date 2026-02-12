use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRow {
    pub id: i64,
    pub package_id: i64,
    pub version: String,
    pub checksum: String,
    pub tarball_path: String,
    pub naze_files: i64,
    pub created_at: String,
}

impl Db {
    pub fn open(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = std::path::Path::new(data_dir).join("registry.db");
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS packages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                latest_version TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_id INTEGER NOT NULL REFERENCES packages(id),
                version TEXT NOT NULL,
                checksum TEXT NOT NULL,
                tarball_path TEXT NOT NULL,
                naze_files INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(package_id, version)
            );",
        )?;
        Ok(())
    }

    pub fn insert_package(
        &self,
        name: &str,
        description: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO packages (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_package(&self, name: &str) -> Result<Option<PackageRow>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, latest_version, created_at, updated_at FROM packages WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(PackageRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                latest_version: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn insert_version(
        &self,
        package_id: i64,
        version: &str,
        checksum: &str,
        tarball_path: &str,
        naze_files: i64,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO versions (package_id, version, checksum, tarball_path, naze_files) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![package_id, version, checksum, tarball_path, naze_files],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_latest_version(
        &self,
        package_id: i64,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE packages SET latest_version = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![version, package_id],
        )?;
        Ok(())
    }

    pub fn get_versions(
        &self,
        package_id: i64,
    ) -> Result<Vec<VersionRow>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, package_id, version, checksum, tarball_path, naze_files, created_at FROM versions WHERE package_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![package_id], |row| {
                Ok(VersionRow {
                    id: row.get(0)?,
                    package_id: row.get(1)?,
                    version: row.get(2)?,
                    checksum: row.get(3)?,
                    tarball_path: row.get(4)?,
                    naze_files: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_version(
        &self,
        package_id: i64,
        version: &str,
    ) -> Result<Option<VersionRow>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, package_id, version, checksum, tarball_path, naze_files, created_at FROM versions WHERE package_id = ?1 AND version = ?2",
        )?;
        let mut rows = stmt.query_map(params![package_id, version], |row| {
            Ok(VersionRow {
                id: row.get(0)?,
                package_id: row.get(1)?,
                version: row.get(2)?,
                checksum: row.get(3)?,
                tarball_path: row.get(4)?,
                naze_files: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<PackageRow>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{query}%");
        let mut stmt = conn.prepare(
            "SELECT id, name, description, latest_version, created_at, updated_at FROM packages WHERE name LIKE ?1 OR description LIKE ?1 ORDER BY updated_at DESC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![pattern, limit], |row| {
                Ok(PackageRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    latest_version: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Db {
        let db = Db::open_in_memory().unwrap();
        db.init_schema().unwrap();
        db
    }

    #[test]
    fn test_insert_and_get_package() {
        let db = setup();
        let id = db.insert_package("my-lib", "A test library").unwrap();
        let pkg = db.get_package("my-lib").unwrap().unwrap();
        assert_eq!(pkg.id, id);
        assert_eq!(pkg.name, "my-lib");
        assert_eq!(pkg.description, "A test library");
    }

    #[test]
    fn test_get_nonexistent_package() {
        let db = setup();
        assert!(db.get_package("nope").unwrap().is_none());
    }

    #[test]
    fn test_insert_and_get_versions() {
        let db = setup();
        let pkg_id = db.insert_package("versioned", "").unwrap();
        db.insert_version(pkg_id, "0.1.0", "abc123", "/path/0.1.0.tar.gz", 3)
            .unwrap();
        db.insert_version(pkg_id, "0.2.0", "def456", "/path/0.2.0.tar.gz", 5)
            .unwrap();
        let versions = db.get_versions(pkg_id).unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_get_specific_version() {
        let db = setup();
        let pkg_id = db.insert_package("specific", "").unwrap();
        db.insert_version(pkg_id, "1.0.0", "hash1", "/p/1.0.0.tar.gz", 2)
            .unwrap();
        let v = db.get_version(pkg_id, "1.0.0").unwrap().unwrap();
        assert_eq!(v.version, "1.0.0");
        assert_eq!(v.checksum, "hash1");
        assert!(db.get_version(pkg_id, "9.9.9").unwrap().is_none());
    }

    #[test]
    fn test_update_latest_version() {
        let db = setup();
        let pkg_id = db.insert_package("updating", "").unwrap();
        db.update_latest_version(pkg_id, "1.2.3").unwrap();
        let pkg = db.get_package("updating").unwrap().unwrap();
        assert_eq!(pkg.latest_version, "1.2.3");
    }

    #[test]
    fn test_search() {
        let db = setup();
        db.insert_package("naze-ui-kit", "UI components").unwrap();
        db.insert_package("naze-icons", "Icon pack").unwrap();
        db.insert_package("other-thing", "Unrelated").unwrap();

        let results = db.search("naze", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search("icon", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "naze-icons");
    }

    #[test]
    fn test_duplicate_package_errors() {
        let db = setup();
        db.insert_package("dup", "").unwrap();
        assert!(db.insert_package("dup", "").is_err());
    }

    #[test]
    fn test_duplicate_version_errors() {
        let db = setup();
        let pkg_id = db.insert_package("dupver", "").unwrap();
        db.insert_version(pkg_id, "1.0.0", "h1", "/p", 1).unwrap();
        assert!(db.insert_version(pkg_id, "1.0.0", "h2", "/p2", 1).is_err());
    }
}
