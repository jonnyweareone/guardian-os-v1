pub mod config;
pub mod db;
pub mod error;
pub mod grpc;
pub mod services;

// Proto module - all services are in same package
pub mod proto {
    tonic::include_proto!("guardian.sync.v1");
}

// Re-export for convenience
pub use proto::auth_service_server;
pub use proto::sync_service_server;
pub use proto::file_service_server;
pub use proto::family_service_server;
