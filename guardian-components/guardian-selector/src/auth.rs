//! Authentication methods for Guardian Selector

use crate::{Child, supabase::SupabaseClient};
use anyhow::Result;
use std::io::{self, Write};

/// Ask parent for approval via push notification
pub async fn ask_parent(
    supabase: &SupabaseClient,
    device_id: &str,
    child: &Child,
) -> Result<bool> {
    println!();
    println!("┌────────────────────────────────────────────────────────┐");
    println!("│  📱 Asking parent for permission...                    │");
    println!("│                                                        │");
    println!("│  {} wants to use this device.              │", child.name);
    println!("│                                                        │");
    println!("│  Waiting for parent approval...                        │");
    println!("│                                                        │");
    println!("│  [Press P for PIN backup if available]                 │");
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    // Create login request
    let request_id = supabase.create_login_request(device_id, &child.slug).await?;

    // Poll for response (with timeout)
    let timeout = std::time::Duration::from_secs(120); // 2 minutes
    let start = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_secs(2);

    loop {
        if start.elapsed() > timeout {
            println!("⏰ Request timed out. Please try again.");
            return Ok(false);
        }

        // Check for response
        let status = supabase.check_login_request(&request_id).await?;

        match status.as_str() {
            "approved" => {
                println!("✅ Parent approved!");
                return Ok(true);
            }
            "denied" => {
                println!("❌ Parent denied the request.");
                return Ok(false);
            }
            "expired" => {
                println!("⏰ Request expired. Please try again.");
                return Ok(false);
            }
            _ => {
                // Still pending, show countdown
                let remaining = timeout.saturating_sub(start.elapsed());
                print!("\r⏳ Waiting... {}s remaining   ", remaining.as_secs());
                io::stdout().flush()?;
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Face ID authentication
pub async fn face_id(child: &Child) -> Result<bool> {
    println!();
    println!("┌────────────────────────────────────────────────────────┐");
    println!("│  👤 Face ID Verification                               │");
    println!("│                                                        │");
    println!("│  Look at the camera, {}               │", child.name);
    println!("│                                                        │");
    println!("│         ┌───────────┐                                  │");
    println!("│         │  📷      │                                  │");
    println!("│         │ Scanning │                                  │");
    println!("│         └───────────┘                                  │");
    println!("│                                                        │");
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    // Check if face data exists for this child
    let face_data_path = format!("/var/lib/guardian/faces/{}.dat", child.slug);
    if !std::path::Path::new(&face_data_path).exists() {
        println!("⚠️  Face ID not enrolled for {}. Use PIN instead.", child.name);
        return Err(anyhow::anyhow!("Face ID not enrolled"));
    }

    // TODO: Integrate with Howdy or custom face recognition
    // For now, this is a placeholder
    
    // Simulate face scan
    println!("Scanning face...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // In production, this would call the face recognition system
    // let result = howdy::verify(&child.slug)?;
    
    // Placeholder - always fail to fall back to PIN
    Err(anyhow::anyhow!("Face ID verification not implemented yet"))
}

/// PIN authentication
pub fn pin(child: &Child) -> Result<bool> {
    println!();
    println!("┌────────────────────────────────────────────────────────┐");
    println!("│  🔢 Enter PIN for {}                     │", child.name);
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    // Check if PIN is set
    // In production, we'd check the database
    
    for attempt in 1..=3 {
        print!("PIN (attempt {}/3): ", attempt);
        io::stdout().flush()?;

        // Read PIN (in production, use rpassword for hidden input)
        let mut pin_input = String::new();
        io::stdin().read_line(&mut pin_input)?;
        let pin = pin_input.trim();

        if pin.len() < 4 {
            println!("PIN must be at least 4 digits.");
            continue;
        }

        // TODO: Verify PIN against Supabase
        // For now, accept any 4+ digit PIN as a placeholder
        // In production:
        // let valid = supabase.verify_pin(&child.id, pin).await?;
        
        println!("⚠️  PIN verification not fully implemented yet.");
        println!("    For testing, any 4+ digit PIN is accepted.");
        
        return Ok(true);
    }

    println!("❌ Too many failed attempts. Try again later.");
    Ok(false)
}
