use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::{AppError, Result};
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct Device {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub device_type: String,
    pub os_version: Option<String>,
    pub hardware_id: Option<String>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

impl Database {
    pub async fn create_device(
        &self,
        id: &str,
        account_id: &str,
        name: &str,
        device_type: &str,
        os_version: Option<&str>,
        hardware_id: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO devices (id, account_id, name, device_type, os_version, hardware_id, last_seen_at, created_at, status)
            VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW(), 'active')
            "#
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .bind(device_type)
        .bind(os_version)
        .bind(hardware_id)
        .execute(self.pool())
        .await?;
        
        Ok(())
    }
    
    pub async fn get_device(&self, id: &str) -> Result<Device> {
        sqlx::query_as::<_, Device>(
            "SELECT id, account_id, name, device_type, os_version, hardware_id, last_seen_at, created_at, status FROM devices WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::DeviceNotFound)
    }
    
    pub async fn get_device_by_hardware_id(&self, hardware_id: &str) -> Result<Option<Device>> {
        let device = sqlx::query_as::<_, Device>(
            "SELECT id, account_id, name, device_type, os_version, hardware_id, last_seen_at, created_at, status FROM devices WHERE hardware_id = ?"
        )
        .bind(hardware_id)
        .fetch_optional(self.pool())
        .await?;
        
        Ok(device)
    }
    
    pub async fn list_devices_for_account(&self, account_id: &str) -> Result<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>(
            "SELECT id, account_id, name, device_type, os_version, hardware_id, last_seen_at, created_at, status FROM devices WHERE account_id = ? AND status = 'active' ORDER BY last_seen_at DESC"
        )
        .bind(account_id)
        .fetch_all(self.pool())
        .await?;
        
        Ok(devices)
    }
    
    pub async fn update_device_last_seen(&self, device_id: &str) -> Result<()> {
        sqlx::query("UPDATE devices SET last_seen_at = NOW() WHERE id = ?")
            .bind(device_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    pub async fn revoke_device(&self, device_id: &str) -> Result<()> {
        sqlx::query("UPDATE devices SET status = 'revoked' WHERE id = ?")
            .bind(device_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
