use std::sync::Arc;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

use crate::db::Database;
use crate::error::Result;

pub struct SyncService {
    db: Arc<Database>,
}

impl SyncService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    // Calculate checksum for value
    pub fn calculate_checksum(value: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value);
        hex::encode(hasher.finalize())
    }
    
    // Push settings from device
    pub async fn push_settings(
        &self,
        account_id: &str,
        device_id: &str,
        entries: Vec<(String, Vec<u8>, String, i64)>, // (key, value, category, modified_at)
    ) -> Result<(i64, Vec<(String, Vec<u8>, Vec<u8>, i64, i64)>)> {
        let mut conflicts = Vec::new();
        let server_timestamp = Utc::now().timestamp();
        
        for (key, value, category, client_modified) in entries {
            // Check for existing setting
            if let Some(existing) = self.db.get_setting(account_id, &key).await? {
                let existing_modified = existing.modified_at.timestamp();
                
                // Conflict detection: server has newer change
                if existing_modified > client_modified {
                    conflicts.push((
                        key.clone(),
                        existing.value.clone(),
                        value.clone(),
                        existing_modified,
                        client_modified,
                    ));
                    continue;
                }
            }
            
            // Upsert the setting
            let id = uuid::Uuid::new_v4().to_string();
            let checksum = Self::calculate_checksum(&value);
            self.db.upsert_setting(
                &id,
                account_id,
                Some(device_id),
                &key,
                &value,
                &category,
                &checksum,
            ).await?;
        }
        
        Ok((server_timestamp, conflicts))
    }
    
    // Pull settings from server
    pub async fn pull_settings(
        &self,
        account_id: &str,
        since_timestamp: i64,
        categories: Option<Vec<String>>,
    ) -> Result<(Vec<crate::db::SettingsEntry>, i64)> {
        let since = if since_timestamp == 0 {
            DateTime::from_timestamp(0, 0).unwrap()
        } else {
            DateTime::from_timestamp(since_timestamp, 0).unwrap()
        };
        
        let entries = self.db.get_settings_since(
            account_id,
            since,
            categories.as_ref().map(|c| c.as_slice()),
        ).await?;
        
        let server_timestamp = Utc::now().timestamp();
        
        Ok((entries, server_timestamp))
    }
    
    // Get all settings for full sync
    pub async fn get_all_settings(&self, account_id: &str) -> Result<Vec<crate::db::SettingsEntry>> {
        self.db.get_all_settings(account_id).await
    }
    
    // Delete a setting
    pub async fn delete_setting(&self, account_id: &str, key: &str) -> Result<()> {
        self.db.delete_setting(account_id, key).await
    }
}
