//! Guardian Selector - Boot-time child selection for Guardian OS
//!
//! This service runs before the display manager and:
//! 1. Shows list of children from this family
//! 2. Handles authentication (ask_parent, face_id, pin, auto)
//! 3. Configures autologin for selected child
//! 4. Writes selection to /run/guardian/current_child

mod config;
mod ui;
mod auth;
mod supabase;

use std::fs;
use std::io::{self, Write};
use std::process::Command;
use anyhow::{Result, Context};
use tracing::{info, warn, error};

use config::GuardianConfig;
use supabase::SupabaseClient;

/// Child profile from Supabase
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Child {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub experience_mode: String,
    pub trust_mode: String,
    pub unlock_method: String,
    pub avatar_url: Option<String>,
}

/// Activation state (from first boot)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ActivationState {
    pub device_id: String,
    pub family_id: String,
    pub children: Vec<ChildInfo>,
    pub activated_at: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChildInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Guardian Selector starting...");

    // Load config
    let config = GuardianConfig::load("/etc/guardian/config.yaml")
        .context("Failed to load Guardian config")?;

    // Check if already activated
    let activation_path = "/var/lib/guardian/activation_state.json";
    if !std::path::Path::new(activation_path).exists() {
        error!("Device not activated! Running activation flow...");
        return run_activation(&config).await;
    }

    // Load activation state
    let activation: ActivationState = serde_json::from_str(
        &fs::read_to_string(activation_path)?
    )?;

    // Initialize Supabase client
    let supabase = SupabaseClient::new(
        &config.api.supabase_url,
        &config.api.supabase_anon_key,
    );

    // Fetch current children from cloud (may have changed since activation)
    let children = match supabase.get_children(&config.family.id).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to fetch children from cloud: {}. Using cached.", e);
            // Fall back to cached children from activation
            activation.children.iter().map(|c| Child {
                id: c.id.clone(),
                name: c.name.clone(),
                slug: c.slug.clone(),
                experience_mode: "kiosk".to_string(),
                trust_mode: "supervised".to_string(),
                unlock_method: "ask_parent".to_string(),
                avatar_url: None,
            }).collect()
        }
    };

    if children.is_empty() {
        error!("No children found for this family!");
        return Err(anyhow::anyhow!("No children in family"));
    }

    // Show selector UI
    let selected_child = ui::show_selector(&children)?;

    info!("Selected child: {} ({})", selected_child.name, selected_child.slug);

    // Authenticate based on unlock_method
    let authenticated = match selected_child.unlock_method.as_str() {
        "ask_parent" => {
            auth::ask_parent(&supabase, &activation.device_id, &selected_child).await?
        }
        "face_id" => {
            auth::face_id(&selected_child).await.unwrap_or_else(|_| {
                // Fall back to PIN
                auth::pin(&selected_child).unwrap_or(false)
            })
        }
        "pin" => {
            auth::pin(&selected_child)?
        }
        "auto" => {
            // No authentication needed, just notify parent
            let _ = supabase.notify_session_start(&activation.device_id, &selected_child.id).await;
            true
        }
        _ => {
            warn!("Unknown unlock method: {}", selected_child.unlock_method);
            auth::ask_parent(&supabase, &activation.device_id, &selected_child).await?
        }
    };

    if !authenticated {
        error!("Authentication failed for {}", selected_child.name);
        println!("\n❌ Authentication failed. Please try again.\n");
        std::process::exit(1);
    }

    // Write current child to runtime file
    fs::create_dir_all("/run/guardian")?;
    fs::write("/run/guardian/current_child", &selected_child.slug)?;
    fs::write("/run/guardian/current_child_id", &selected_child.id)?;
    fs::write("/run/guardian/experience_mode", &selected_child.experience_mode)?;

    // Configure autologin for this child's Linux user
    configure_autologin(&selected_child.slug)?;

    // Create session record
    let _ = supabase.create_session(&activation.device_id, &selected_child.id, &selected_child.unlock_method).await;

    info!("Guardian Selector complete. Booting as {}.", selected_child.slug);
    println!("\n✅ Welcome, {}! Starting your session...\n", selected_child.name);

    Ok(())
}

/// Configure autologin for the selected child
fn configure_autologin(username: &str) -> Result<()> {
    // Create systemd autologin override
    let autologin_dir = "/etc/systemd/system/getty@tty1.service.d";
    fs::create_dir_all(autologin_dir)?;

    let config = format!(
        r#"[Service]
ExecStart=
ExecStart=-/sbin/agetty -o '-p -f -- \\u' --noclear --autologin {} %I $TERM
"#,
        username
    );

    fs::write(format!("{}/autologin.conf", autologin_dir), config)?;

    // Also configure GDM/SDDM if present
    if std::path::Path::new("/etc/gdm/custom.conf").exists() {
        configure_gdm_autologin(username)?;
    }

    if std::path::Path::new("/etc/sddm.conf.d").exists() {
        configure_sddm_autologin(username)?;
    }

    Ok(())
}

fn configure_gdm_autologin(username: &str) -> Result<()> {
    let config = format!(
        r#"[daemon]
AutomaticLoginEnable=true
AutomaticLogin={}
"#,
        username
    );
    fs::write("/etc/gdm/custom.conf", config)?;
    Ok(())
}

fn configure_sddm_autologin(username: &str) -> Result<()> {
    fs::create_dir_all("/etc/sddm.conf.d")?;
    let config = format!(
        r#"[Autologin]
User={}
Session=plasma
"#,
        username
    );
    fs::write("/etc/sddm.conf.d/guardian-autologin.conf", config)?;
    Ok(())
}

/// Run first-boot activation
async fn run_activation(config: &GuardianConfig) -> Result<()> {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║             🛡️  GUARDIAN OS ACTIVATION                    ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║                                                          ║");
    println!("║  This device needs to be activated by a parent.          ║");
    println!("║                                                          ║");
    println!("║  Family ID: {}...                        ║", &config.family.id[..8]);
    println!("║  Build ID:  {}...                        ║", &config.family.build_id[..8]);
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    print!("Parent Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    print!("Device Name: ");
    io::stdout().flush()?;
    let mut device_name = String::new();
    io::stdin().read_line(&mut device_name)?;
    let device_name = device_name.trim();
    let device_name = if device_name.is_empty() { 
        hostname::get()?.to_string_lossy().to_string() 
    } else { 
        device_name.to_string() 
    };

    println!("\nActivating device...");

    // Generate device hash
    let device_hash = generate_device_hash();

    // Call activation endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/functions/v1/device-activate", config.api.supabase_url))
        .header("apikey", &config.api.supabase_anon_key)
        .json(&serde_json::json!({
            "family_id": config.family.id,
            "build_id": config.family.build_id,
            "email": email,
            "password": password,
            "device_name": device_name,
            "device_hash": device_hash
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error: serde_json::Value = response.json().await?;
        let message = error["message"].as_str().unwrap_or("Activation failed");
        let brick = error["brick"].as_bool().unwrap_or(false);
        
        if brick {
            error!("SECURITY VIOLATION: {}", message);
            println!("\n❌ SECURITY ERROR: {}", message);
            println!("This device cannot be activated.");
            std::process::exit(1);
        }
        
        return Err(anyhow::anyhow!("Activation failed: {}", message));
    }

    let result: serde_json::Value = response.json().await?;

    // Save activation state
    let activation = ActivationState {
        device_id: result["device_id"].as_str().unwrap().to_string(),
        family_id: config.family.id.clone(),
        children: result["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| ChildInfo {
                id: c["id"].as_str().unwrap().to_string(),
                slug: c["slug"].as_str().unwrap().to_string(),
                name: c["name"].as_str().unwrap().to_string(),
            })
            .collect(),
        activated_at: chrono::Utc::now().to_rfc3339(),
    };

    fs::create_dir_all("/var/lib/guardian")?;
    fs::write(
        "/var/lib/guardian/activation_state.json",
        serde_json::to_string_pretty(&activation)?
    )?;

    // Save activation signature for boot verification
    fs::write(
        "/var/lib/guardian/activation_token.json",
        result["activation_token"].as_str().unwrap()
    )?;
    fs::write(
        "/var/lib/guardian/activation_signature.txt",
        result["activation_signature"].as_str().unwrap()
    )?;

    // Create Linux users for each child
    println!("\nCreating user accounts...");
    for child in &activation.children {
        create_linux_user(&child.slug, &child.name)?;
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ✅ DEVICE ACTIVATED SUCCESSFULLY!              ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║                                                          ║");
    for child in &activation.children {
        println!("║  ✓ Created account for: {:30} ║", child.name);
    }
    println!("║                                                          ║");
    println!("║  Reboot to start using Guardian OS.                      ║");
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Reboot after activation
    println!("Rebooting in 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    Command::new("reboot").spawn()?;

    Ok(())
}

fn generate_device_hash() -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    
    // Machine ID
    if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
        hasher.update(machine_id.trim().as_bytes());
    }
    
    // CPU info
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") || line.starts_with("Serial") {
                hasher.update(line.as_bytes());
            }
        }
    }
    
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

fn create_linux_user(username: &str, display_name: &str) -> Result<()> {
    // Check if user exists
    let exists = Command::new("id")
        .arg(username)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if exists {
        info!("User {} already exists", username);
        return Ok(());
    }

    // Create user with no password (login via guardian-selector)
    Command::new("useradd")
        .args([
            "-m",                           // Create home directory
            "-c", display_name,             // Full name
            "-s", "/bin/bash",              // Shell
            "-G", "wheel,video,audio",      // Groups
            username
        ])
        .status()
        .context("Failed to create user")?;

    // Lock password (no direct login)
    Command::new("passwd")
        .args(["-l", username])
        .status()?;

    info!("Created Linux user: {}", username);
    Ok(())
}
