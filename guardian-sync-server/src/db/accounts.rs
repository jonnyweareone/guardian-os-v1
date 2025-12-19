use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::{AppError, Result};
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct Account {
    pub id: u64,  // BIGINT UNSIGNED auto_increment
    pub account_hash: String,  // UUID for external reference
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email_verified: bool,
    pub status: String,
}

impl Database {
    pub async fn create_account(
        &self,
        account_hash: &str,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            INSERT INTO accounts (account_hash, email, password_hash, display_name, created_at, updated_at, email_verified, status)
            VALUES (?, ?, ?, ?, NOW(), NOW(), FALSE, 'active')
            "#
        )
        .bind(account_hash)
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .execute(self.pool())
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate entry") {
                AppError::UserAlreadyExists
            } else {
                AppError::Database(e)
            }
        })?;
        
        Ok(result.last_insert_id())
    }
    
    pub async fn get_account_by_hash(&self, account_hash: &str) -> Result<Account> {
        sqlx::query_as::<_, Account>(
            "SELECT id, account_hash, email, password_hash, display_name, created_at, updated_at, email_verified, status FROM accounts WHERE account_hash = ?"
        )
        .bind(account_hash)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::UserNotFound)
    }
    
    pub async fn get_account_by_email(&self, email: &str) -> Result<Account> {
        sqlx::query_as::<_, Account>(
            "SELECT id, account_hash, email, password_hash, display_name, created_at, updated_at, email_verified, status FROM accounts WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::UserNotFound)
    }
    
    pub async fn update_password(&self, account_hash: &str, new_password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE accounts SET password_hash = ?, updated_at = NOW() WHERE account_hash = ?")
            .bind(new_password_hash)
            .bind(account_hash)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    pub async fn verify_email(&self, account_hash: &str) -> Result<()> {
        sqlx::query("UPDATE accounts SET email_verified = TRUE, updated_at = NOW() WHERE account_hash = ?")
            .bind(account_hash)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
