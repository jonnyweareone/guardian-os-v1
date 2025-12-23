//! Configuration for Guardian Selector

use serde::Deserialize;
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct GuardianConfig {
    pub guardian: GuardianSection,
}

#[derive(Debug, Deserialize)]
pub struct GuardianSection {
    pub version: String,
    pub family: FamilyConfig,
    pub verification: VerificationConfig,
    pub api: ApiConfig,
    pub build: BuildConfig,
}

#[derive(Debug, Deserialize)]
pub struct FamilyConfig {
    pub id: String,
    pub build_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VerificationConfig {
    pub algorithm: String,
    pub public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub supabase_url: String,
    pub supabase_anon_key: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub timestamp: String,
    pub device_type: String,
    pub base_image: String,
}

impl GuardianConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: GuardianConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

// Re-export for convenience
impl std::ops::Deref for GuardianConfig {
    type Target = GuardianSection;
    fn deref(&self) -> &Self::Target {
        &self.guardian
    }
}
