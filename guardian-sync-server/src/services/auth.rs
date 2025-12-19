use std::sync::Arc;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use crate::config::Config;
use crate::db::Database;
use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // account_hash (UUID)
    pub device_id: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String, // "access" or "refresh"
}

pub struct AuthService {
    db: Arc<Database>,
    config: Arc<Config>,
}

impl AuthService {
    pub fn new(db: Arc<Database>, config: Arc<Config>) -> Self {
        Self { db, config }
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
        Ok(hash.to_string())
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid hash format: {}", e)))?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    pub fn generate_token(&self, account_hash: &str, device_id: Option<&str>, token_type: &str) -> Result<(String, i64)> {
        let now = Utc::now();
        let exp = match token_type {
            "access" => now + Duration::hours(self.config.jwt_access_expiry_hours),
            "refresh" => now + Duration::days(self.config.jwt_refresh_expiry_days),
            "device" => now + Duration::days(365),
            _ => return Err(AppError::Internal("Invalid token type".to_string())),
        };
        
        let claims = Claims {
            sub: account_hash.to_string(),
            device_id: device_id.map(String::from),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: token_type.to_string(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )?;
        
        Ok((token, exp.timestamp()))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
    
    pub fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    pub async fn register(&self, email: &str, password: &str, display_name: &str) -> Result<(String, String, String, i64)> {
        let account_hash = uuid::Uuid::new_v4().to_string();
        let password_hash = self.hash_password(password)?;
        
        self.db.create_account(&account_hash, email, &password_hash, display_name).await?;
        
        let (access_token, _) = self.generate_token(&account_hash, None, "access")?;
        let (refresh_token, expires_at) = self.generate_token(&account_hash, None, "refresh")?;
        
        let token_id = uuid::Uuid::new_v4().to_string();
        let token_hash = self.hash_token(&refresh_token);
        self.db.create_auth_token(
            &token_id,
            &account_hash,
            None,
            &token_hash,
            "refresh",
            chrono::DateTime::from_timestamp(expires_at, 0).unwrap(),
        ).await?;
        
        Ok((account_hash, access_token, refresh_token, expires_at))
    }
    
    pub async fn login(&self, email: &str, password: &str, device_id: Option<&str>) -> Result<(String, String, String, i64, String)> {
        let account = self.db.get_account_by_email(email).await?;
        
        if !self.verify_password(password, &account.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }
        
        let (access_token, _) = self.generate_token(&account.account_hash, device_id, "access")?;
        let (refresh_token, expires_at) = self.generate_token(&account.account_hash, device_id, "refresh")?;
        
        let token_id = uuid::Uuid::new_v4().to_string();
        let token_hash = self.hash_token(&refresh_token);
        self.db.create_auth_token(
            &token_id,
            &account.account_hash,
            device_id,
            &token_hash,
            "refresh",
            chrono::DateTime::from_timestamp(expires_at, 0).unwrap(),
        ).await?;
        
        Ok((account.account_hash, access_token, refresh_token, expires_at, account.display_name))
    }
    
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<(String, String, i64)> {
        let claims = self.validate_token(refresh_token)?;
        
        if claims.token_type != "refresh" {
            return Err(AppError::TokenInvalid);
        }
        
        let token_hash = self.hash_token(refresh_token);
        let stored_token = self.db.get_auth_token(&token_hash).await?
            .ok_or(AppError::TokenInvalid)?;
        
        if stored_token.revoked {
            return Err(AppError::TokenInvalid);
        }
        
        self.db.revoke_token(&token_hash).await?;
        
        let (new_access, _) = self.generate_token(&claims.sub, claims.device_id.as_deref(), "access")?;
        let (new_refresh, expires_at) = self.generate_token(&claims.sub, claims.device_id.as_deref(), "refresh")?;
        
        let token_id = uuid::Uuid::new_v4().to_string();
        let new_token_hash = self.hash_token(&new_refresh);
        self.db.create_auth_token(
            &token_id,
            &claims.sub,
            claims.device_id.as_deref(),
            &new_token_hash,
            "refresh",
            chrono::DateTime::from_timestamp(expires_at, 0).unwrap(),
        ).await?;
        
        Ok((new_access, new_refresh, expires_at))
    }
    
    pub async fn logout(&self, refresh_token: &str, all_devices: bool) -> Result<()> {
        let claims = self.validate_token(refresh_token)?;
        
        if all_devices {
            self.db.revoke_all_tokens_for_account(&claims.sub).await?;
        } else {
            let token_hash = self.hash_token(refresh_token);
            self.db.revoke_token(&token_hash).await?;
        }
        
        Ok(())
    }
    
    pub async fn register_device(
        &self,
        account_hash: &str,
        name: &str,
        device_type: &str,
        os_version: Option<&str>,
        hardware_id: Option<&str>,
    ) -> Result<(String, String, i64)> {
        if let Some(hw_id) = hardware_id {
            if let Some(existing) = self.db.get_device_by_hardware_id(hw_id).await? {
                if existing.account_id == account_hash {
                    let (device_token, expires_at) = self.generate_token(account_hash, Some(&existing.id), "device")?;
                    return Ok((existing.id, device_token, expires_at));
                } else {
                    return Err(AppError::DeviceAlreadyRegistered);
                }
            }
        }
        
        let device_id = uuid::Uuid::new_v4().to_string();
        self.db.create_device(&device_id, account_hash, name, device_type, os_version, hardware_id).await?;
        
        let (device_token, expires_at) = self.generate_token(account_hash, Some(&device_id), "device")?;
        
        Ok((device_id, device_token, expires_at))
    }
    
    pub async fn verify_device(&self, device_token: &str) -> Result<(bool, String, String)> {
        let claims = self.validate_token(device_token)?;
        let device_id = claims.device_id.ok_or(AppError::TokenInvalid)?;
        self.db.update_device_last_seen(&device_id).await?;
        Ok((true, device_id, claims.sub))
    }
    
    pub async fn list_devices(&self, account_hash: &str) -> Result<Vec<crate::db::Device>> {
        self.db.list_devices_for_account(account_hash).await
    }
    
    pub async fn revoke_device(&self, device_id: &str) -> Result<()> {
        self.db.revoke_device(device_id).await
    }
    
    pub async fn change_password(&self, account_hash: &str, current_password: &str, new_password: &str) -> Result<()> {
        let account = self.db.get_account_by_hash(account_hash).await?;
        
        if !self.verify_password(current_password, &account.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }
        
        let new_hash = self.hash_password(new_password)?;
        self.db.update_password(account_hash, &new_hash).await?;
        self.db.revoke_all_tokens_for_account(account_hash).await?;
        
        Ok(())
    }
}
