use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::Result;
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct AuthToken {
    pub id: String,
    pub account_id: String,
    pub device_id: Option<String>,
    pub token_hash: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}

impl Database {
    pub async fn create_auth_token(
        &self,
        id: &str,
        account_id: &str,
        device_id: Option<&str>,
        token_hash: &str,
        token_type: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO auth_tokens (id, account_id, device_id, token_hash, token_type, expires_at, created_at, revoked)
            VALUES (?, ?, ?, ?, ?, ?, NOW(), FALSE)
            "#
        )
        .bind(id)
        .bind(account_id)
        .bind(device_id)
        .bind(token_hash)
        .bind(token_type)
        .bind(expires_at)
        .execute(self.pool())
        .await?;
        
        Ok(())
    }
    
    pub async fn get_auth_token(&self, token_hash: &str) -> Result<Option<AuthToken>> {
        let token = sqlx::query_as::<_, AuthToken>(
            "SELECT id, account_id, device_id, token_hash, token_type, expires_at, created_at, revoked FROM auth_tokens WHERE token_hash = ? AND revoked = FALSE"
        )
        .bind(token_hash)
        .fetch_optional(self.pool())
        .await?;
        
        Ok(token)
    }
    
    pub async fn revoke_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("UPDATE auth_tokens SET revoked = TRUE WHERE token_hash = ?")
            .bind(token_hash)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    pub async fn revoke_all_tokens_for_account(&self, account_id: &str) -> Result<()> {
        sqlx::query("UPDATE auth_tokens SET revoked = TRUE WHERE account_id = ?")
            .bind(account_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    pub async fn cleanup_expired_tokens(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM auth_tokens WHERE expires_at < NOW() OR revoked = TRUE")
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }
}
