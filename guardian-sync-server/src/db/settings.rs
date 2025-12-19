use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::Result;
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct SettingsEntry {
    pub id: String,
    pub account_id: String,
    pub device_id: Option<String>,
    pub key: String,
    pub value: Vec<u8>,
    pub category: String,
    pub checksum: String,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Database {
    pub async fn upsert_setting(
        &self,
        id: &str,
        account_id: &str,
        device_id: Option<&str>,
        key: &str,
        value: &[u8],
        category: &str,
        checksum: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
            ON DUPLICATE KEY UPDATE
                value = VALUES(value),
                checksum = VALUES(checksum),
                modified_at = NOW()
            "#
        )
        .bind(id)
        .bind(account_id)
        .bind(device_id)
        .bind(key)
        .bind(value)
        .bind(category)
        .bind(checksum)
        .execute(self.pool())
        .await?;
        
        Ok(())
    }
    
    pub async fn get_settings_since(
        &self,
        account_id: &str,
        since: DateTime<Utc>,
        categories: Option<&[String]>,
    ) -> Result<Vec<SettingsEntry>> {
        let settings = if let Some(cats) = categories {
            if cats.is_empty() {
                sqlx::query_as::<_, SettingsEntry>(
                    "SELECT id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at FROM settings WHERE account_id = ? AND modified_at > ? ORDER BY modified_at ASC"
                )
                .bind(account_id)
                .bind(since)
                .fetch_all(self.pool())
                .await?
            } else {
                // Build IN clause dynamically
                let placeholders: Vec<&str> = cats.iter().map(|_| "?").collect();
                let query = format!(
                    "SELECT id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at FROM settings WHERE account_id = ? AND modified_at > ? AND category IN ({}) ORDER BY modified_at ASC",
                    placeholders.join(", ")
                );
                
                let mut q = sqlx::query_as::<_, SettingsEntry>(&query)
                    .bind(account_id)
                    .bind(since);
                
                for cat in cats {
                    q = q.bind(cat);
                }
                
                q.fetch_all(self.pool()).await?
            }
        } else {
            sqlx::query_as::<_, SettingsEntry>(
                "SELECT id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at FROM settings WHERE account_id = ? AND modified_at > ? ORDER BY modified_at ASC"
            )
            .bind(account_id)
            .bind(since)
            .fetch_all(self.pool())
            .await?
        };
        
        Ok(settings)
    }
    
    pub async fn get_all_settings(&self, account_id: &str) -> Result<Vec<SettingsEntry>> {
        let settings = sqlx::query_as::<_, SettingsEntry>(
            "SELECT id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at FROM settings WHERE account_id = ? ORDER BY `key` ASC"
        )
        .bind(account_id)
        .fetch_all(self.pool())
        .await?;
        
        Ok(settings)
    }
    
    pub async fn get_setting(&self, account_id: &str, key: &str) -> Result<Option<SettingsEntry>> {
        let setting = sqlx::query_as::<_, SettingsEntry>(
            "SELECT id, account_id, device_id, `key`, value, category, checksum, modified_at, created_at FROM settings WHERE account_id = ? AND `key` = ?"
        )
        .bind(account_id)
        .bind(key)
        .fetch_optional(self.pool())
        .await?;
        
        Ok(setting)
    }
    
    pub async fn delete_setting(&self, account_id: &str, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM settings WHERE account_id = ? AND `key` = ?")
            .bind(account_id)
            .bind(key)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
