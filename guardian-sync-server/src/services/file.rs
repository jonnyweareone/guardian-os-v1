use std::sync::Arc;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

use crate::config::Config;
use crate::db::Database;
use crate::error::{AppError, Result};

pub struct FileService {
    db: Arc<Database>,
    s3: S3Client,
    bucket: String,
}

impl FileService {
    pub async fn new(db: Arc<Database>, config: Arc<Config>) -> Result<Self> {
        // Configure S3 client for Peasoup (S3-compatible)
        let s3_config = aws_config::from_env()
            .endpoint_url(&config.s3_endpoint)
            .region(aws_config::Region::new(config.s3_region.clone()))
            .credentials_provider(aws_credential_types::Credentials::new(
                &config.s3_access_key,
                &config.s3_secret_key,
                None,
                None,
                "guardian-sync",
            ))
            .load()
            .await;
        
        let s3 = S3Client::new(&s3_config);
        
        Ok(Self {
            db,
            s3,
            bucket: config.s3_bucket.clone(),
        })
    }
    
    // Generate storage path
    fn storage_path(&self, account_id: &str, file_id: &str, filename: &str) -> String {
        format!("{}/{}/{}", account_id, file_id, filename)
    }
    
    // Upload file to S3
    pub async fn upload_file(
        &self,
        account_id: &str,
        filename: &str,
        file_type: &str,
        content_type: &str,
        data: Vec<u8>,
        checksum: &str,
    ) -> Result<(String, String)> {
        let file_id = uuid::Uuid::new_v4().to_string();
        let storage_path = self.storage_path(account_id, &file_id, filename);
        let size = data.len() as i64;
        
        // Upload to S3
        self.s3
            .put_object()
            .bucket(&self.bucket)
            .key(&storage_path)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        // Create DB record
        self.db.create_file(
            &file_id,
            account_id,
            filename,
            file_type,
            content_type,
            size,
            checksum,
            &storage_path,
        ).await?;
        
        Ok((file_id, storage_path))
    }
    
    // Download file from S3
    pub async fn download_file(&self, file_id: &str, account_id: &str) -> Result<(crate::db::FileRecord, Vec<u8>)> {
        let file = self.db.get_file(file_id).await?;
        
        // Verify ownership
        if file.account_id != account_id {
            return Err(AppError::PermissionDenied);
        }
        
        // Download from S3
        let response = self.s3
            .get_object()
            .bucket(&self.bucket)
            .key(&file.storage_path)
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?
            .into_bytes()
            .to_vec();
        
        Ok((file, data))
    }
    
    // Get presigned URL for upload
    pub async fn get_upload_url(
        &self,
        account_id: &str,
        filename: &str,
        content_type: &str,
        expires_in_secs: u64,
    ) -> Result<(String, String, i64)> {
        let file_id = uuid::Uuid::new_v4().to_string();
        let storage_path = self.storage_path(account_id, &file_id, filename);
        
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        let presigned = self.s3
            .put_object()
            .bucket(&self.bucket)
            .key(&storage_path)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        let expires_at = chrono::Utc::now().timestamp() + expires_in_secs as i64;
        
        Ok((presigned.uri().to_string(), file_id, expires_at))
    }
    
    // Get presigned URL for download
    pub async fn get_download_url(
        &self,
        file_id: &str,
        account_id: &str,
        expires_in_secs: u64,
    ) -> Result<(String, i64)> {
        let file = self.db.get_file(file_id).await?;
        
        // Verify ownership
        if file.account_id != account_id {
            return Err(AppError::PermissionDenied);
        }
        
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        let presigned = self.s3
            .get_object()
            .bucket(&self.bucket)
            .key(&file.storage_path)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        let expires_at = chrono::Utc::now().timestamp() + expires_in_secs as i64;
        
        Ok((presigned.uri().to_string(), expires_at))
    }
    
    // List files
    pub async fn list_files(
        &self,
        account_id: &str,
        file_type: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<crate::db::FileRecord>> {
        self.db.list_files(account_id, file_type, limit, offset).await
    }
    
    // Get file metadata
    pub async fn get_file_metadata(&self, file_id: &str, account_id: &str) -> Result<crate::db::FileRecord> {
        let file = self.db.get_file(file_id).await?;
        
        if file.account_id != account_id {
            return Err(AppError::PermissionDenied);
        }
        
        Ok(file)
    }
    
    // Delete file
    pub async fn delete_file(&self, file_id: &str, account_id: &str) -> Result<()> {
        let file = self.db.get_file(file_id).await?;
        
        if file.account_id != account_id {
            return Err(AppError::PermissionDenied);
        }
        
        // Delete from S3
        self.s3
            .delete_object()
            .bucket(&self.bucket)
            .key(&file.storage_path)
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;
        
        // Delete from DB
        self.db.delete_file(file_id).await?;
        
        Ok(())
    }
    
    // Get storage usage
    pub async fn get_storage_usage(&self, account_id: &str) -> Result<i64> {
        self.db.get_total_storage_used(account_id).await
    }
}
