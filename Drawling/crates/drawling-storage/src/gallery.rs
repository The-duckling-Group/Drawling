use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub title: String,
    pub file_path: String,
    pub thumbnail_png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edit_duration_secs: u64,
    pub tags: Vec<String>,
}

pub struct GalleryDatabase {
    conn: Connection,
}

impl GalleryDatabase {
    pub fn open(db_path: impl AsRef<Path>) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                thumbnail_png BLOB,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                edit_duration_secs INTEGER NOT NULL,
                tags TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_updated_at ON projects(updated_at DESC);
            CREATE TABLE IF NOT EXISTS autosave_journal (
                project_id TEXT PRIMARY KEY,
                snapshot_path TEXT NOT NULL,
                saved_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                thumbnail_png BLOB,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                edit_duration_secs INTEGER NOT NULL,
                tags TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_updated_at ON projects(updated_at DESC);
            CREATE TABLE IF NOT EXISTS autosave_journal (
                project_id TEXT PRIMARY KEY,
                snapshot_path TEXT NOT NULL,
                saved_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn upsert_project(&mut self, record: &ProjectRecord) -> SqlResult<()> {
        let tags_json = serde_json::to_string(&record.tags).unwrap_or_else(|_| "[]".into());
        self.conn.execute(
            "
            INSERT INTO projects (
                id, title, file_path, thumbnail_png, width, height,
                created_at, updated_at, edit_duration_secs, tags
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                file_path = excluded.file_path,
                thumbnail_png = excluded.thumbnail_png,
                width = excluded.width,
                height = excluded.height,
                updated_at = excluded.updated_at,
                edit_duration_secs = excluded.edit_duration_secs,
                tags = excluded.tags;
            ",
            params![
                record.id.to_string(),
                record.title,
                record.file_path,
                record.thumbnail_png,
                record.width,
                record.height,
                record.created_at.to_rfc3339(),
                record.updated_at.to_rfc3339(),
                record.edit_duration_secs,
                tags_json,
            ],
        )?;
        Ok(())
    }

    pub fn list_projects(&self) -> SqlResult<Vec<ProjectRecord>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, title, file_path, thumbnail_png, width, height,
                   created_at, updated_at, edit_duration_secs, tags
            FROM projects
            ORDER BY updated_at DESC;
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let file_path: String = row.get(2)?;
            let thumbnail_png: Vec<u8> = row.get(3).unwrap_or_default();
            let width: u32 = row.get(4)?;
            let height: u32 = row.get(5)?;
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;
            let edit_duration_secs: u64 = row.get(8)?;
            let tags_str: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

            Ok(ProjectRecord {
                id,
                title,
                file_path,
                thumbnail_png,
                width,
                height,
                created_at,
                updated_at,
                edit_duration_secs,
                tags,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn delete_project(&mut self, id: &Uuid) -> SqlResult<bool> {
        let count = self.conn.execute("DELETE FROM projects WHERE id = ?1", params![id.to_string()])?;
        Ok(count > 0)
    }

    pub fn search_by_tag(&self, tag: &str) -> SqlResult<Vec<ProjectRecord>> {
        let all = self.list_projects()?;
        let tag_lower = tag.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|p| p.tags.iter().any(|t| t.to_lowercase().contains(&tag_lower)))
            .collect())
    }

    pub fn record_autosave(&mut self, project_id: &Uuid, snapshot_path: &Path) -> SqlResult<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "
            INSERT INTO autosave_journal (project_id, snapshot_path, saved_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(project_id) DO UPDATE SET
                snapshot_path = excluded.snapshot_path,
                saved_at = excluded.saved_at;
            ",
            params![project_id.to_string(), snapshot_path.to_string_lossy(), now],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: &Uuid) -> SqlResult<Option<ProjectRecord>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, title, file_path, thumbnail_png, width, height,
                   created_at, updated_at, edit_duration_secs, tags
            FROM projects
            WHERE id = ?1;
            ",
        )?;

        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let title: String = row.get(1)?;
            let file_path: String = row.get(2)?;
            let thumbnail_png: Vec<u8> = row.get(3).unwrap_or_default();
            let width: u32 = row.get(4)?;
            let height: u32 = row.get(5)?;
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;
            let edit_duration_secs: u64 = row.get(8)?;
            let tags_str: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

            Ok(ProjectRecord {
                id,
                title,
                file_path,
                thumbnail_png,
                width,
                height,
                created_at,
                updated_at,
                edit_duration_secs,
                tags,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn duplicate_project(
        &mut self,
        source_id: &Uuid,
        new_title: &str,
        new_file_path: &str,
    ) -> SqlResult<Option<ProjectRecord>> {
        let orig = match self.get_project(source_id)? {
            Some(p) => p,
            None => return Ok(None),
        };

        let new_record = ProjectRecord {
            id: Uuid::new_v4(),
            title: new_title.to_string(),
            file_path: new_file_path.to_string(),
            thumbnail_png: orig.thumbnail_png,
            width: orig.width,
            height: orig.height,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            edit_duration_secs: 0,
            tags: orig.tags,
        };

        self.upsert_project(&new_record)?;
        Ok(Some(new_record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_db_crud_and_duplicate() {
        let mut db = GalleryDatabase::open_in_memory().unwrap();
        let proj_id = Uuid::new_v4();

        let record = ProjectRecord {
            id: proj_id,
            title: "Anime Sketch".into(),
            file_path: "/tmp/sketch.drawling".into(),
            thumbnail_png: vec![1, 2, 3, 4],
            width: 1920,
            height: 1080,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            edit_duration_secs: 300,
            tags: vec!["Anime".into(), "Character".into()],
        };

        db.upsert_project(&record).unwrap();

        let loaded = db.get_project(&proj_id).unwrap().unwrap();
        assert_eq!(loaded.title, "Anime Sketch");
        assert_eq!(loaded.width, 1920);

        // Search by tag
        let anime_results = db.search_by_tag("anime").unwrap();
        assert_eq!(anime_results.len(), 1);
        let landscape_results = db.search_by_tag("landscape").unwrap();
        assert_eq!(landscape_results.len(), 0);

        // Duplicate
        let dup = db
            .duplicate_project(&proj_id, "Anime Sketch Copy", "/tmp/sketch_copy.drawling")
            .unwrap()
            .unwrap();
        assert_eq!(dup.title, "Anime Sketch Copy");
        assert_ne!(dup.id, proj_id);

        let list = db.list_projects().unwrap();
        assert_eq!(list.len(), 2);

        // Delete
        assert!(db.delete_project(&proj_id).unwrap());
        assert_eq!(db.list_projects().unwrap().len(), 1);
    }
}
