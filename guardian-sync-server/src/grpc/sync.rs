use std::sync::Arc;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use crate::proto::*;
use crate::services::SyncService;

pub struct SyncGrpc {
    service: Arc<SyncService>,
}

impl SyncGrpc {
    pub fn new(service: Arc<SyncService>) -> Self {
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
impl sync_service_server::SyncService for SyncGrpc {
    async fn push_settings(&self, request: Request<PushSettingsRequest>) -> Result<Response<PushSettingsResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let entries: Vec<(String, Vec<u8>, String, i64)> = req.entries.into_iter()
            .map(|e| {
                let cat = category_to_string(e.category());
                (e.key, e.value, cat, e.modified_at)
            })
            .collect();
        
        let (server_timestamp, conflicts) = self.service
            .push_settings(&account_id, &req.device_id, entries)
            .await?;
        
        let proto_conflicts: Vec<ConflictEntry> = conflicts.into_iter()
            .map(|(key, server_value, client_value, server_modified, client_modified)| {
                ConflictEntry {
                    key,
                    server_value,
                    client_value,
                    server_modified,
                    client_modified,
                }
            })
            .collect();
        
        Ok(Response::new(PushSettingsResponse {
            success: true,
            server_timestamp,
            conflicts: proto_conflicts,
        }))
    }
    
    async fn pull_settings(&self, request: Request<PullSettingsRequest>) -> Result<Response<PullSettingsResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let categories: Option<Vec<String>> = if req.categories.is_empty() {
            None
        } else {
            Some(req.categories.iter().map(|c| category_to_string(SettingsCategory::try_from(*c).unwrap_or_default())).collect())
        };
        
        let (entries, server_timestamp) = self.service
            .pull_settings(&account_id, req.since_timestamp, categories)
            .await?;
        
        let proto_entries: Vec<SettingsEntry> = entries.into_iter()
            .map(|e| SettingsEntry {
                key: e.key,
                value: e.value,
                category: string_to_category(&e.category).into(),
                modified_at: e.modified_at.timestamp(),
                checksum: e.checksum,
            })
            .collect();
        
        Ok(Response::new(PullSettingsResponse {
            entries: proto_entries,
            server_timestamp,
            has_more: false,
            continuation_token: String::new(),
        }))
    }
    
    async fn get_settings_diff(&self, request: Request<GetSettingsDiffRequest>) -> Result<Response<GetSettingsDiffResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let (entries, server_timestamp) = self.service
            .pull_settings(&account_id, req.since_timestamp, None)
            .await?;
        
        let changes: Vec<SettingsDiffEntry> = entries.into_iter()
            .map(|e| SettingsDiffEntry {
                key: e.key,
                operation: DiffOperation::Update.into(),
                new_value: e.value,
                modified_at: e.modified_at.timestamp(),
            })
            .collect();
        
        Ok(Response::new(GetSettingsDiffResponse {
            changes,
            server_timestamp,
        }))
    }
    
    type StreamChangesStream = ReceiverStream<Result<SettingsChange, Status>>;
    
    async fn stream_changes(&self, _request: Request<StreamChangesRequest>) -> Result<Response<Self::StreamChangesStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        drop(tx); // Empty stream for now
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    
    async fn resolve_conflict(&self, _request: Request<ResolveConflictRequest>) -> Result<Response<ResolveConflictResponse>, Status> {
        Ok(Response::new(ResolveConflictResponse {
            success: true,
            resolved_entry: None,
        }))
    }
    
    async fn get_sync_status(&self, _request: Request<GetSyncStatusRequest>) -> Result<Response<GetSyncStatusResponse>, Status> {
        Ok(Response::new(GetSyncStatusResponse {
            last_sync_timestamp: chrono::Utc::now().timestamp(),
            pending_changes: 0,
            conflicts_count: 0,
            state: SyncState::Idle.into(),
        }))
    }
}

fn category_to_string(cat: SettingsCategory) -> String {
    match cat {
        SettingsCategory::Desktop => "desktop",
        SettingsCategory::Panel => "panel",
        SettingsCategory::Dock => "dock",
        SettingsCategory::Keyboard => "keyboard",
        SettingsCategory::Display => "display",
        SettingsCategory::Power => "power",
        SettingsCategory::Network => "network",
        SettingsCategory::Apps => "apps",
        SettingsCategory::Theme => "theme",
        SettingsCategory::Guardian => "guardian",
        _ => "unknown",
    }.to_string()
}

fn string_to_category(s: &str) -> SettingsCategory {
    match s {
        "desktop" => SettingsCategory::Desktop,
        "panel" => SettingsCategory::Panel,
        "dock" => SettingsCategory::Dock,
        "keyboard" => SettingsCategory::Keyboard,
        "display" => SettingsCategory::Display,
        "power" => SettingsCategory::Power,
        "network" => SettingsCategory::Network,
        "apps" => SettingsCategory::Apps,
        "theme" => SettingsCategory::Theme,
        "guardian" => SettingsCategory::Guardian,
        _ => SettingsCategory::Unspecified,
    }
}
