use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::kdl::Package;
use crate::store::simple_store_name;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS packages (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    name                  TEXT    NOT NULL,
    version               TEXT    NOT NULL,
    description           TEXT    NOT NULL,
    category              TEXT    NOT NULL,
    store_name            TEXT    NOT NULL,
    store_path            TEXT    NOT NULL UNIQUE,
    installed_at          INTEGER NOT NULL,
    icon                  TEXT,
    native_description    TEXT,
    run_dep_store_paths   TEXT    NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS generations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    number      INTEGER NOT NULL UNIQUE,
    created_at  INTEGER NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS generation_packages (
    generation_id INTEGER NOT NULL REFERENCES generations(id) ON DELETE CASCADE,
    package_id    INTEGER NOT NULL REFERENCES packages(id)    ON DELETE CASCADE,
    PRIMARY KEY (generation_id, package_id)
);

-- Single-row table: current active generation (0 = none).
CREATE TABLE IF NOT EXISTS current_generation (
    singleton         INTEGER PRIMARY KEY DEFAULT 1 CHECK (singleton = 1),
    generation_number INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO current_generation (singleton, generation_number) VALUES (1, 0);

CREATE TABLE IF NOT EXISTS repositories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    url        TEXT NOT NULL UNIQUE,
    added_at   INTEGER NOT NULL
);
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub store_name: String,
    pub store_path: String,
    pub installed_at: i64,
    pub icon: Option<String>,
    pub native_description: Option<String>,
    pub run_dep_store_paths: Vec<String>,
}

impl From<PackageRecord> for Package {
    fn from(rec: PackageRecord) -> Self {
        Self::from(&rec)
    }
}

impl From<&PackageRecord> for Package {
    fn from(rec: &PackageRecord) -> Self {
        Self {
            is_installed: Some(true),
            is_cached: None,
            is_disabled: None,
            package_hash: Some(rec.store_name.clone()),
            installed_at: Some(rec.installed_at.to_string()),
            name: rec.name.clone(),
            version: rec.version.clone(),
            description: rec.description.clone(),
            native_depiction: rec.native_description.clone(),
            license: None,
            homepage: None,
            category: Some(rec.category.clone()),
            icon: rec.icon.clone(),
            platforms: Vec::new(),
            dependencies: None,
            source: None,
            autobump: None,
            scripts: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRecord {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub id: i64,
    pub number: u32,
    pub created_at: i64,
    pub description: Option<String>,
    pub current: bool,
    pub packages: Vec<PackageRecord>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let root = Path::new(crate::store::STORE_ROOT);
        std::fs::create_dir_all(root).map_err(|e| Error::path(root, e.to_string()))?;
        let db_path = root.join("ventrica.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn insert_package(
        &self,
        package: &Package,
        store_path: &str,
        run_dep_store_paths: &[String],
    ) -> Result<()> {
        let name = package.name.as_str();
        let version = package.version.as_str();
        let description = package.description.as_str();
        let category = package.category.as_deref().unwrap_or_default();
        let store_name = simple_store_name(name, version);
        let icon = package.icon.as_deref();
        let native_description = package.native_depiction.as_deref();

        if self.find_package_by_store_path(store_path)?.is_some() {
            return Err(Error::AlreadyInstalled {
                name: name.into(),
                version: version.into(),
            });
        }

        let deps_json = serde_json::to_string(run_dep_store_paths).unwrap_or_else(|_| "[]".into());
        let now = unix_now();
        self.conn.execute(
            "INSERT INTO packages (name, version, description, category, store_name, store_path, installed_at, icon, native_description, run_dep_store_paths) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![name, version, description, category, &store_name, store_path, now, icon, native_description, deps_json],
        )?;
        Ok(())
    }

    pub fn remove_package(&self, name: &str) -> Result<()> {
        let n = self
            .conn
            .execute("DELETE FROM packages WHERE name = ?1", params![name])?;
        if n == 0 {
            return Err(Error::PackageNotFound { name: name.into() });
        }
        Ok(())
    }

    pub fn find_package(&self, name: &str) -> Result<Option<PackageRecord>> {
        let row = self.conn
            .query_row(
                "SELECT id, name, version, description, category, store_name, store_path, installed_at, icon, native_description, run_dep_store_paths \
                    FROM packages WHERE name = ?1 LIMIT 1",
                params![name],
                row_to_package,
            )
            .optional()?;
        Ok(row)
    }

    pub fn find_package_by_store_path(&self, store_path: &str) -> Result<Option<PackageRecord>> {
        self.conn
            .query_row(
                "SELECT id, name, version, description, category, store_name, store_path, installed_at, icon, native_description, run_dep_store_paths \
                 FROM packages WHERE store_path = ?1",
                params![store_path],
                row_to_package,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn find_package_manifest(&self, name: &str) -> Result<Option<Package>> {
        self.find_package(name).map(|pkg| pkg.map(Into::into))
    }

    pub fn list_packages(&self) -> Result<Vec<PackageRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, description, category, store_name, store_path, installed_at, icon, native_description, run_dep_store_paths \
             FROM packages ORDER BY name, version",
        )?;
        let rows = stmt
            .query_map([], row_to_package)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn list_packages_manifest(&self) -> Result<Vec<Package>> {
        self.list_packages()
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub fn create_generation(
        &self,
        package_ids: &[i64],
        description: Option<&str>,
    ) -> Result<GenerationRecord> {
        let number = self.next_generation_number()?;
        let now = unix_now();

        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO generations (number, created_at, description) VALUES (?1, ?2, ?3)",
            params![number, now, description],
        )?;
        let gen_id = tx.last_insert_rowid();

        for &pkg_id in package_ids {
            tx.execute(
                "INSERT INTO generation_packages (generation_id, package_id) VALUES (?1, ?2)",
                params![gen_id, pkg_id],
            )?;
        }

        tx.execute(
            "UPDATE current_generation SET generation_number = ?1 WHERE singleton = 1",
            params![number],
        )?;

        tx.commit()?;

        Ok(GenerationRecord {
            id: gen_id,
            number,
            created_at: now,
            description: description.map(ToOwned::to_owned),
            current: false,
            packages: vec![],
        })
    }

    pub fn get_generation(&self, number: u32) -> Result<GenerationRecord> {
        let rec = self
            .conn
            .query_row(
                "SELECT id, number, created_at, description FROM generations WHERE number = ?1",
                params![number],
                row_to_generation,
            )
            .optional()?
            .ok_or(Error::GenerationNotFound(number))?;
        Ok(rec)
    }

    pub fn list_generations(&self) -> Result<Vec<GenerationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, number, created_at, description FROM generations ORDER BY number",
        )?;
        let rows = stmt
            .query_map([], row_to_generation)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn packages_in_generation(&self, generation_number: u32) -> Result<Vec<PackageRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.version, p.description, p.category, p.store_name, p.store_path, p.installed_at, p.icon, p.native_description, p.run_dep_store_paths \
             FROM packages p \
             JOIN generation_packages gp ON gp.package_id = p.id \
             JOIN generations g ON g.id = gp.generation_id \
             WHERE g.number = ?1 \
             ORDER BY p.name",
        )?;
        let rows = stmt
            .query_map(params![generation_number], row_to_package)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn packages_in_generation_manifest(&self, generation_number: u32) -> Result<Vec<Package>> {
        self.packages_in_generation(generation_number)
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub fn current_generation_number(&self) -> Result<u32> {
        let n: u32 = self.conn.query_row(
            "SELECT generation_number FROM current_generation WHERE singleton = 1",
            [],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    pub fn set_current_generation(&self, number: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE current_generation SET generation_number = ?1 WHERE singleton = 1",
            params![number],
        )?;
        Ok(())
    }

    pub fn delete_generation(&self, number: u32) -> Result<()> {
        self.conn
            .execute("DELETE FROM generations WHERE number = ?1", params![number])?;
        Ok(())
    }

    fn next_generation_number(&self) -> Result<u32> {
        let max: Option<u32> =
            self.conn
                .query_row("SELECT MAX(number) FROM generations", [], |r| r.get(0))?;
        Ok(max.unwrap_or(0) + 1)
    }

    pub fn add_repo(&self, name: &str, url: &str) -> Result<()> {
        let now = unix_now();
        self.conn.execute(
            "INSERT OR IGNORE INTO repositories (name, url, added_at) VALUES (?1, ?2, ?3)",
            params![name, url, now],
        )?;
        Ok(())
    }

    pub fn remove_repo(&self, url: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM repositories WHERE url = ?1", params![url])?;
        Ok(())
    }

    pub fn list_repos(&self) -> Result<Vec<RepoRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, url, added_at FROM repositories ORDER BY added_at")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RepoRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    added_at: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
}

fn row_to_package(row: &rusqlite::Row<'_>) -> rusqlite::Result<PackageRecord> {
    let deps_json: String = row.get(10).unwrap_or_else(|_| "[]".to_owned());
    let run_dep_store_paths: Vec<String> = serde_json::from_str(&deps_json).unwrap_or_default();
    Ok(PackageRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        description: row.get(3)?,
        category: row.get(4)?,
        store_name: row.get(5)?,
        store_path: row.get(6)?,
        installed_at: row.get(7)?,
        icon: row.get(8)?,
        native_description: row.get(9)?,
        run_dep_store_paths,
    })
}

fn row_to_generation(row: &rusqlite::Row<'_>) -> rusqlite::Result<GenerationRecord> {
    Ok(GenerationRecord {
        id: row.get(0)?,
        number: row.get(1)?,
        created_at: row.get(2)?,
        description: row.get(3)?,
        current: false,
        packages: vec![],
    })
}

fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
