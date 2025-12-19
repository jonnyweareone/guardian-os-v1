//! Local SQLite storage for offline operation

use std::path::{Path, PathBuf};
use std::time::Duration;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;
use anyhow::Result;
use tracing::info;

use crate::activity::ActivityEvent;
use crate::rules::SafetyRules;

/// Local SQLite storage for activity logs and cached rules
pub struct LocalStorage {
    db_path: PathBuf,
    conn: Mutex<Connection>,
}

impl LocalStorage {
    /// Create new storage instance
    pub fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("guardian.db");
        
        let conn = Connection::open(&db_path)?;
        
        let storage = Self {
            db_path,
            conn: Mutex::new(conn),
        };
        
        // Initialize schema
        tokio::runtime::Handle::current().block_on(storage.init_schema())?;
        
        Ok(storage)
    }
    
    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute_batch(r#"
            -- Activity events table
            CREATE TABLE IF NOT EXISTS activity_events (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                child_id TEXT,
                activity_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                data TEXT NOT NULL,
                synced INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Index for efficient queries
            CREATE INDEX IF NOT EXISTS idx_activity_timestamp 
                ON activity_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activity_child 
                ON activity_events(child_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_activity_synced 
                ON activity_events(synced);
            
            -- Cached safety rules
            CREATE TABLE IF NOT EXISTS safety_rules (
                child_id TEXT PRIMARY KEY,
                rules_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Screen time tracking (aggregated daily)
            CREATE TABLE IF NOT EXISTS daily_screen_time (
                date TEXT NOT NULL,
                child_id TEXT NOT NULL,
                total_seconds INTEGER DEFAULT 0,
                last_updated TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (date, child_id)
            );
            
            -- URL classification cache
            CREATE TABLE IF NOT EXISTS url_cache (
                url_hash TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                classification TEXT NOT NULL,
                cached_at TEXT DEFAULT CURRENT_TIMESTAMP,
                expires_at TEXT
            );
        "#)?;
        
        info!("Database schema initialized");
        Ok(())
    }
    
    /// Record an activity event
    pub async fn record_activity(&self, event: &ActivityEvent) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            r#"INSERT INTO activity_events 
               (id, device_id, child_id, activity_type, timestamp, data, synced)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                event.id,
                event.device_id,
                event.child_id,
                serde_json::to_string(&event.activity_type)?,
                event.timestamp.to_rfc3339(),
                event.data.to_string(),
                event.synced as i32,
            ],
        )?;
        
        Ok(())
    }
    
    /// Get unsynced activity events
    pub async fn get_unsynced_events(&self, limit: usize) -> Result<Vec<ActivityEvent>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            r#"SELECT id, device_id, child_id, activity_type, timestamp, data, synced
               FROM activity_events 
               WHERE synced = 0 
               ORDER BY timestamp ASC 
               LIMIT ?1"#
        )?;
        
        let events = stmt.query_map([limit], |row| {
            Ok(ActivityEvent {
                id: row.get(0)?,
                device_id: row.get(1)?,
                child_id: row.get(2)?,
                activity_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                data: serde_json::from_str(&row.get::<_, String>(5)?).unwrap(),
                synced: row.get::<_, i32>(6)? != 0,
            })
        })?;
        
        Ok(events.filter_map(|e| e.ok()).collect())
    }
    
    /// Mark events as synced
    pub async fn mark_synced(&self, event_ids: &[String]) -> Result<()> {
        let conn = self.conn.lock().await;
        
        for id in event_ids {
            conn.execute(
                "UPDATE activity_events SET synced = 1 WHERE id = ?1",
                [id],
            )?;
        }
        
        Ok(())
    }
    
    /// Get daily screen time for a child
    pub async fn get_daily_screen_time(&self, child_id: &str) -> Result<Duration> {
        let conn = self.conn.lock().await;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        let seconds: i64 = conn.query_row(
            "SELECT total_seconds FROM daily_screen_time WHERE date = ?1 AND child_id = ?2",
            [&today, child_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        Ok(Duration::from_secs(seconds as u64))
    }
    
    /// Update daily screen time
    pub async fn add_screen_time(&self, child_id: &str, seconds: u64) -> Result<()> {
        let conn = self.conn.lock().await;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        conn.execute(
            r#"INSERT INTO daily_screen_time (date, child_id, total_seconds)
               VALUES (?1, ?2, ?3)
               ON CONFLICT(date, child_id) DO UPDATE SET
                   total_seconds = total_seconds + ?3,
                   last_updated = CURRENT_TIMESTAMP"#,
            params![today, child_id, seconds as i64],
        )?;
        
        Ok(())
    }
    
    /// Load cached safety rules
    pub async fn load_rules(&self) -> Result<SafetyRules> {
        let conn = self.conn.lock().await;
        
        let rules_json: String = conn.query_row(
            "SELECT rules_json FROM safety_rules LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        
        Ok(serde_json::from_str(&rules_json)?)
    }
    
    /// Save safety rules to cache
    pub async fn save_rules(&self, rules: &SafetyRules) -> Result<()> {
        let conn = self.conn.lock().await;
        let child_id = rules.child_id.as_deref().unwrap_or("default");
        
        conn.execute(
            r#"INSERT INTO safety_rules (child_id, rules_json, version)
               VALUES (?1, ?2, ?3)
               ON CONFLICT(child_id) DO UPDATE SET
                   rules_json = ?2,
                   version = ?3,
                   updated_at = CURRENT_TIMESTAMP"#,
            params![
                child_id,
                serde_json::to_string(rules)?,
                rules.version,
            ],
        )?;
        
        Ok(())
    }
}
