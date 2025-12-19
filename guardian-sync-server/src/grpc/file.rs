use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;

use crate::proto::*;
use crate::services::FileService;

pub struct FileGrpc {
    service: Arc<FileService>,
}

impl FileGrpc {
    pub fn new(service: Arc<FileService>) -> Self {
        Self { service }
    }
    
    fn get_account_id<T>(request: &Request<T>) -> Result<String, Status> {
        request.metadata()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .ok_or_else(|| Status::unauthenticated("Missing account ID"))
    }
}

#[tonic::async_trait]
impl file_service_server::FileService for FileGrpc {
    async fn upload_file(&self, request: Request<Streaming<UploadFileRequest>>) -> Result<Response<UploadFileResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let mut stream = request.into_inner();
        
        let first = stream.next().await
            .ok_or_else(|| Status::invalid_argument("Empty stream"))?
            .map_err(|e| Status::internal(e.to_string()))?;
        
        let header = match first.data {
            Some(upload_file_request::Data::Header(h)) => h,
            _ => return Err(Status::invalid_argument("First message must be header")),
        };
        
        let mut data = Vec::with_capacity(header.size as usize);
        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| Status::internal(e.to_string()))?;
            if let Some(upload_file_request::Data::Chunk(chunk)) = msg.data {
                data.extend_from_slice(&chunk);
            }
        }
        
        let file_type = file_type_to_string(FileType::try_from(header.file_type).unwrap_or_default());
        
        let (file_id, storage_path) = self.service
            .upload_file(&account_id, &header.filename, &file_type, &header.content_type, data, &header.checksum_sha256)
            .await?;
        
        Ok(Response::new(UploadFileResponse {
            file_id,
            storage_path,
            uploaded_at: chrono::Utc::now().timestamp(),
        }))
    }
    
    type DownloadFileStream = tokio_stream::wrappers::ReceiverStream<Result<DownloadFileResponse, Status>>;
    
    async fn download_file(&self, request: Request<DownloadFileRequest>) -> Result<Response<Self::DownloadFileStream>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let (file_record, data) = self.service.download_file(&req.file_id, &account_id).await?;
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let metadata = FileMetadata {
            file_id: file_record.id,
            filename: file_record.filename,
            file_type: string_to_file_type(&file_record.file_type).into(),
            size: file_record.size,
            content_type: file_record.content_type,
            checksum_sha256: file_record.checksum,
            created_at: file_record.created_at.timestamp(),
            updated_at: file_record.updated_at.timestamp(),
            metadata: std::collections::HashMap::new(),
        };
        
        tokio::spawn(async move {
            let _ = tx.send(Ok(DownloadFileResponse {
                data: Some(download_file_response::Data::Metadata(metadata)),
            })).await;
            
            const CHUNK_SIZE: usize = 64 * 1024;
            for chunk in data.chunks(CHUNK_SIZE) {
                let _ = tx.send(Ok(DownloadFileResponse {
                    data: Some(download_file_response::Data::Chunk(chunk.to_vec())),
                })).await;
            }
        });
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
    
    async fn list_files(&self, request: Request<ListFilesRequest>) -> Result<Response<ListFilesResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let file_type = if req.file_type == 0 { None } else {
            Some(file_type_to_string(FileType::try_from(req.file_type).unwrap_or_default()))
        };
        
        let limit = if req.limit == 0 { 50 } else { req.limit };
        
        let files = self.service.list_files(&account_id, file_type.as_deref(), limit, 0).await?;
        
        let proto_files: Vec<FileMetadata> = files.into_iter()
            .map(|f| FileMetadata {
                file_id: f.id,
                filename: f.filename,
                file_type: string_to_file_type(&f.file_type).into(),
                size: f.size,
                content_type: f.content_type,
                checksum_sha256: f.checksum,
                created_at: f.created_at.timestamp(),
                updated_at: f.updated_at.timestamp(),
                metadata: std::collections::HashMap::new(),
            })
            .collect();
        
        Ok(Response::new(ListFilesResponse {
            files: proto_files,
            next_cursor: String::new(),
            total_count: 0,
        }))
    }
    
    async fn get_file_metadata(&self, request: Request<GetFileMetadataRequest>) -> Result<Response<FileMetadata>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let file = self.service.get_file_metadata(&req.file_id, &account_id).await?;
        
        Ok(Response::new(FileMetadata {
            file_id: file.id,
            filename: file.filename,
            file_type: string_to_file_type(&file.file_type).into(),
            size: file.size,
            content_type: file.content_type,
            checksum_sha256: file.checksum,
            created_at: file.created_at.timestamp(),
            updated_at: file.updated_at.timestamp(),
            metadata: std::collections::HashMap::new(),
        }))
    }
    
    async fn delete_file(&self, request: Request<DeleteFileRequest>) -> Result<Response<DeleteFileResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        self.service.delete_file(&req.file_id, &account_id).await?;
        Ok(Response::new(DeleteFileResponse { success: true }))
    }
    
    async fn get_presigned_url(&self, request: Request<GetPresignedUrlRequest>) -> Result<Response<GetPresignedUrlResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let expires_in = if req.expires_in_seconds == 0 { 3600 } else { req.expires_in_seconds as u64 };
        
        match req.operation() {
            UrlOperation::Upload => {
                let header = req.header.ok_or_else(|| Status::invalid_argument("Header required for upload"))?;
                let (url, file_id, expires_at) = self.service
                    .get_upload_url(&account_id, &header.filename, &header.content_type, expires_in)
                    .await?;
                
                Ok(Response::new(GetPresignedUrlResponse {
                    url, file_id, expires_at,
                    required_headers: std::collections::HashMap::new(),
                }))
            },
            UrlOperation::Download => {
                let (url, expires_at) = self.service
                    .get_download_url(&req.file_id, &account_id, expires_in)
                    .await?;
                
                Ok(Response::new(GetPresignedUrlResponse {
                    url, file_id: req.file_id, expires_at,
                    required_headers: std::collections::HashMap::new(),
                }))
            },
            _ => Err(Status::invalid_argument("Invalid operation")),
        }
    }
}

fn file_type_to_string(ft: FileType) -> String {
    match ft {
        FileType::Wallpaper => "wallpaper",
        FileType::Theme => "theme",
        FileType::Config => "config",
        FileType::IconPack => "icon_pack",
        FileType::Font => "font",
        FileType::Document => "document",
        _ => "unknown",
    }.to_string()
}

fn string_to_file_type(s: &str) -> FileType {
    match s {
        "wallpaper" => FileType::Wallpaper,
        "theme" => FileType::Theme,
        "config" => FileType::Config,
        "icon_pack" => FileType::IconPack,
        "font" => FileType::Font,
        "document" => FileType::Document,
        _ => FileType::Unspecified,
    }
}
