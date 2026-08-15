use crate::db::Model;

#[derive(Default)]
pub struct ValidEntry {
    pub id: i64,
    pub r#type: String,
    pub path: String,
    pub hash: String,
    pub name: String,
    pub version: Option<String>,
}

impl Model for ValidEntry {
    fn schema() -> &'static str {
        "CREATE TABLE IF NOT EXISTS ValidPaths (
            id      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            type    TEXT NOT NULL,
            path    TEXT UNIQUE NOT NULL,
            hash    TEXT NOT NULL,
            name    TEXT NOT NULL DEFAULT '',
            version TEXT
        );"
    }

    fn row_to_self(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            r#type: row.get("type")?,
            path: row.get("path")?,
            hash: row.get("hash")?,
            name: row.get("name")?,
            version: row.get("version")?,
        })
    }
}
