use std::path::Path;
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Result};
use crate::model::{Category, TimeBlock};

pub struct Database {
    conn: Connection,
    pub path_display: String,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        let path_display = path.to_string_lossy().to_string();

        let db = Self { conn, path_display };
        db.init_schema()?;
        db.seed_defaults_if_empty()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color_name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS blocks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                category_id TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                all_day INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );

            CREATE INDEX IF NOT EXISTS idx_blocks_time ON blocks(start_time, end_time);
            ",
        )?;
        Ok(())
    }

    fn seed_defaults_if_empty(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;

        if count == 0 {
            let defaults = [
                ("cat_deepwork", "Deep Work", "cyan"),
                ("cat_meeting", "Meeting", "yellow"),
                ("cat_personal", "Personal", "green"),
                ("cat_health", "Health", "magenta"),
                ("cat_task", "General Task", "white"),
            ];

            for (id, name, color) in defaults {
                self.conn.execute(
                    "INSERT INTO categories (id, name, color_name) VALUES (?1, ?2, ?3)",
                    params![id, name, color],
                )?;
            }
        }
        Ok(())
    }

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color_name FROM categories ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color_name: row.get(2)?,
            })
        })?;

        let mut list = Vec::new();
        for cat in rows {
            list.push(cat?);
        }
        Ok(list)
    }

    pub fn get_blocks_for_range(
        &self,
        range_start: DateTime<Local>,
        range_end: DateTime<Local>,
    ) -> Result<Vec<TimeBlock>> {
        let start_str = range_start.to_rfc3339();
        let end_str = range_end.to_rfc3339();

        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, category_id, start_time, end_time, all_day
             FROM blocks
             WHERE end_time >= ?1 AND start_time <= ?2
             ORDER BY start_time ASC",
        )?;

        let rows = stmt.query_map(params![start_str, end_str], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2)?;
            let category_id: String = row.get(3)?;
            let start_time_str: String = row.get(4)?;
            let end_time_str: String = row.get(5)?;
            let all_day_int: i32 = row.get(6)?;

            let start_time = DateTime::parse_from_rfc3339(&start_time_str)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());

            let end_time = DateTime::parse_from_rfc3339(&end_time_str)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());

            Ok(TimeBlock {
                id,
                title,
                description,
                category_id,
                start_time,
                end_time,
                all_day: all_day_int != 0,
            })
        })?;

        let mut blocks = Vec::new();
        for b in rows {
            blocks.push(b?);
        }
        Ok(blocks)
    }

    pub fn insert_block(&self, block: &TimeBlock) -> Result<()> {
        let now_str = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO blocks (id, title, description, category_id, start_time, end_time, all_day, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                block.id,
                block.title,
                block.description,
                block.category_id,
                block.start_time.to_rfc3339(),
                block.end_time.to_rfc3339(),
                if block.all_day { 1 } else { 0 },
                now_str,
                now_str,
            ],
        )?;
        Ok(())
    }

    pub fn update_block(&self, block: &TimeBlock) -> Result<()> {
        let now_str = Local::now().to_rfc3339();
        self.conn.execute(
            "UPDATE blocks SET title = ?1, description = ?2, category_id = ?3, start_time = ?4, end_time = ?5, all_day = ?6, updated_at = ?7
             WHERE id = ?8",
            params![
                block.title,
                block.description,
                block.category_id,
                block.start_time.to_rfc3339(),
                block.end_time.to_rfc3339(),
                if block.all_day { 1 } else { 0 },
                now_str,
                block.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_block(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM blocks WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};

    #[test]
    fn test_db_init_and_crud() {
        let temp_dir = std::env::temp_dir().join("dayplanner_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join(format!("test_{}.db", uuid::Uuid::new_v4()));

        let db = Database::new(&db_path).expect("Failed to init DB");

        // Verify categories seeded
        let categories = db.get_categories().expect("Failed to get categories");
        assert!(!categories.is_empty(), "Default categories should be seeded");

        // Test insert block
        let now = Local::now();
        let block = TimeBlock {
            id: "test-block-1".to_string(),
            title: "Test Standup".to_string(),
            description: "Daily sync meeting".to_string(),
            category_id: categories[0].id.clone(),
            start_time: now,
            end_time: now + Duration::hours(1),
            all_day: false,
        };

        db.insert_block(&block).expect("Insert failed");

        // Query blocks
        let blocks = db
            .get_blocks_for_range(now - Duration::hours(1), now + Duration::hours(2))
            .expect("Query failed");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].title, "Test Standup");

        // Update block
        let mut updated = block.clone();
        updated.title = "Updated Standup Title".to_string();
        db.update_block(&updated).expect("Update failed");

        let blocks2 = db
            .get_blocks_for_range(now - Duration::hours(1), now + Duration::hours(2))
            .expect("Query failed");
        assert_eq!(blocks2[0].title, "Updated Standup Title");

        // Delete block
        db.delete_block(&block.id).expect("Delete failed");
        let blocks3 = db
            .get_blocks_for_range(now - Duration::hours(1), now + Duration::hours(2))
            .expect("Query failed");
        assert_eq!(blocks3.len(), 0);

        let _ = std::fs::remove_file(db_path);
    }
}
