use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Server
    pub grpc_port: u16,
    pub http_port: u16,
    pub host: String,
    
    // Database
    pub database_url: String,
    
    // Redis
    pub redis_url: String,
    
    // S3 Storage
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    
    // JWT
    pub jwt_secret: String,
    pub jwt_access_expiry_hours: i64,
    pub jwt_refresh_expiry_days: i64,
    
    // Domain
    pub domain: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            // Server
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .unwrap_or(50051),
            http_port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            
            // Database
            database_url: format!(
                "mysql://{}:{}@{}:{}/{}",
                env::var("DB_USER").unwrap_or_else(|_| "guardian".to_string()),
                env::var("DB_PASS")?,
                env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string()),
                env::var("DB_NAME").unwrap_or_else(|_| "guardian_sync".to_string()),
            ),
            
            // Redis
            redis_url: format!(
                "redis://:{}@{}:{}",
                env::var("REDIS_PASSWORD").unwrap_or_default(),
                env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
            ),
            
            // S3
            s3_endpoint: env::var("S3_ENDPOINT")?,
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "eu-west-1".to_string()),
            s3_bucket: env::var("S3_BUCKET")?,
            s3_access_key: env::var("S3_ACCESS_KEY")?,
            s3_secret_key: env::var("S3_SECRET_KEY")?,
            
            // JWT
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                // Generate random secret if not set (dev only!)
                use rand::Rng;
                let secret: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect();
                tracing::warn!("JWT_SECRET not set, using random secret (dev mode)");
                secret
            }),
            jwt_access_expiry_hours: env::var("JWT_ACCESS_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            jwt_refresh_expiry_days: env::var("JWT_REFRESH_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
                
            // Domain
            domain: env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
        })
    }
}
