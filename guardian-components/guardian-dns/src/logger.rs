//! Query logging for Guardian DNS

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::Path;

/// Query action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryAction {
    Allowed,
    Blocked,
    SafeSearch,
    VpnBlocked,
}

impl QueryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueryAction::Allowed => "allowed",
            QueryAction::Blocked => "blocked",
            QueryAction::SafeSearch => "safesearch",
            QueryAction::VpnBlocked => "vpn_blocked",
        }
    }
}

/// DNS query log entry
#[derive(Debug, Clone)]
pub struct QueryLogEntry {
    pub timestamp: DateTime<Utc>,
    pub child_id: String,
    pub domain: String,
    pub category: Option<String>,
    pub action: QueryAction,
}

/// Query logger - stores DNS query logs in SQLite
pub struct QueryLogger {
    conn: Connection,
    enabled: bool,
    retention_days: u32,
}

impl QueryLogger {
    /// Create a new query logger
    pub fn new(db_path: &Path, enabled: bool, retention_days: u32) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dns_queries (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                child_id TEXT NOT NULL,
                domain TEXT NOT NULL,
                category TEXT,
                action TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON dns_queries (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_child ON dns_queries (child_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_domain ON dns_queries (domain)",
            [],
        )?;
        
        let logger = Self {
            conn,
            enabled,
            retention_days,
        };
        
        // Run initial cleanup
        logger.cleanup_old_entries()?;
        
        Ok(logger)
    }
    
    /// Log a DNS query
    pub fn log(&self, entry: &QueryLogEntry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        self.conn.execute(
            "INSERT INTO dns_queries (timestamp, child_id, domain, category, action)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.child_id,
                entry.domain,
                entry.category,
                entry.action.as_str(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Log an allowed query
    pub fn log_allowed(&self, child_id: &str, domain: &str, category: Option<&str>) {
        let entry = QueryLogEntry {
            timestamp: Utc::now(),
            child_id: child_id.to_string(),
            domain: domain.to_string(),
            category: category.map(|s| s.to_string()),
            action: QueryAction::Allowed,
        };
        
        if let Err(e) = self.log(&entry) {
            tracing::warn!("Failed to log allowed query: {}", e);
        }
    }
    
    /// Log a blocked query
    pub fn log_blocked(&self, child_id: &str, domain: &str, category: &str) {
        let entry = QueryLogEntry {
            timestamp: Utc::now(),
            child_id: child_id.to_string(),
            domain: domain.to_string(),
            category: Some(category.to_string()),
            action: QueryAction::Blocked,
        };
        
        if let Err(e) = self.log(&entry) {
            tracing::warn!("Failed to log blocked query: {}", e);
        }
    }
    
    /// Log a safe search rewrite
    pub fn log_safesearch(&self, child_id: &str, original_domain: &str, rewrite_target: &str) {
        let entry = QueryLogEntry {
            timestamp: Utc::now(),
            child_id: child_id.to_string(),
            domain: original_domain.to_string(),
            category: Some(format!("rewrite:{}", rewrite_target)),
            action: QueryAction::SafeSearch,
        };
        
        if let Err(e) = self.log(&entry) {
            tracing::warn!("Failed to log safesearch: {}", e);
        }
    }
    
    /// Log a VPN/proxy block
    pub fn log_vpn_blocked(&self, child_id: &str, domain: &str, reason: &str) {
        let entry = QueryLogEntry {
            timestamp: Utc::now(),
            child_id: child_id.to_string(),
            domain: domain.to_string(),
            category: Some(reason.to_string()),
            action: QueryAction::VpnBlocked,
        };
        
        if let Err(e) = self.log(&entry) {
            tracing::warn!("Failed to log VPN block: {}", e);
        }
    }
    
    /// Clean up old log entries
    pub fn cleanup_old_entries(&self) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        
        let deleted = self.conn.execute(
            "DELETE FROM dns_queries WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        
        // Vacuum to reclaim space
        if deleted > 0 {
            self.conn.execute("VACUUM", [])?;
        }
        
        Ok(deleted)
    }
    
    /// Get query count for a child
    pub fn get_query_count(&self, child_id: &str, since: DateTime<Utc>) -> Result<u64> {
        let count: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM dns_queries 
             WHERE child_id = ?1 AND timestamp >= ?2",
            params![child_id, since.to_rfc3339()],
            |row| row.get(0),
        )?;
        
        Ok(count)
    }
    
    /// Get top domains for a child
    pub fn get_top_domains(
        &self,
        child_id: &str,
        since: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<(String, u64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT domain, COUNT(*) as count FROM dns_queries 
             WHERE child_id = ?1 AND timestamp >= ?2
             GROUP BY domain
             ORDER BY count DESC
             LIMIT ?3"
        )?;
        
        let rows = stmt.query_map(
            params![child_id, since.to_rfc3339(), limit],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?)),
        )?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
    
    /// Get blocked attempts for a child
    pub fn get_blocked_attempts(
        &self,
        child_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<QueryLogEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, child_id, domain, category, action FROM dns_queries 
             WHERE child_id = ?1 AND timestamp >= ?2 AND action IN ('blocked', 'vpn_blocked')
             ORDER BY timestamp DESC"
        )?;
        
        let rows = stmt.query_map(
            params![child_id, since.to_rfc3339()],
            |row| {
                let timestamp_str: String = row.get(0)?;
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                
                let action_str: String = row.get(4)?;
                let action = match action_str.as_str() {
                    "blocked" => QueryAction::Blocked,
                    "vpn_blocked" => QueryAction::VpnBlocked,
                    "safesearch" => QueryAction::SafeSearch,
                    _ => QueryAction::Allowed,
                };
                
                Ok(QueryLogEntry {
                    timestamp,
                    child_id: row.get(1)?,
                    domain: row.get(2)?,
                    category: row.get(3)?,
                    action,
                })
            },
        )?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
}
