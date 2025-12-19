//! App catalog management

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ratings::AgeRating;

/// App entry in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub rating: AgeRating,
    pub guardian_approved: bool,
    pub flatpak_ref: Option<String>,
    pub homepage: Option<String>,
    pub version: Option<String>,
}

/// Category in the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: String,
}

/// Load app catalog
/// In production, this would:
/// 1. Load appstream data from Flathub
/// 2. Merge with Guardian catalog (approved apps, custom ratings)
/// 3. Cache locally
pub async fn load_catalog() -> Result<Vec<AppEntry>> {
    // For now, return sample data
    // TODO: Integrate with appstream and Supabase app_catalog table
    
    Ok(vec![
        AppEntry {
            id: "org.gnome.Calculator".to_string(),
            name: "Calculator".to_string(),
            summary: Some("Perform arithmetic, scientific or financial calculations".to_string()),
            description: Some("A simple calculator for the GNOME desktop".to_string()),
            developer: Some("GNOME".to_string()),
            icon_url: None,
            categories: vec!["Utility".to_string(), "Education".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.gnome.Calculator".to_string()),
            homepage: Some("https://wiki.gnome.org/Apps/Calculator".to_string()),
            version: Some("46.0".to_string()),
        },
        AppEntry {
            id: "org.kde.kturtle".to_string(),
            name: "KTurtle".to_string(),
            summary: Some("Educational programming environment".to_string()),
            description: Some("KTurtle is an educational programming environment for learning how to program.".to_string()),
            developer: Some("KDE".to_string()),
            icon_url: None,
            categories: vec!["Education".to_string(), "Development".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.kde.kturtle".to_string()),
            homepage: Some("https://edu.kde.org/kturtle/".to_string()),
            version: Some("24.02".to_string()),
        },
        AppEntry {
            id: "org.tuxpaint.Tuxpaint".to_string(),
            name: "Tux Paint".to_string(),
            summary: Some("Drawing program for children".to_string()),
            description: Some("Tux Paint is a free, award-winning drawing program for children ages 3 to 12.".to_string()),
            developer: Some("Tux Paint".to_string()),
            icon_url: None,
            categories: vec!["Graphics".to_string(), "Education".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.tuxpaint.Tuxpaint".to_string()),
            homepage: Some("http://www.tuxpaint.org/".to_string()),
            version: Some("0.9.32".to_string()),
        },
        AppEntry {
            id: "org.kde.gcompris".to_string(),
            name: "GCompris".to_string(),
            summary: Some("Educational software for children aged 2 to 10".to_string()),
            description: Some("GCompris is a high quality educational software suite, including a large number of activities for children aged 2 to 10.".to_string()),
            developer: Some("KDE".to_string()),
            icon_url: None,
            categories: vec!["Education".to_string(), "Game".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.kde.gcompris".to_string()),
            homepage: Some("https://gcompris.net/".to_string()),
            version: Some("4.0".to_string()),
        },
        AppEntry {
            id: "io.github.nickvergessen.Mindustry".to_string(),
            name: "Mindustry".to_string(),
            summary: Some("A factory-building game with tower defense".to_string()),
            description: Some("Build factories and tower defenses in this sandbox strategy game.".to_string()),
            developer: Some("Anuken".to_string()),
            icon_url: None,
            categories: vec!["Game".to_string(), "Strategy".to_string()],
            rating: AgeRating::Everyone10,
            guardian_approved: true,
            flatpak_ref: Some("io.github.nickvergessen.Mindustry".to_string()),
            homepage: Some("https://mindustrygame.github.io/".to_string()),
            version: Some("146".to_string()),
        },
        AppEntry {
            id: "org.supertuxkart.SuperTuxKart".to_string(),
            name: "SuperTuxKart".to_string(),
            summary: Some("A 3D arcade racer".to_string()),
            description: Some("SuperTuxKart is a 3D open-source arcade racer with a variety of characters, tracks, and modes.".to_string()),
            developer: Some("SuperTuxKart Team".to_string()),
            icon_url: None,
            categories: vec!["Game".to_string(), "Racing".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.supertuxkart.SuperTuxKart".to_string()),
            homepage: Some("https://supertuxkart.net/".to_string()),
            version: Some("1.4".to_string()),
        },
        AppEntry {
            id: "org.libreoffice.LibreOffice".to_string(),
            name: "LibreOffice".to_string(),
            summary: Some("The free office suite".to_string()),
            description: Some("LibreOffice is a powerful office suite for word processing, spreadsheets, presentations, and more.".to_string()),
            developer: Some("The Document Foundation".to_string()),
            icon_url: None,
            categories: vec!["Office".to_string(), "Productivity".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.libreoffice.LibreOffice".to_string()),
            homepage: Some("https://www.libreoffice.org/".to_string()),
            version: Some("24.2".to_string()),
        },
        AppEntry {
            id: "org.gimp.GIMP".to_string(),
            name: "GIMP".to_string(),
            summary: Some("Image manipulation program".to_string()),
            description: Some("GIMP is a cross-platform image editor for graphic design, photo editing, and more.".to_string()),
            developer: Some("GIMP Team".to_string()),
            icon_url: None,
            categories: vec!["Graphics".to_string(), "Creativity".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.gimp.GIMP".to_string()),
            homepage: Some("https://www.gimp.org/".to_string()),
            version: Some("2.10.36".to_string()),
        },
        AppEntry {
            id: "org.blender.Blender".to_string(),
            name: "Blender".to_string(),
            summary: Some("3D creation suite".to_string()),
            description: Some("Blender is a free and open source 3D creation suite for modeling, animation, simulation, and more.".to_string()),
            developer: Some("Blender Foundation".to_string()),
            icon_url: None,
            categories: vec!["Graphics".to_string(), "Creativity".to_string(), "Education".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("org.blender.Blender".to_string()),
            homepage: Some("https://www.blender.org/".to_string()),
            version: Some("4.1".to_string()),
        },
        AppEntry {
            id: "net.minetest.Minetest".to_string(),
            name: "Minetest".to_string(),
            summary: Some("Open source voxel game engine".to_string()),
            description: Some("An open source voxel game engine with easy modding and game creation.".to_string()),
            developer: Some("Minetest Team".to_string()),
            icon_url: None,
            categories: vec!["Game".to_string(), "Simulation".to_string()],
            rating: AgeRating::Everyone,
            guardian_approved: true,
            flatpak_ref: Some("net.minetest.Minetest".to_string()),
            homepage: Some("https://www.minetest.net/".to_string()),
            version: Some("5.8.0".to_string()),
        },
        // Teen-rated games
        AppEntry {
            id: "io.github.AmatCoder.mednaffe".to_string(),
            name: "OpenMW".to_string(),
            summary: Some("Open-source game engine for The Elder Scrolls III: Morrowind".to_string()),
            description: Some("A free, open source, and modern game engine for playing Morrowind.".to_string()),
            developer: Some("OpenMW Team".to_string()),
            icon_url: None,
            categories: vec!["Game".to_string(), "RPG".to_string()],
            rating: AgeRating::Teen,
            guardian_approved: false,
            flatpak_ref: Some("org.openmw.OpenMW".to_string()),
            homepage: Some("https://openmw.org/".to_string()),
            version: Some("0.48.0".to_string()),
        },
    ])
}

/// Load categories
pub async fn load_categories() -> Result<Vec<Category>> {
    Ok(vec![
        Category {
            id: "games".to_string(),
            name: "Games".to_string(),
            icon: "applications-games-symbolic".to_string(),
        },
        Category {
            id: "education".to_string(),
            name: "Education".to_string(),
            icon: "accessories-dictionary-symbolic".to_string(),
        },
        Category {
            id: "graphics".to_string(),
            name: "Graphics & Art".to_string(),
            icon: "applications-graphics-symbolic".to_string(),
        },
        Category {
            id: "productivity".to_string(),
            name: "Productivity".to_string(),
            icon: "applications-office-symbolic".to_string(),
        },
        Category {
            id: "development".to_string(),
            name: "Development".to_string(),
            icon: "applications-development-symbolic".to_string(),
        },
        Category {
            id: "multimedia".to_string(),
            name: "Multimedia".to_string(),
            icon: "applications-multimedia-symbolic".to_string(),
        },
    ])
}
