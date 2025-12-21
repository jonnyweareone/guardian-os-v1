// Guardian OS Wallpaper Selector
// Personalized wallpapers based on child's profile (age, gender, interests)

use cosmic::{
    Element, Task,
    cosmic_config::{Config, ConfigSet},
    iced::{Alignment, Length},
    theme,
    widget,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{fl, page};

// Freepik API configuration
const FREEPIK_API_KEY: &str = "FPSX0f76042bd69e140f63c9c6232822da74";
const FREEPIK_SEARCH_URL: &str = "https://api.freepik.com/v1/resources";

/// Child profile for wallpaper personalization
#[derive(Clone, Debug, Default)]
pub struct ChildProfile {
    pub name: String,
    pub age: u8,
    pub gender: Gender,
    pub interests: Vec<Interest>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Gender {
    #[default]
    Neutral,
    Boy,
    Girl,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Interest {
    Gaming,
    Sports,
    Music,
    Art,
    Science,
    Nature,
    Animals,
    Space,
    Cars,
    Dinosaurs,
    Unicorns,
    Superheroes,
    Minecraft,
    Anime,
    Reading,
}

impl Interest {
    fn search_terms(&self) -> &'static str {
        match self {
            Interest::Gaming => "gaming setup neon controller",
            Interest::Sports => "sports stadium dynamic",
            Interest::Music => "music headphones neon",
            Interest::Art => "creative art colorful abstract",
            Interest::Science => "science technology futuristic",
            Interest::Nature => "nature landscape beautiful",
            Interest::Animals => "cute animals wildlife",
            Interest::Space => "space galaxy stars nebula",
            Interest::Cars => "cool cars racing",
            Interest::Dinosaurs => "dinosaurs prehistoric",
            Interest::Unicorns => "unicorn magical fantasy",
            Interest::Superheroes => "superhero action dynamic",
            Interest::Minecraft => "pixel art blocks gaming",
            Interest::Anime => "anime aesthetic japanese",
            Interest::Reading => "books library cozy",
        }
    }
    
    fn display_name(&self) -> &'static str {
        match self {
            Interest::Gaming => "Gaming",
            Interest::Sports => "Sports",
            Interest::Music => "Music",
            Interest::Art => "Art & Drawing",
            Interest::Science => "Science",
            Interest::Nature => "Nature",
            Interest::Animals => "Animals",
            Interest::Space => "Space",
            Interest::Cars => "Cars",
            Interest::Dinosaurs => "Dinosaurs",
            Interest::Unicorns => "Unicorns",
            Interest::Superheroes => "Superheroes",
            Interest::Minecraft => "Minecraft",
            Interest::Anime => "Anime",
            Interest::Reading => "Reading",
        }
    }
    
    fn icon(&self) -> &'static str {
        match self {
            Interest::Gaming => "🎮",
            Interest::Sports => "⚽",
            Interest::Music => "🎵",
            Interest::Art => "🎨",
            Interest::Science => "🔬",
            Interest::Nature => "🌲",
            Interest::Animals => "🐾",
            Interest::Space => "🚀",
            Interest::Cars => "🏎️",
            Interest::Dinosaurs => "🦖",
            Interest::Unicorns => "🦄",
            Interest::Superheroes => "🦸",
            Interest::Minecraft => "⛏️",
            Interest::Anime => "🎌",
            Interest::Reading => "📚",
        }
    }
}

/// A wallpaper option fetched from Freepik
#[derive(Clone, Debug)]
pub struct WallpaperOption {
    pub id: String,
    pub preview_url: String,
    pub download_url: String,
    pub title: String,
    pub handle: Option<widget::image::Handle>,
}

#[derive(Clone, Debug)]
pub enum Message {
    // Profile setup
    SetGender(Gender),
    SetAge(u8),
    ToggleInterest(Interest),
    
    // Wallpaper selection
    SearchWallpapers,
    WallpapersLoaded(Vec<WallpaperOption>),
    WallpaperPreviewLoaded(usize, widget::image::Handle),
    SelectWallpaper(usize),
    ApplyWallpaper,
    
    // Navigation
    NextStep,
    PrevStep,
    
    // Error handling
    SearchError(String),
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::Wallpaper(message)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    ProfileSetup,
    InterestSelection,
    WallpaperSelection,
}

pub struct Page {
    step: Step,
    profile: ChildProfile,
    wallpapers: Vec<WallpaperOption>,
    selected_wallpaper: Option<usize>,
    loading: bool,
    error: Option<String>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            step: Step::ProfileSetup,
            profile: ChildProfile::default(),
            wallpapers: Vec::new(),
            selected_wallpaper: None,
            loading: false,
            error: None,
        }
    }
    
    /// Build search query based on profile
    fn build_search_query(&self) -> String {
        let mut terms = Vec::new();
        
        // Add age-appropriate modifier
        match self.profile.age {
            0..=6 => terms.push("cute cartoon colorful"),
            7..=12 => terms.push("cool vibrant"),
            13..=17 => terms.push("aesthetic modern"),
            _ => terms.push("stylish"),
        }
        
        // Add gender preference (subtle)
        match self.profile.gender {
            Gender::Boy => terms.push("blue dynamic"),
            Gender::Girl => terms.push("pink purple"),
            Gender::Neutral => terms.push("colorful"),
        }
        
        // Add interest terms
        for interest in &self.profile.interests {
            terms.push(interest.search_terms());
        }
        
        // Always add wallpaper/desktop terms
        terms.push("desktop wallpaper background");
        
        terms.join(" ")
    }
    
    /// Fetch wallpapers from Freepik API
    fn fetch_wallpapers(&self) -> Task<page::Message> {
        let query = self.build_search_query();
        
        Task::perform(
            async move {
                fetch_freepik_wallpapers(&query).await
            },
            |result| {
                match result {
                    Ok(wallpapers) => Message::WallpapersLoaded(wallpapers).into(),
                    Err(e) => Message::SearchError(e).into(),
                }
            }
        )
    }
    
    pub fn update(&mut self, message: Message) -> Task<page::Message> {
        match message {
            Message::SetGender(gender) => {
                self.profile.gender = gender;
                Task::none()
            }
            
            Message::SetAge(age) => {
                self.profile.age = age;
                Task::none()
            }
            
            Message::ToggleInterest(interest) => {
                if self.profile.interests.contains(&interest) {
                    self.profile.interests.retain(|i| i != &interest);
                } else {
                    self.profile.interests.push(interest);
                }
                Task::none()
            }
            
            Message::NextStep => {
                match self.step {
                    Step::ProfileSetup => {
                        self.step = Step::InterestSelection;
                    }
                    Step::InterestSelection => {
                        self.step = Step::WallpaperSelection;
                        self.loading = true;
                        return self.fetch_wallpapers();
                    }
                    Step::WallpaperSelection => {
                        // Apply wallpaper and continue
                        return Task::done(Message::ApplyWallpaper.into());
                    }
                }
                Task::none()
            }
            
            Message::PrevStep => {
                match self.step {
                    Step::ProfileSetup => {}
                    Step::InterestSelection => {
                        self.step = Step::ProfileSetup;
                    }
                    Step::WallpaperSelection => {
                        self.step = Step::InterestSelection;
                    }
                }
                Task::none()
            }
            
            Message::SearchWallpapers => {
                self.loading = true;
                self.fetch_wallpapers()
            }
            
            Message::WallpapersLoaded(wallpapers) => {
                self.wallpapers = wallpapers;
                self.loading = false;
                
                // Start loading preview images
                let tasks: Vec<_> = self.wallpapers.iter().enumerate().map(|(idx, wp)| {
                    let url = wp.preview_url.clone();
                    Task::perform(
                        async move {
                            load_image_from_url(&url).await
                        },
                        move |result| {
                            if let Ok(handle) = result {
                                Message::WallpaperPreviewLoaded(idx, handle).into()
                            } else {
                                Message::SearchError("Failed to load preview".into()).into()
                            }
                        }
                    )
                }).collect();
                
                Task::batch(tasks)
            }
            
            Message::WallpaperPreviewLoaded(idx, handle) => {
                if let Some(wp) = self.wallpapers.get_mut(idx) {
                    wp.handle = Some(handle);
                }
                Task::none()
            }
            
            Message::SelectWallpaper(idx) => {
                self.selected_wallpaper = Some(idx);
                Task::none()
            }
            
            Message::ApplyWallpaper => {
                if let Some(idx) = self.selected_wallpaper {
                    if let Some(wallpaper) = self.wallpapers.get(idx) {
                        // Download and apply wallpaper
                        let download_url = wallpaper.download_url.clone();
                        return Task::perform(
                            async move {
                                download_and_apply_wallpaper(&download_url).await
                            },
                            |_| page::Message::Wallpaper(Message::NextStep)
                        );
                    }
                }
                Task::none()
            }
            
            Message::SearchError(err) => {
                self.error = Some(err);
                self.loading = false;
                Task::none()
            }
        }
    }
    
    fn view_profile_setup(&self) -> Element<'_, page::Message> {
        let spacing = theme::active().cosmic().spacing;
        
        let title = widget::text::title3("Tell us about yourself!")
            .width(Length::Fill)
            .align_x(Alignment::Center);
        
        let subtitle = widget::text::body("We'll find the perfect wallpapers for you")
            .width(Length::Fill)
            .align_x(Alignment::Center);
        
        // Gender selection
        let gender_label = widget::text::body("I am a...");
        
        let gender_buttons = widget::row::with_capacity(3)
            .push(
                widget::button::text("👦 Boy")
                    .on_press(Message::SetGender(Gender::Boy).into())
                    .class(if self.profile.gender == Gender::Boy {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Standard
                    })
            )
            .push(
                widget::button::text("👧 Girl")
                    .on_press(Message::SetGender(Gender::Girl).into())
                    .class(if self.profile.gender == Gender::Girl {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Standard
                    })
            )
            .push(
                widget::button::text("😊 Other")
                    .on_press(Message::SetGender(Gender::Neutral).into())
                    .class(if self.profile.gender == Gender::Neutral {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Standard
                    })
            )
            .spacing(spacing.space_s);
        
        // Age selection
        let age_label = widget::text::body("My age:");
        
        let age_buttons = widget::row::with_children(
            (5..=17).map(|age| {
                widget::button::text(age.to_string())
                    .on_press(Message::SetAge(age).into())
                    .class(if self.profile.age == age {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Standard
                    })
                    .into()
            }).collect()
        )
        .spacing(spacing.space_xxs)
        .wrap();
        
        // Next button
        let next_btn = widget::button::suggested("Next →")
            .on_press(Message::NextStep.into());
        
        widget::column::with_capacity(8)
            .push(title)
            .push(subtitle)
            .push(widget::vertical_space().height(spacing.space_l))
            .push(gender_label)
            .push(gender_buttons)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(age_label)
            .push(age_buttons)
            .push(widget::vertical_space().height(spacing.space_xl))
            .push(next_btn)
            .spacing(spacing.space_s)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
    }
    
    fn view_interest_selection(&self) -> Element<'_, page::Message> {
        let spacing = theme::active().cosmic().spacing;
        
        let title = widget::text::title3("What do you like?")
            .width(Length::Fill)
            .align_x(Alignment::Center);
        
        let subtitle = widget::text::body("Pick your favourite things (choose at least 1)")
            .width(Length::Fill)
            .align_x(Alignment::Center);
        
        // Interest grid
        let all_interests = vec![
            Interest::Gaming,
            Interest::Space,
            Interest::Animals,
            Interest::Sports,
            Interest::Music,
            Interest::Art,
            Interest::Science,
            Interest::Nature,
            Interest::Cars,
            Interest::Dinosaurs,
            Interest::Unicorns,
            Interest::Superheroes,
            Interest::Minecraft,
            Interest::Anime,
            Interest::Reading,
        ];
        
        let mut grid = widget::grid()
            .column_spacing(spacing.space_s)
            .row_spacing(spacing.space_s);
        
        for (i, interest) in all_interests.iter().enumerate() {
            if i > 0 && i % 5 == 0 {
                grid = grid.insert_row();
            }
            
            let is_selected = self.profile.interests.contains(interest);
            let label = format!("{} {}", interest.icon(), interest.display_name());
            
            let btn = widget::button::text(label)
                .on_press(Message::ToggleInterest(interest.clone()).into())
                .class(if is_selected {
                    theme::Button::Suggested
                } else {
                    theme::Button::Standard
                });
            
            grid = grid.push(btn);
        }
        
        // Navigation buttons
        let nav_buttons = widget::row::with_capacity(2)
            .push(
                widget::button::standard("← Back")
                    .on_press(Message::PrevStep.into())
            )
            .push(widget::horizontal_space())
            .push(
                widget::button::suggested("Find Wallpapers →")
                    .on_press_maybe(
                        if !self.profile.interests.is_empty() {
                            Some(Message::NextStep.into())
                        } else {
                            None
                        }
                    )
            )
            .width(Length::Fill);
        
        widget::column::with_capacity(5)
            .push(title)
            .push(subtitle)
            .push(widget::vertical_space().height(spacing.space_l))
            .push(grid)
            .push(widget::vertical_space().height(spacing.space_xl))
            .push(nav_buttons)
            .spacing(spacing.space_s)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
    }
    
    fn view_wallpaper_selection(&self) -> Element<'_, page::Message> {
        let spacing = theme::active().cosmic().spacing;
        
        let title = widget::text::title3("Choose your wallpaper!")
            .width(Length::Fill)
            .align_x(Alignment::Center);
        
        if self.loading {
            let loading = widget::column::with_capacity(2)
                .push(widget::text::body("Finding cool wallpapers for you..."))
                .push(widget::text::body("🔍"))
                .align_x(Alignment::Center);
            
            return widget::column::with_capacity(2)
                .push(title)
                .push(loading)
                .spacing(spacing.space_xl)
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .into();
        }
        
        if let Some(err) = &self.error {
            let error_view = widget::column::with_capacity(3)
                .push(widget::text::body(format!("Oops! {}", err)))
                .push(
                    widget::button::standard("Try Again")
                        .on_press(Message::SearchWallpapers.into())
                )
                .align_x(Alignment::Center);
            
            return widget::column::with_capacity(2)
                .push(title)
                .push(error_view)
                .spacing(spacing.space_xl)
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .into();
        }
        
        // Wallpaper grid
        let mut grid = widget::grid()
            .column_spacing(spacing.space_m)
            .row_spacing(spacing.space_m);
        
        for (i, wallpaper) in self.wallpapers.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                grid = grid.insert_row();
            }
            
            let is_selected = self.selected_wallpaper == Some(i);
            
            let thumbnail: Element<'_, page::Message> = if let Some(handle) = &wallpaper.handle {
                widget::image(handle.clone())
                    .width(200)
                    .height(112)
                    .into()
            } else {
                widget::container(
                    widget::text::body("Loading...")
                )
                .width(200)
                .height(112)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
            };
            
            let button = widget::button::custom(thumbnail)
                .class(if is_selected {
                    theme::Button::Suggested
                } else {
                    theme::Button::Standard
                })
                .on_press(Message::SelectWallpaper(i).into());
            
            grid = grid.push(button);
        }
        
        // Navigation buttons
        let nav_buttons = widget::row::with_capacity(3)
            .push(
                widget::button::standard("← Back")
                    .on_press(Message::PrevStep.into())
            )
            .push(widget::horizontal_space())
            .push(
                widget::button::suggested("Apply Wallpaper")
                    .on_press_maybe(
                        self.selected_wallpaper.map(|_| Message::ApplyWallpaper.into())
                    )
            )
            .width(Length::Fill);
        
        widget::column::with_capacity(4)
            .push(title)
            .push(widget::scrollable(grid))
            .push(widget::vertical_space().height(spacing.space_m))
            .push(nav_buttons)
            .spacing(spacing.space_s)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
    }
}

impl page::Page for Page {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> String {
        match self.step {
            Step::ProfileSetup => "Personalise Your Desktop".to_string(),
            Step::InterestSelection => "Your Interests".to_string(),
            Step::WallpaperSelection => "Choose Wallpaper".to_string(),
        }
    }

    fn skippable(&self) -> bool {
        true
    }

    fn view(&self) -> Element<'_, page::Message> {
        match self.step {
            Step::ProfileSetup => self.view_profile_setup(),
            Step::InterestSelection => self.view_interest_selection(),
            Step::WallpaperSelection => self.view_wallpaper_selection(),
        }
    }
}

// ============================================================================
// Freepik API Integration
// ============================================================================

#[derive(Debug, Deserialize)]
struct FreepikResponse {
    data: Vec<FreepikResource>,
}

#[derive(Debug, Deserialize)]
struct FreepikResource {
    id: u64,
    title: String,
    #[serde(rename = "preview")]
    preview: FreepikPreview,
}

#[derive(Debug, Deserialize)]
struct FreepikPreview {
    url: String,
}

async fn fetch_freepik_wallpapers(query: &str) -> Result<Vec<WallpaperOption>, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(FREEPIK_SEARCH_URL)
        .header("x-freepik-api-key", FREEPIK_API_KEY)
        .query(&[
            ("term", query),
            ("limit", "12"),
            ("order", "relevance"),
            ("filters[orientation][landscape]", "1"),
            ("filters[content_type][photo]", "1"),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    let data: FreepikResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let wallpapers = data.data.into_iter().map(|resource| {
        WallpaperOption {
            id: resource.id.to_string(),
            preview_url: resource.preview.url.clone(),
            download_url: resource.preview.url, // Would need actual download URL
            title: resource.title,
            handle: None,
        }
    }).collect();
    
    Ok(wallpapers)
}

async fn load_image_from_url(url: &str) -> Result<widget::image::Handle, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch image: {}", e))?;
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image: {}", e))?;
    
    Ok(widget::image::Handle::from_bytes(bytes.to_vec()))
}

async fn download_and_apply_wallpaper(url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    
    // Download image
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {}", e))?;
    
    // Save to backgrounds directory
    let wallpaper_dir = PathBuf::from("/usr/share/backgrounds/guardian/custom");
    std::fs::create_dir_all(&wallpaper_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    let wallpaper_path = wallpaper_dir.join("user-wallpaper.png");
    std::fs::write(&wallpaper_path, &bytes)
        .map_err(|e| format!("Failed to save: {}", e))?;
    
    // Update COSMIC background config
    let config_path = dirs::config_dir()
        .unwrap_or_default()
        .join("cosmic/com.system76.CosmicBackground/v1/all");
    
    let config_content = format!(r#"(
    backgrounds: [
        (
            source: Path("{}"),
            filter_by_theme: false,
            rotation_frequency: 0,
        ),
    ],
    same_on_all: true,
)"#, wallpaper_path.display());
    
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&config_path, config_content).ok();
    
    Ok(())
}
