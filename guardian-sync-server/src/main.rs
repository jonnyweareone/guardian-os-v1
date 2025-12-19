use std::sync::Arc;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use guardian_sync_server::config::Config;
use guardian_sync_server::db::Database;
use guardian_sync_server::services::{AuthService, SyncService, FileService, FamilyService};
use guardian_sync_server::grpc::{AuthGrpc, SyncGrpc, FileGrpc, FamilyGrpc};
use guardian_sync_server::proto::{
    auth_service_server::AuthServiceServer,
    sync_service_server::SyncServiceServer,
    file_service_server::FileServiceServer,
    family_service_server::FamilyServiceServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "guardian_sync_server=info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env().expect("Failed to load configuration"));
    
    info!("Starting Guardian Sync Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Domain: {}", config.domain);
    
    // Connect to database
    info!("Connecting to database...");
    let db = Arc::new(Database::connect(&config.database_url).await?);
    info!("Database connected");
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db.clone(), config.clone()));
    let sync_service = Arc::new(SyncService::new(db.clone()));
    let file_service = Arc::new(FileService::new(db.clone(), config.clone()).await?);
    let family_service = Arc::new(FamilyService::new(db.clone()));
    
    // Create gRPC handlers
    let auth_grpc = AuthGrpc::new(auth_service);
    let sync_grpc = SyncGrpc::new(sync_service);
    let file_grpc = FileGrpc::new(file_service);
    let family_grpc = FamilyGrpc::new(family_service);
    
    // Build gRPC server
    let addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port).parse()?;
    
    info!("Starting gRPC server on {}", addr);
    
    Server::builder()
        .add_service(AuthServiceServer::new(auth_grpc))
        .add_service(SyncServiceServer::new(sync_grpc))
        .add_service(FileServiceServer::new(file_grpc))
        .add_service(FamilyServiceServer::new(family_grpc))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    
    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
