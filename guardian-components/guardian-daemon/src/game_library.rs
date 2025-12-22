//! Game Library Scanner
//!
//! Scans installed game libraries (Steam, Heroic, Lutris, Flatpak) and
//! reports them to the Guardian daemon for policy enforcement.
//!
//! The launcher uses this to know what games exist, then filters based
//! on the child's policy.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// A game discovered on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredGame {
    /// Unique identifier (platform-specific)
    pub id: String,
    /// Display name
    pub name: String,
    /// Source platform
    pub platform: GamePlatform,
    /// Is the game installed?
    pub installed: bool,
    /// Install path (if installed)
    pub install_path: Option<PathBuf>,
    /// Executable path (if installed)
    pub executable: Option<PathBuf>,
    /// Cover art URL or local path
    pub cover_url: Option<String>,
    /// Steam AppID (for Steam games)
    pub steam_app_id: Option<u64>,
    /// Last played timestamp
    pub last_played: Option<i64>,
    /// Total playtime in minutes
    pub playtime_minutes: Option<u32>,
    /// Size in bytes
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePlatform {
    Steam,
    Epic,
    GOG,
    Lutris,
    Flatpak,
    Local,
}

impl GamePlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            GamePlatform::Steam => "steam",
            GamePlatform::Epic => "epic",
            GamePlatform::GOG => "gog",
            GamePlatform::Lutris => "lutris",
            GamePlatform::Flatpak => "flatpak",
            GamePlatform::Local => "local",
        }
    }
}

/// Game library scanner
pub struct GameLibraryScanner {
    /// Home directory
    home_dir: PathBuf,
    /// Discovered games
    games: HashMap<String, DiscoveredGame>,
}

impl GameLibraryScanner {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Could not determine home directory")?;
        
        Ok(Self {
            home_dir,
            games: HashMap::new(),
        })
    }
    
    /// Scan all game sources
    pub fn scan_all(&mut self) -> Result<Vec<DiscoveredGame>> {
        info!("Scanning game libraries...");
        
        self.games.clear();
        
        // Scan each platform
        if let Err(e) = self.scan_steam() {
            warn!("Steam scan failed: {}", e);
        }
        
        if let Err(e) = self.scan_heroic() {
            warn!("Heroic scan failed: {}", e);
        }
        
        if let Err(e) = self.scan_lutris() {
            warn!("Lutris scan failed: {}", e);
        }
        
        if let Err(e) = self.scan_flatpak_games() {
            warn!("Flatpak games scan failed: {}", e);
        }
        
        info!("Found {} games total", self.games.len());
        
        Ok(self.games.values().cloned().collect())
    }
    
    /// Scan Steam libraries
    fn scan_steam(&mut self) -> Result<()> {
        // Check both native and Flatpak Steam paths
        let steam_paths = [
            self.home_dir.join(".local/share/Steam"),
            self.home_dir.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ];
        
        for steam_path in steam_paths {
            if !steam_path.exists() {
                continue;
            }
            
            debug!("Scanning Steam at {:?}", steam_path);
            
            // Read libraryfolders.vdf to find all library paths
            let libraryfolders = steam_path.join("steamapps/libraryfolders.vdf");
            if libraryfolders.exists() {
                self.scan_steam_library(&steam_path.join("steamapps"))?;
                
                // Parse libraryfolders.vdf for additional paths
                if let Ok(content) = std::fs::read_to_string(&libraryfolders) {
                    for line in content.lines() {
                        if line.contains("\"path\"") {
                            // Extract path from VDF line like: "path"    "/mnt/games/steam"
                            if let Some(path) = line.split('"').nth(3) {
                                let lib_path = PathBuf::from(path).join("steamapps");
                                if lib_path.exists() && lib_path != steam_path.join("steamapps") {
                                    self.scan_steam_library(&lib_path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Scan a single Steam library folder
    fn scan_steam_library(&mut self, steamapps: &PathBuf) -> Result<()> {
        let common = steamapps.join("common");
        
        // Read .acf manifest files
        for entry in std::fs::read_dir(steamapps)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "acf").unwrap_or(false) {
                if let Ok(game) = self.parse_steam_acf(&path, &common) {
                    let key = format!("steam_{}", game.steam_app_id.unwrap_or(0));
                    self.games.insert(key, game);
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse a Steam .acf manifest file
    fn parse_steam_acf(&self, acf_path: &PathBuf, common_path: &PathBuf) -> Result<DiscoveredGame> {
        let content = std::fs::read_to_string(acf_path)?;
        
        let mut app_id: Option<u64> = None;
        let mut name: Option<String> = None;
        let mut install_dir: Option<String> = None;
        let mut size: Option<u64> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("\"appid\"") {
                app_id = line.split('"').nth(3).and_then(|s| s.parse().ok());
            } else if line.starts_with("\"name\"") {
                name = line.split('"').nth(3).map(String::from);
            } else if line.starts_with("\"installdir\"") {
                install_dir = line.split('"').nth(3).map(String::from);
            } else if line.starts_with("\"SizeOnDisk\"") {
                size = line.split('"').nth(3).and_then(|s| s.parse().ok());
            }
        }
        
        let app_id = app_id.context("No appid in ACF")?;
        let name = name.context("No name in ACF")?;
        
        let install_path = install_dir.map(|d| common_path.join(d));
        
        Ok(DiscoveredGame {
            id: format!("steam_{}", app_id),
            name,
            platform: GamePlatform::Steam,
            installed: install_path.as_ref().map(|p| p.exists()).unwrap_or(false),
            install_path,
            executable: None, // Would need to parse launch options
            cover_url: Some(format!(
                "https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900.jpg",
                app_id
            )),
            steam_app_id: Some(app_id),
            last_played: None,
            playtime_minutes: None,
            size_bytes: size,
        })
    }
    
    /// Scan Heroic Games Launcher (Epic + GOG)
    fn scan_heroic(&mut self) -> Result<()> {
        let heroic_path = self.home_dir.join(".config/heroic");
        
        if !heroic_path.exists() {
            return Ok(());
        }
        
        debug!("Scanning Heroic at {:?}", heroic_path);
        
        // Scan Epic games (legendaryConfig/installed.json)
        let legendary_installed = heroic_path.join("legendaryConfig/installed.json");
        if legendary_installed.exists() {
            if let Ok(content) = std::fs::read_to_string(&legendary_installed) {
                if let Ok(installed) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                    for (app_name, info) in installed {
                        let game = DiscoveredGame {
                            id: format!("epic_{}", app_name),
                            name: info.get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or(&app_name)
                                .to_string(),
                            platform: GamePlatform::Epic,
                            installed: true,
                            install_path: info.get("install_path")
                                .and_then(|p| p.as_str())
                                .map(PathBuf::from),
                            executable: info.get("executable")
                                .and_then(|p| p.as_str())
                                .map(PathBuf::from),
                            cover_url: None, // Would need to fetch from Epic API
                            steam_app_id: None,
                            last_played: None,
                            playtime_minutes: None,
                            size_bytes: info.get("install_size")
                                .and_then(|s| s.as_u64()),
                        };
                        self.games.insert(game.id.clone(), game);
                    }
                }
            }
        }
        
        // Scan GOG games (gog_store/installed.json)
        let gog_installed = heroic_path.join("gog_store/installed.json");
        if gog_installed.exists() {
            if let Ok(content) = std::fs::read_to_string(&gog_installed) {
                if let Ok(installed) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                    for (app_name, info) in installed {
                        let game = DiscoveredGame {
                            id: format!("gog_{}", app_name),
                            name: info.get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or(&app_name)
                                .to_string(),
                            platform: GamePlatform::GOG,
                            installed: true,
                            install_path: info.get("install_path")
                                .and_then(|p| p.as_str())
                                .map(PathBuf::from),
                            executable: None,
                            cover_url: None,
                            steam_app_id: None,
                            last_played: None,
                            playtime_minutes: None,
                            size_bytes: None,
                        };
                        self.games.insert(game.id.clone(), game);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Scan Lutris
    fn scan_lutris(&mut self) -> Result<()> {
        let lutris_path = self.home_dir.join(".config/lutris");
        let games_dir = lutris_path.join("games");
        
        if !games_dir.exists() {
            return Ok(());
        }
        
        debug!("Scanning Lutris at {:?}", games_dir);
        
        for entry in std::fs::read_dir(&games_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "yml").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Simple YAML parsing for name
                    let mut name = None;
                    let mut slug = None;
                    
                    for line in content.lines() {
                        if line.starts_with("name:") {
                            name = Some(line.trim_start_matches("name:").trim().to_string());
                        } else if line.starts_with("slug:") {
                            slug = Some(line.trim_start_matches("slug:").trim().to_string());
                        }
                    }
                    
                    if let (Some(name), Some(slug)) = (name, slug) {
                        let game = DiscoveredGame {
                            id: format!("lutris_{}", slug),
                            name,
                            platform: GamePlatform::Lutris,
                            installed: true,
                            install_path: None,
                            executable: None,
                            cover_url: None,
                            steam_app_id: None,
                            last_played: None,
                            playtime_minutes: None,
                            size_bytes: None,
                        };
                        self.games.insert(game.id.clone(), game);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Scan Flatpak apps that are games
    fn scan_flatpak_games(&mut self) -> Result<()> {
        // Run flatpak list to get installed apps
        let output = std::process::Command::new("flatpak")
            .args(["list", "--app", "--columns=application,name,branch"])
            .output()?;
        
        if !output.status.success() {
            return Ok(());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Known game-related Flatpak patterns
        let game_patterns = [
            "com.valvesoftware.Steam",
            "com.heroicgameslauncher.hgl",
            "net.lutris.Lutris",
            "org.prismlauncher.PrismLauncher", // Minecraft
            "com.mojang.Minecraft",
            "io.itch.itch",
            "com.discordapp.Discord", // Not a game but gaming-related
        ];
        
        // Gaming categories to look for
        let game_categories = ["Game", "ActionGame", "AdventureGame", "ArcadeGame"];
        
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }
            
            let app_id = parts[0];
            let name = parts[1];
            
            // Check if it's a known gaming app
            let is_game = game_patterns.iter().any(|p| app_id.contains(p));
            
            if is_game {
                let game = DiscoveredGame {
                    id: format!("flatpak_{}", app_id),
                    name: name.to_string(),
                    platform: GamePlatform::Flatpak,
                    installed: true,
                    install_path: None,
                    executable: None,
                    cover_url: None,
                    steam_app_id: None,
                    last_played: None,
                    playtime_minutes: None,
                    size_bytes: None,
                };
                self.games.insert(game.id.clone(), game);
            }
        }
        
        Ok(())
    }
    
    /// Get all discovered games
    pub fn get_games(&self) -> Vec<&DiscoveredGame> {
        self.games.values().collect()
    }
    
    /// Get a specific game by ID
    pub fn get_game(&self, id: &str) -> Option<&DiscoveredGame> {
        self.games.get(id)
    }
    
    /// Filter games by what's allowed for a child
    pub fn filter_allowed(
        &self,
        allowed_ids: &[String],
        blocked_ids: &[String],
    ) -> Vec<&DiscoveredGame> {
        self.games.values()
            .filter(|game| {
                // If specifically blocked, exclude
                if blocked_ids.iter().any(|b| game.id.contains(b) || game.name.to_lowercase().contains(&b.to_lowercase())) {
                    return false;
                }
                
                // If allowed list is empty, allow all (that aren't blocked)
                if allowed_ids.is_empty() {
                    return true;
                }
                
                // Otherwise must be in allowed list
                allowed_ids.iter().any(|a| game.id.contains(a) || game.name.to_lowercase().contains(&a.to_lowercase()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_as_str() {
        assert_eq!(GamePlatform::Steam.as_str(), "steam");
        assert_eq!(GamePlatform::Epic.as_str(), "epic");
    }
    
    #[test]
    fn test_filter_allowed() {
        let mut scanner = GameLibraryScanner {
            home_dir: PathBuf::from("/home/test"),
            games: HashMap::new(),
        };
        
        scanner.games.insert("steam_123".into(), DiscoveredGame {
            id: "steam_123".into(),
            name: "Minecraft".into(),
            platform: GamePlatform::Steam,
            installed: true,
            install_path: None,
            executable: None,
            cover_url: None,
            steam_app_id: Some(123),
            last_played: None,
            playtime_minutes: None,
            size_bytes: None,
        });
        
        scanner.games.insert("steam_456".into(), DiscoveredGame {
            id: "steam_456".into(),
            name: "GTA V".into(),
            platform: GamePlatform::Steam,
            installed: true,
            install_path: None,
            executable: None,
            cover_url: None,
            steam_app_id: Some(456),
            last_played: None,
            playtime_minutes: None,
            size_bytes: None,
        });
        
        // Block GTA
        let allowed = scanner.filter_allowed(&[], &["GTA".into()]);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0].name, "Minecraft");
        
        // Allow only Minecraft
        let allowed = scanner.filter_allowed(&["Minecraft".into()], &[]);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0].name, "Minecraft");
    }
}
