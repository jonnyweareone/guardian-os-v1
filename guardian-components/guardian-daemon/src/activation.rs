//! Guardian OS Activation Service
//! 
//! Handles first-boot activation and cryptographic verification
//! Ensures the ISO is being used by the correct family account

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::fs;
use std::path::Path;
use tracing::{info, error, warn};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

const CONFIG_PATH: &str = "/etc/guardian/config.json";
const ACTIVATION_STATE_PATH: &str = "/var/lib/guardian/activation_state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianConfig {
    pub family_id: String,
    pub build_id: String,
    pub profile_hash: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub nakama_host: String,
    pub nakama_port: u16,
    pub verification_public_key: String,
    pub activated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationState {
    pub activated: bool,
    pub device_id: Option<String>,
    pub child_id: Option<String>,
    pub child_name: Option<String>,
    pub token_data: Option<String>,
    pub signature: Option<String>,
    pub activated_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct ActivationRequest {
    email: String,
    password: String,
    family_id: String,
    build_id: String,
    device_hash: String,
    device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActivationResponse {
    success: bool,
    device_id: Option<String>,
    child_name: Option<String>,
    token_data: Option<String>,
    signature: Option<String>,
    profile: Option<serde_json::Value>,
    message: Option<String>,
    error: Option<String>,
    code: Option<String>,
    brick: Option<bool>,
}

pub struct ActivationService {
    config: GuardianConfig,
    client: Client,
    state: ActivationState,
}

impl ActivationService {
    pub fn new() -> Result<Self> {
        // Load config from baked-in file
        let config_str = fs::read_to_string(CONFIG_PATH)
            .context("Failed to read Guardian config - is this a genuine Guardian OS?")?;
        
        let config: GuardianConfig = serde_json::from_str(&config_str)
            .context("Failed to parse Guardian config")?;
        
        // Load existing activation state if present
        let state = if Path::new(ACTIVATION_STATE_PATH).exists() {
            let state_str = fs::read_to_string(ACTIVATION_STATE_PATH)?;
            serde_json::from_str(&state_str)?
        } else {
            ActivationState {
                activated: false,
                device_id: None,
                child_id: None,
                child_name: None,
                token_data: None,
                signature: None,
                activated_at: None,
            }
        };
        
        Ok(Self {
            config,
            client: Client::new(),
            state,
        })
    }
    
    /// Check if this device is already activated
    pub fn is_activated(&self) -> bool {
        self.state.activated
    }
    
    /// Get the family ID baked into this ISO
    pub fn get_family_id(&self) -> &str {
        &self.config.family_id
    }
    
    /// Get the build ID baked into this ISO
    pub fn get_build_id(&self) -> &str {
        &self.config.build_id
    }
    
    /// Get device info for display
    pub fn get_device_info(&self) -> Option<(&str, &str)> {
        match (&self.state.device_id, &self.state.child_name) {
            (Some(id), Some(name)) => Some((id.as_str(), name.as_str())),
            _ => None,
        }
    }
    
    /// Generate device hash from hardware identifiers
    pub fn generate_device_hash(&self) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        // Read machine ID
        let machine_id = fs::read_to_string("/etc/machine-id")
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();
        
        // Get CPU info
        let cpu_info = fs::read_to_string("/proc/cpuinfo")
            .unwrap_or_else(|_| "unknown".to_string());
        let cpu_id = cpu_info
            .lines()
            .find(|l| l.starts_with("model name"))
            .unwrap_or("unknown")
            .to_string();
        
        // Get root disk serial (simplified)
        let disk_id = fs::read_to_string("/sys/block/sda/device/serial")
            .or_else(|_| fs::read_to_string("/sys/block/nvme0n1/serial"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();
        
        // Combine and hash
        let combined = format!("{}:{}:{}", machine_id, cpu_id, disk_id);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        
        // Return first 16 chars of hex
        Ok(hex::encode(&result[..8]))
    }
    
    /// Attempt to activate this device with parent credentials
    pub async fn activate(&mut self, email: &str, password: &str, device_name: Option<&str>) -> Result<()> {
        if self.state.activated {
            info!("Device already activated");
            return Ok(());
        }
        
        let device_hash = self.generate_device_hash()?;
        info!("Attempting activation with device hash: {}", device_hash);
        
        let request = ActivationRequest {
            email: email.to_string(),
            password: password.to_string(),
            family_id: self.config.family_id.clone(),
            build_id: self.config.build_id.clone(),
            device_hash: device_hash.clone(),
            device_name: device_name.map(|s| s.to_string()),
        };
        
        let response = self.client
            .post(&format!("{}/functions/v1/device-activate", self.config.supabase_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Guardian servers")?;
        
        let status = response.status();
        let activation: ActivationResponse = response.json().await
            .context("Failed to parse activation response")?;
        
        if !status.is_success() {
            let error_msg = activation.error.unwrap_or_else(|| "Unknown error".to_string());
            
            // Check if this is a brick condition
            if activation.brick.unwrap_or(false) {
                error!("ACTIVATION FAILED - BRICK CONDITION: {}", error_msg);
                self.brick_device(&error_msg)?;
            }
            
            bail!("Activation failed: {}", error_msg);
        }
        
        // Verify the signature with our baked-in public key
        if let (Some(token_data), Some(signature)) = (&activation.token_data, &activation.signature) {
            if !self.verify_signature(token_data, signature)? {
                error!("SECURITY: Signature verification failed!");
                self.brick_device("Server signature invalid - possible tampering detected")?;
            }
        }
        
        // Success! Save activation state
        self.state = ActivationState {
            activated: true,
            device_id: activation.device_id,
            child_id: None, // Extracted from token
            child_name: activation.child_name,
            token_data: activation.token_data,
            signature: activation.signature,
            activated_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        
        self.save_state()?;
        
        info!("Device activated successfully!");
        Ok(())
    }
    
    /// Verify server signature with baked-in public key
    fn verify_signature(&self, token_data: &str, signature: &str) -> Result<bool> {
        use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_ASN1};
        
        // Parse PEM public key
        let pem = pem::parse(&self.config.verification_public_key)
            .context("Failed to parse public key PEM")?;
        
        // Create verification key
        let pem_contents = pem.contents();
        let public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &pem_contents);
        
        // Decode signature (JWS compact format)
        let parts: Vec<&str> = signature.split('.').collect();
        if parts.len() != 3 {
            warn!("Invalid JWS signature format");
            return Ok(false);
        }
        
        // For JWS, verify the signature part
        let sig_bytes = URL_SAFE_NO_PAD.decode(parts[2])
            .context("Failed to decode signature")?;
        
        let message = format!("{}.{}", parts[0], parts[1]);
        
        match public_key.verify(message.as_bytes(), &sig_bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Save activation state to disk
    fn save_state(&self) -> Result<()> {
        let state_dir = Path::new(ACTIVATION_STATE_PATH).parent().unwrap();
        fs::create_dir_all(state_dir)?;
        
        let state_json = serde_json::to_string_pretty(&self.state)?;
        fs::write(ACTIVATION_STATE_PATH, state_json)?;
        
        Ok(())
    }
    
    /// Brick the device (activation permanently failed)
    fn brick_device(&self, reason: &str) -> Result<()> {
        error!("🚫 DEVICE BRICKED: {}", reason);
        
        // Write brick state
        let brick_state = serde_json::json!({
            "bricked": true,
            "reason": reason,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "family_id": self.config.family_id,
            "build_id": self.config.build_id,
        });
        
        fs::write("/var/lib/guardian/brick_state.json", brick_state.to_string())?;
        
        // In production, this would:
        // 1. Disable the boot process
        // 2. Show error on screen
        // 3. Require complete reinstall
        
        bail!("Device has been bricked due to security violation: {}", reason);
    }
    
    /// Verify activation on each boot
    pub async fn verify_boot(&self) -> Result<()> {
        if !self.state.activated {
            warn!("Device not activated - activation required");
            return Ok(());
        }
        
        // Verify our token is still valid
        if let (Some(token_data), Some(signature)) = (&self.state.token_data, &self.state.signature) {
            if !self.verify_signature(token_data, signature)? {
                error!("Boot verification failed - signature invalid");
                self.brick_device("Boot signature verification failed")?;
            }
        }
        
        // Verify device hash hasn't changed (hardware swap detection)
        let current_hash = self.generate_device_hash()?;
        
        if let Some(ref token_data) = self.state.token_data {
            let token: serde_json::Value = serde_json::from_str(token_data)?;
            if let Some(stored_hash) = token.get("device_hash").and_then(|v| v.as_str()) {
                if stored_hash != current_hash {
                    error!("Hardware change detected! Stored: {}, Current: {}", stored_hash, current_hash);
                    // Don't brick immediately - might be legitimate hardware upgrade
                    // Instead, require re-verification
                    warn!("Device hardware changed - re-verification may be required");
                }
            }
        }
        
        info!("Boot verification passed");
        Ok(())
    }
}

impl Default for ActivationService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize activation service")
    }
}
