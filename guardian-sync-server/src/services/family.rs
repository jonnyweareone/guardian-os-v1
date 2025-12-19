use std::sync::Arc;
use rand::Rng;

use crate::db::Database;
use crate::error::{AppError, Result};

pub struct FamilyService {
    db: Arc<Database>,
}

impl FamilyService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    // Generate invite code
    fn generate_invite_code() -> String {
        let code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        code.to_uppercase()
    }
    
    // Create family
    pub async fn create_family(&self, name: &str, owner_id: &str) -> Result<(String, String)> {
        let family_id = uuid::Uuid::new_v4().to_string();
        let invite_code = Self::generate_invite_code();
        
        self.db.create_family(&family_id, name, owner_id, &invite_code).await?;
        
        Ok((family_id, invite_code))
    }
    
    // Get family
    pub async fn get_family(&self, family_id: &str) -> Result<crate::db::Family> {
        self.db.get_family(family_id).await
    }
    
    // Get family members
    pub async fn get_family_members(&self, family_id: &str) -> Result<Vec<crate::db::FamilyMember>> {
        self.db.get_family_members(family_id).await
    }
    
    // Get children
    pub async fn get_children(&self, family_id: &str) -> Result<Vec<crate::db::Child>> {
        self.db.get_children_for_family(family_id).await
    }
    
    // Invite member
    pub async fn invite_member(&self, family_id: &str, inviter_id: &str, email: &str, role: &str) -> Result<(String, String, i64)> {
        // Verify inviter has permission
        let family = self.db.get_family(family_id).await?;
        let members = self.db.get_family_members(family_id).await?;
        
        let inviter_member = members.iter()
            .find(|m| m.account_id == inviter_id)
            .ok_or(AppError::PermissionDenied)?;
        
        if inviter_member.role != "owner" && inviter_member.role != "admin" {
            return Err(AppError::PermissionDenied);
        }
        
        // Generate new invite code
        let invite_code = Self::generate_invite_code();
        let invitation_id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now().timestamp() + (7 * 24 * 60 * 60); // 7 days
        
        // Store invitation (you'd want a separate invitations table for this)
        // For now, we'll use the family's invite code
        
        Ok((invitation_id, invite_code, expires_at))
    }
    
    // Accept invitation
    pub async fn accept_invitation(&self, invite_code: &str, account_id: &str) -> Result<String> {
        let family = self.db.get_family_by_invite_code(invite_code).await?;
        
        // Add as member
        self.db.add_family_member(&family.id, account_id, "member").await?;
        
        Ok(family.id)
    }
    
    // Remove member
    pub async fn remove_member(&self, family_id: &str, requester_id: &str, target_id: &str) -> Result<()> {
        let family = self.db.get_family(family_id).await?;
        
        // Can't remove owner
        if target_id == family.owner_id {
            return Err(AppError::PermissionDenied);
        }
        
        // Must be owner or admin to remove others, or removing self
        if requester_id != target_id {
            let members = self.db.get_family_members(family_id).await?;
            let requester = members.iter()
                .find(|m| m.account_id == requester_id)
                .ok_or(AppError::PermissionDenied)?;
            
            if requester.role != "owner" && requester.role != "admin" {
                return Err(AppError::PermissionDenied);
            }
        }
        
        self.db.remove_family_member(family_id, target_id).await
    }
    
    // Add child
    pub async fn add_child(&self, family_id: &str, requester_id: &str, name: &str, age: i32) -> Result<String> {
        // Verify permission
        let members = self.db.get_family_members(family_id).await?;
        let requester = members.iter()
            .find(|m| m.account_id == requester_id)
            .ok_or(AppError::PermissionDenied)?;
        
        if requester.role != "owner" && requester.role != "admin" {
            return Err(AppError::PermissionDenied);
        }
        
        let child_id = uuid::Uuid::new_v4().to_string();
        self.db.add_child(&child_id, family_id, name, age).await?;
        
        Ok(child_id)
    }
    
    // Update child
    pub async fn update_child(&self, child_id: &str, requester_id: &str, name: &str, age: i32, avatar_url: Option<&str>) -> Result<()> {
        let child = self.db.get_child(child_id).await?;
        
        // Verify permission
        let members = self.db.get_family_members(&child.family_id).await?;
        let requester = members.iter()
            .find(|m| m.account_id == requester_id)
            .ok_or(AppError::PermissionDenied)?;
        
        if requester.role != "owner" && requester.role != "admin" {
            return Err(AppError::PermissionDenied);
        }
        
        self.db.update_child(child_id, name, age, avatar_url).await
    }
    
    // Remove child
    pub async fn remove_child(&self, child_id: &str, requester_id: &str) -> Result<()> {
        let child = self.db.get_child(child_id).await?;
        
        // Verify permission
        let members = self.db.get_family_members(&child.family_id).await?;
        let requester = members.iter()
            .find(|m| m.account_id == requester_id)
            .ok_or(AppError::PermissionDenied)?;
        
        if requester.role != "owner" && requester.role != "admin" {
            return Err(AppError::PermissionDenied);
        }
        
        self.db.delete_child(child_id).await
    }
    
    // Get child
    pub async fn get_child(&self, child_id: &str) -> Result<crate::db::Child> {
        self.db.get_child(child_id).await
    }
}
