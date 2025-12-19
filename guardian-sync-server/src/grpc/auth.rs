use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::proto::*;
use crate::services::AuthService;

pub struct AuthGrpc {
    service: Arc<AuthService>,
}

impl AuthGrpc {
    pub fn new(service: Arc<AuthService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl auth_service_server::AuthService for AuthGrpc {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        tracing::info!("Register request received");
        let req = request.into_inner();
        tracing::info!("Registering user: {}", req.email);
        
        let (account_id, access_token, refresh_token, expires_at) = self.service
            .register(&req.email, &req.password, &req.display_name)
            .await
            .map_err(|e| {
                tracing::error!("Registration failed: {:?}", e);
                e
            })?;
        tracing::info!("Registration successful for account: {}", account_id);
        
        Ok(Response::new(RegisterResponse {
            account_id,
            access_token,
            refresh_token,
            expires_at,
        }))
    }
    
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        
        let device_id = if req.device_id.is_empty() { None } else { Some(req.device_id.as_str()) };
        
        let (account_id, access_token, refresh_token, expires_at, display_name) = self.service
            .login(&req.email, &req.password, device_id)
            .await?;
        
        Ok(Response::new(LoginResponse {
            account_id,
            access_token,
            refresh_token,
            expires_at,
            display_name,
        }))
    }
    
    async fn refresh_token(&self, request: Request<RefreshTokenRequest>) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        
        let (access_token, refresh_token, expires_at) = self.service
            .refresh_token(&req.refresh_token)
            .await?;
        
        Ok(Response::new(RefreshTokenResponse {
            access_token,
            refresh_token,
            expires_at,
        }))
    }
    
    async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        
        self.service.logout(&req.refresh_token, req.all_devices).await?;
        
        Ok(Response::new(LogoutResponse { success: true }))
    }
    
    async fn register_device(&self, request: Request<RegisterDeviceRequest>) -> Result<Response<RegisterDeviceResponse>, Status> {
        let account_id = request.metadata()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing account ID"))?
            .to_string();
        
        let req = request.into_inner();
        
        let hardware_id = if req.hardware_id.is_empty() { None } else { Some(req.hardware_id.as_str()) };
        let os_version = if req.os_version.is_empty() { None } else { Some(req.os_version.as_str()) };
        
        let (device_id, device_token, token_expires_at) = self.service
            .register_device(&account_id, &req.device_name, &req.device_type, os_version, hardware_id)
            .await?;
        
        Ok(Response::new(RegisterDeviceResponse {
            device_id,
            device_token,
            token_expires_at,
        }))
    }
    
    async fn verify_device(&self, request: Request<VerifyDeviceRequest>) -> Result<Response<VerifyDeviceResponse>, Status> {
        let req = request.into_inner();
        
        let (valid, device_id, account_id) = self.service
            .verify_device(&req.device_token)
            .await?;
        
        Ok(Response::new(VerifyDeviceResponse {
            valid,
            device_id,
            account_id,
        }))
    }
    
    async fn list_devices(&self, request: Request<ListDevicesRequest>) -> Result<Response<ListDevicesResponse>, Status> {
        let account_id = request.metadata()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing account ID"))?
            .to_string();
        
        let current_device_id = request.metadata()
            .get("x-device-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        
        let devices = self.service.list_devices(&account_id).await?;
        
        let proto_devices: Vec<Device> = devices.into_iter().map(|d| {
            Device {
                device_id: d.id.clone(),
                device_name: d.name,
                device_type: d.device_type,
                os_version: d.os_version.unwrap_or_default(),
                last_seen: d.last_seen_at.timestamp(),
                is_current: current_device_id.as_ref().map(|id| id == &d.id).unwrap_or(false),
                created_at: d.created_at.timestamp(),
            }
        }).collect();
        
        Ok(Response::new(ListDevicesResponse { devices: proto_devices }))
    }
    
    async fn revoke_device(&self, request: Request<RevokeDeviceRequest>) -> Result<Response<RevokeDeviceResponse>, Status> {
        let req = request.into_inner();
        self.service.revoke_device(&req.device_id).await?;
        Ok(Response::new(RevokeDeviceResponse { success: true }))
    }
    
    async fn change_password(&self, request: Request<ChangePasswordRequest>) -> Result<Response<ChangePasswordResponse>, Status> {
        let account_id = request.metadata()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing account ID"))?
            .to_string();
        
        let req = request.into_inner();
        self.service.change_password(&account_id, &req.current_password, &req.new_password).await?;
        Ok(Response::new(ChangePasswordResponse { success: true }))
    }
    
    async fn request_password_reset(&self, _request: Request<RequestPasswordResetRequest>) -> Result<Response<RequestPasswordResetResponse>, Status> {
        Ok(Response::new(RequestPasswordResetResponse {
            success: true,
            message: "If an account exists, a reset link has been sent.".to_string(),
        }))
    }
    
    async fn reset_password(&self, _request: Request<ResetPasswordRequest>) -> Result<Response<ResetPasswordResponse>, Status> {
        Ok(Response::new(ResetPasswordResponse { success: false }))
    }
}
