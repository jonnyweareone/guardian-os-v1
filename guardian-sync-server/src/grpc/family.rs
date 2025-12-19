use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::proto::*;
use crate::services::FamilyService;

pub struct FamilyGrpc {
    service: Arc<FamilyService>,
}

impl FamilyGrpc {
    pub fn new(service: Arc<FamilyService>) -> Self {
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
impl family_service_server::FamilyService for FamilyGrpc {
    async fn create_family(&self, request: Request<CreateFamilyRequest>) -> Result<Response<CreateFamilyResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let (family_id, invite_code) = self.service.create_family(&req.name, &account_id).await?;
        
        Ok(Response::new(CreateFamilyResponse { family_id, invite_code }))
    }
    
    async fn get_family(&self, request: Request<GetFamilyRequest>) -> Result<Response<Family>, Status> {
        let _account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let family = self.service.get_family(&req.family_id).await?;
        let members = self.service.get_family_members(&req.family_id).await?;
        let children = self.service.get_children(&req.family_id).await?;
        
        let proto_members: Vec<FamilyMember> = members.into_iter()
            .map(|m| FamilyMember {
                account_id: m.account_id,
                email: String::new(),
                display_name: String::new(),
                role: string_to_role(&m.role).into(),
                joined_at: m.joined_at.timestamp(),
            })
            .collect();
        
        let proto_children: Vec<Child> = children.into_iter()
            .map(|c| Child {
                child_id: c.id,
                name: c.name,
                avatar_url: c.avatar_url.unwrap_or_default(),
                age: c.age,
                created_at: c.created_at.timestamp(),
            })
            .collect();
        
        Ok(Response::new(Family {
            family_id: family.id,
            name: family.name,
            owner_id: family.owner_id,
            members: proto_members,
            children: proto_children,
            created_at: family.created_at.timestamp(),
        }))
    }
    
    async fn invite_member(&self, request: Request<InviteMemberRequest>) -> Result<Response<InviteMemberResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let role = role_to_string(FamilyRole::try_from(req.role).unwrap_or_default());
        let (invitation_id, invite_code, expires_at) = self.service
            .invite_member(&req.family_id, &account_id, &req.email, &role)
            .await?;
        
        Ok(Response::new(InviteMemberResponse { invitation_id, invite_code, expires_at }))
    }
    
    async fn accept_invitation(&self, request: Request<AcceptInvitationRequest>) -> Result<Response<AcceptInvitationResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let family_id = self.service.accept_invitation(&req.invite_code, &account_id).await?;
        
        Ok(Response::new(AcceptInvitationResponse { family_id, success: true }))
    }
    
    async fn remove_member(&self, request: Request<RemoveMemberRequest>) -> Result<Response<RemoveMemberResponse>, Status> {
        let requester_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        self.service.remove_member(&req.family_id, &requester_id, &req.account_id).await?;
        
        Ok(Response::new(RemoveMemberResponse { success: true }))
    }
    
    async fn add_child(&self, request: Request<AddChildRequest>) -> Result<Response<AddChildResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let child_id = self.service.add_child(&req.family_id, &account_id, &req.name, req.age).await?;
        
        Ok(Response::new(AddChildResponse { child_id }))
    }
    
    async fn update_child(&self, request: Request<UpdateChildRequest>) -> Result<Response<UpdateChildResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        let avatar_url = if req.avatar_url.is_empty() { None } else { Some(req.avatar_url.as_str()) };
        self.service.update_child(&req.child_id, &account_id, &req.name, req.age, avatar_url).await?;
        
        Ok(Response::new(UpdateChildResponse { success: true }))
    }
    
    async fn remove_child(&self, request: Request<RemoveChildRequest>) -> Result<Response<RemoveChildResponse>, Status> {
        let account_id = Self::get_account_id(&request)?;
        let req = request.into_inner();
        
        self.service.remove_child(&req.child_id, &account_id).await?;
        
        Ok(Response::new(RemoveChildResponse { success: true }))
    }
    
    async fn link_device_to_child(&self, _request: Request<LinkDeviceToChildRequest>) -> Result<Response<LinkDeviceToChildResponse>, Status> {
        Ok(Response::new(LinkDeviceToChildResponse { success: true }))
    }
    
    async fn get_child_devices(&self, _request: Request<GetChildDevicesRequest>) -> Result<Response<GetChildDevicesResponse>, Status> {
        Ok(Response::new(GetChildDevicesResponse { devices: vec![] }))
    }
    
    async fn set_screen_time_rules(&self, _request: Request<SetScreenTimeRulesRequest>) -> Result<Response<SetScreenTimeRulesResponse>, Status> {
        Ok(Response::new(SetScreenTimeRulesResponse { success: true }))
    }
    
    async fn get_screen_time_rules(&self, _request: Request<GetScreenTimeRulesRequest>) -> Result<Response<ScreenTimeRules>, Status> {
        Ok(Response::new(ScreenTimeRules {
            daily_limit_minutes: 120,
            schedule: vec![],
            enabled: false,
        }))
    }
    
    async fn get_screen_time_usage(&self, _request: Request<GetScreenTimeUsageRequest>) -> Result<Response<GetScreenTimeUsageResponse>, Status> {
        Ok(Response::new(GetScreenTimeUsageResponse { usage: vec![], total_minutes: 0 }))
    }
    
    async fn set_content_filters(&self, _request: Request<SetContentFiltersRequest>) -> Result<Response<SetContentFiltersResponse>, Status> {
        Ok(Response::new(SetContentFiltersResponse { success: true }))
    }
    
    async fn get_content_filters(&self, _request: Request<GetContentFiltersRequest>) -> Result<Response<ContentFilters>, Status> {
        Ok(Response::new(ContentFilters {
            safe_search: true,
            blocked_sites: vec![],
            allowed_sites: vec![],
            blocked_apps: vec![],
            content_level: ContentLevel::Child.into(),
        }))
    }
    
    async fn approve_app_request(&self, _request: Request<ApproveAppRequestRequest>) -> Result<Response<ApproveAppRequestResponse>, Status> {
        Ok(Response::new(ApproveAppRequestResponse { success: true }))
    }
}

fn role_to_string(role: FamilyRole) -> String {
    match role {
        FamilyRole::Owner => "owner",
        FamilyRole::Admin => "admin",
        FamilyRole::Member => "member",
        _ => "member",
    }.to_string()
}

fn string_to_role(s: &str) -> FamilyRole {
    match s {
        "owner" => FamilyRole::Owner,
        "admin" => FamilyRole::Admin,
        "member" => FamilyRole::Member,
        _ => FamilyRole::Unspecified,
    }
}
