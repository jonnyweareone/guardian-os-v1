use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::error::{AppError, Result};
use super::Database;

#[derive(Debug, Clone, FromRow)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub invite_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct FamilyMember {
    pub id: String,
    pub family_id: String,
    pub account_id: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Child {
    pub id: String,
    pub family_id: String,
    pub name: String,
    pub age: i32,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Database {
    // Family operations
    pub async fn create_family(&self, id: &str, name: &str, owner_id: &str, invite_code: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO families (id, name, owner_id, invite_code, created_at) VALUES (?, ?, ?, ?, NOW())"
        )
        .bind(id)
        .bind(name)
        .bind(owner_id)
        .bind(invite_code)
        .execute(self.pool())
        .await?;
        
        // Add owner as member
        self.add_family_member(id, owner_id, "owner").await?;
        
        Ok(())
    }
    
    pub async fn get_family(&self, id: &str) -> Result<Family> {
        sqlx::query_as::<_, Family>(
            "SELECT id, name, owner_id, invite_code, created_at FROM families WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::FamilyNotFound)
    }
    
    pub async fn get_family_by_invite_code(&self, invite_code: &str) -> Result<Family> {
        sqlx::query_as::<_, Family>(
            "SELECT id, name, owner_id, invite_code, created_at FROM families WHERE invite_code = ?"
        )
        .bind(invite_code)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::FamilyNotFound)
    }
    
    // Family members
    pub async fn add_family_member(&self, family_id: &str, account_id: &str, role: &str) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO family_members (id, family_id, account_id, role, joined_at) VALUES (?, ?, ?, ?, NOW())"
        )
        .bind(&id)
        .bind(family_id)
        .bind(account_id)
        .bind(role)
        .execute(self.pool())
        .await?;
        Ok(())
    }
    
    pub async fn get_family_members(&self, family_id: &str) -> Result<Vec<FamilyMember>> {
        let members = sqlx::query_as::<_, FamilyMember>(
            "SELECT id, family_id, account_id, role, joined_at FROM family_members WHERE family_id = ?"
        )
        .bind(family_id)
        .fetch_all(self.pool())
        .await?;
        Ok(members)
    }
    
    pub async fn remove_family_member(&self, family_id: &str, account_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM family_members WHERE family_id = ? AND account_id = ?")
            .bind(family_id)
            .bind(account_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    // Children
    pub async fn add_child(&self, id: &str, family_id: &str, name: &str, age: i32) -> Result<()> {
        sqlx::query(
            "INSERT INTO children (id, family_id, name, age, created_at) VALUES (?, ?, ?, ?, NOW())"
        )
        .bind(id)
        .bind(family_id)
        .bind(name)
        .bind(age)
        .execute(self.pool())
        .await?;
        Ok(())
    }
    
    pub async fn get_child(&self, id: &str) -> Result<Child> {
        sqlx::query_as::<_, Child>(
            "SELECT id, family_id, name, age, avatar_url, created_at FROM children WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or(AppError::ChildNotFound)
    }
    
    pub async fn get_children_for_family(&self, family_id: &str) -> Result<Vec<Child>> {
        let children = sqlx::query_as::<_, Child>(
            "SELECT id, family_id, name, age, avatar_url, created_at FROM children WHERE family_id = ?"
        )
        .bind(family_id)
        .fetch_all(self.pool())
        .await?;
        Ok(children)
    }
    
    pub async fn update_child(&self, id: &str, name: &str, age: i32, avatar_url: Option<&str>) -> Result<()> {
        sqlx::query("UPDATE children SET name = ?, age = ?, avatar_url = ? WHERE id = ?")
            .bind(name)
            .bind(age)
            .bind(avatar_url)
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
    
    pub async fn delete_child(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM children WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
