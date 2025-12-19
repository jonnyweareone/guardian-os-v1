use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::{AppError, Result};
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct FileRecord {
    pub id: String,
    pub account_id: String,
    pub filename: String,
    pub file_type: String,
    pub content_type: String,
    pub size: i64,
    pub checksum: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Database {
    pub async fn create_file(
        &self,
        id: &str,
        account_id: &str,
        filename: &str,
        file_type: &str,
        content_type: &str,
        size: i64,
        checksum: &str,
        storage_path: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO files (id, account_id, filename, file_type, content_type, size, checksum, storage_path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
            "#
        )
        .bind(id)
        .bind(account_id)
        .bind(filename)
        .bind(file_type)
        .bind(content_type)
        .bind(size)
        .bind(checksum)
        .bind(storage_path)
        .execute(self.pool())
        .await?;
        
        Ok(())
    }
    
    pub async fn get_file(&self, id: &str) -> Result<FileRecord> {
        sqlx::query_as::<_, FileRecord>(
            "SELECT id, account_id, filename, file_type, content_type, size, checksum, storage_path, created_at, updated_at FROM files WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::FileNotFound)
    }
    
    pub async fn list_files(
        &self,
        account_id: &str,
        file_type: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<FileRecord>> {
        let files = if let Some(ft) = file_type {
            sqlx::query_as::<_, FileRecord>(
                "SELECT id, account_id, filename, file_type, content_type, size, checksum, storage_path, created_at, updated_at FROM files WHERE account_id = ? AND file_type = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(account_id)
            .bind(ft)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, FileRecord>(
                "SELECT id, account_id, filename, file_type, content_type, size, checksum, storage_path, created_at, updated_at FROM files WHERE account_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(account_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?
        };
        
        Ok(files)
    }
    
    pub async fn delete_file(&self, id: &str) -> Result<String> {
        let file = self.get_file(id).await?;
        
        sqlx::query("DELETE FROM files WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
            
        Ok(file.storage_path)
    }
    
    pub async fn get_total_storage_used(&self, account_id: &str) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(size), 0) FROM files WHERE account_id = ?"
        )
        .bind(account_id)
        .fetch_one(self.pool())
        .await?;
        
        Ok(result.0)
    }
}
