use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Token invalid")]
    TokenInvalid,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("Device already registered")]
    DeviceAlreadyRegistered,
    
    #[error("Family not found")]
    FamilyNotFound,
    
    #[error("Child not found")]
    ChildNotFound,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("File not found")]
    FileNotFound,
    
    #[error("File too large")]
    FileTooLarge,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("S3 error: {0}")]
    S3(String),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::AuthenticationFailed(msg) => tonic::Status::unauthenticated(msg),
            AppError::InvalidCredentials => tonic::Status::unauthenticated("Invalid credentials"),
            AppError::TokenExpired => tonic::Status::unauthenticated("Token expired"),
            AppError::TokenInvalid => tonic::Status::unauthenticated("Token invalid"),
            AppError::UserNotFound => tonic::Status::not_found("User not found"),
            AppError::UserAlreadyExists => tonic::Status::already_exists("User already exists"),
            AppError::DeviceNotFound => tonic::Status::not_found("Device not found"),
            AppError::DeviceAlreadyRegistered => tonic::Status::already_exists("Device already registered"),
            AppError::FamilyNotFound => tonic::Status::not_found("Family not found"),
            AppError::ChildNotFound => tonic::Status::not_found("Child not found"),
            AppError::PermissionDenied => tonic::Status::permission_denied("Permission denied"),
            AppError::FileNotFound => tonic::Status::not_found("File not found"),
            AppError::FileTooLarge => tonic::Status::invalid_argument("File too large"),
            AppError::InvalidInput(msg) => tonic::Status::invalid_argument(msg),
            AppError::Database(e) => tonic::Status::internal(format!("Database error: {}", e)),
            AppError::Redis(e) => tonic::Status::internal(format!("Cache error: {}", e)),
            AppError::S3(msg) => tonic::Status::internal(format!("Storage error: {}", msg)),
            AppError::Jwt(e) => tonic::Status::unauthenticated(format!("JWT error: {}", e)),
            AppError::Internal(msg) => tonic::Status::internal(msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
