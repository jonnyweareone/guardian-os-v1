//! Age rating system for Guardian Store

use serde::{Deserialize, Serialize};

/// Age rating for apps (unified PEGI/ESRB style)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgeRating {
    /// Suitable for all ages (PEGI 3, ESRB Everyone)
    Everyone,
    /// Ages 7+ (PEGI 7, ESRB Everyone 10+)
    Everyone10,
    /// Ages 12+ (PEGI 12, ESRB Teen)
    Teen,
    /// Ages 16+ (PEGI 16)
    Mature16,
    /// Ages 18+ (PEGI 18, ESRB Mature)
    Mature,
    /// Not rated
    Unrated,
}

impl Default for AgeRating {
    fn default() -> Self {
        Self::Everyone
    }
}

impl AgeRating {
    /// Numeric value for comparison (lower = more restrictive)
    pub fn numeric_value(&self) -> u8 {
        match self {
            Self::Everyone => 0,
            Self::Everyone10 => 10,
            Self::Teen => 13,
            Self::Mature16 => 16,
            Self::Mature => 18,
            Self::Unrated => 99,
        }
    }
    
    /// Minimum age for this rating
    pub fn min_age(&self) -> u8 {
        match self {
            Self::Everyone => 0,
            Self::Everyone10 => 10,
            Self::Teen => 13,
            Self::Mature16 => 16,
            Self::Mature => 18,
            Self::Unrated => 18, // Default to adult
        }
    }
    
    /// Get rating appropriate for a child's age
    pub fn for_age(age: u32) -> Self {
        if age >= 18 {
            Self::Mature
        } else if age >= 16 {
            Self::Mature16
        } else if age >= 13 {
            Self::Teen
        } else if age >= 10 {
            Self::Everyone10
        } else {
            Self::Everyone
        }
    }
    
    /// Short label for display
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Everyone => "E",
            Self::Everyone10 => "E10+",
            Self::Teen => "T",
            Self::Mature16 => "16+",
            Self::Mature => "M",
            Self::Unrated => "NR",
        }
    }
    
    /// Full label for display
    pub fn full_label(&self) -> &'static str {
        match self {
            Self::Everyone => "Everyone",
            Self::Everyone10 => "Everyone 10+",
            Self::Teen => "Teen",
            Self::Mature16 => "Mature 16+",
            Self::Mature => "Mature 18+",
            Self::Unrated => "Not Rated",
        }
    }
    
    /// Badge info (label, RGB color)
    pub fn badge_info(&self) -> (&'static str, (f32, f32, f32)) {
        match self {
            Self::Everyone => ("E", (0.2, 0.7, 0.2)),      // Green
            Self::Everyone10 => ("E10+", (0.4, 0.7, 0.2)), // Yellow-green
            Self::Teen => ("T", (0.8, 0.6, 0.0)),          // Orange
            Self::Mature16 => ("16+", (0.8, 0.4, 0.0)),    // Dark orange
            Self::Mature => ("M", (0.8, 0.2, 0.2)),        // Red
            Self::Unrated => ("NR", (0.5, 0.5, 0.5)),      // Gray
        }
    }
    
    /// Parse from appstream content_rating age hint
    pub fn from_appstream_age(age: u8) -> Self {
        match age {
            0..=6 => Self::Everyone,
            7..=9 => Self::Everyone10,
            10..=15 => Self::Teen,
            16..=17 => Self::Mature16,
            _ => Self::Mature,
        }
    }
    
    /// Parse from PEGI rating string
    pub fn from_pegi(pegi: &str) -> Self {
        match pegi.trim().to_lowercase().as_str() {
            "3" | "pegi 3" => Self::Everyone,
            "7" | "pegi 7" => Self::Everyone10,
            "12" | "pegi 12" => Self::Teen,
            "16" | "pegi 16" => Self::Mature16,
            "18" | "pegi 18" => Self::Mature,
            _ => Self::Unrated,
        }
    }
    
    /// Parse from ESRB rating string
    pub fn from_esrb(esrb: &str) -> Self {
        let lower = esrb.trim().to_lowercase();
        if lower.contains("everyone 10") || lower.contains("e10+") {
            Self::Everyone10
        } else if lower.contains("everyone") || lower == "e" {
            Self::Everyone
        } else if lower.contains("teen") || lower == "t" {
            Self::Teen
        } else if lower.contains("mature") || lower == "m" {
            Self::Mature
        } else if lower.contains("adult") || lower == "ao" {
            Self::Mature
        } else {
            Self::Unrated
        }
    }
}

/// Content descriptors that may affect rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDescriptors {
    pub violence: ContentLevel,
    pub language: ContentLevel,
    pub drugs: ContentLevel,
    pub nudity: ContentLevel,
    pub sex: ContentLevel,
    pub gambling: ContentLevel,
    pub fear: ContentLevel,
    pub social: ContentLevel,
    pub in_app_purchases: bool,
    pub ads: bool,
    pub online_interaction: bool,
}

impl Default for ContentDescriptors {
    fn default() -> Self {
        Self {
            violence: ContentLevel::None,
            language: ContentLevel::None,
            drugs: ContentLevel::None,
            nudity: ContentLevel::None,
            sex: ContentLevel::None,
            gambling: ContentLevel::None,
            fear: ContentLevel::None,
            social: ContentLevel::None,
            in_app_purchases: false,
            ads: false,
            online_interaction: false,
        }
    }
}

impl ContentDescriptors {
    /// Calculate appropriate age rating from content
    pub fn calculate_rating(&self) -> AgeRating {
        let mut max_age = 0u8;
        
        // Violence
        max_age = max_age.max(match self.violence {
            ContentLevel::None => 0,
            ContentLevel::Mild => 7,
            ContentLevel::Moderate => 12,
            ContentLevel::Intense => 18,
        });
        
        // Language
        max_age = max_age.max(match self.language {
            ContentLevel::None => 0,
            ContentLevel::Mild => 7,
            ContentLevel::Moderate => 12,
            ContentLevel::Intense => 16,
        });
        
        // Nudity
        max_age = max_age.max(match self.nudity {
            ContentLevel::None => 0,
            ContentLevel::Mild => 12,
            ContentLevel::Moderate => 16,
            ContentLevel::Intense => 18,
        });
        
        // Sex
        max_age = max_age.max(match self.sex {
            ContentLevel::None => 0,
            ContentLevel::Mild => 12,
            ContentLevel::Moderate => 16,
            ContentLevel::Intense => 18,
        });
        
        // Drugs
        max_age = max_age.max(match self.drugs {
            ContentLevel::None => 0,
            ContentLevel::Mild => 12,
            ContentLevel::Moderate => 16,
            ContentLevel::Intense => 18,
        });
        
        // Gambling
        max_age = max_age.max(match self.gambling {
            ContentLevel::None => 0,
            ContentLevel::Mild => 12,
            ContentLevel::Moderate => 16,
            ContentLevel::Intense => 18,
        });
        
        // Fear
        max_age = max_age.max(match self.fear {
            ContentLevel::None => 0,
            ContentLevel::Mild => 7,
            ContentLevel::Moderate => 12,
            ContentLevel::Intense => 16,
        });
        
        AgeRating::from_appstream_age(max_age)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentLevel {
    None,
    Mild,
    Moderate,
    Intense,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_age_rating_order() {
        assert!(AgeRating::Everyone < AgeRating::Everyone10);
        assert!(AgeRating::Teen < AgeRating::Mature);
    }
    
    #[test]
    fn test_for_age() {
        assert_eq!(AgeRating::for_age(5), AgeRating::Everyone);
        assert_eq!(AgeRating::for_age(11), AgeRating::Everyone10);
        assert_eq!(AgeRating::for_age(14), AgeRating::Teen);
        assert_eq!(AgeRating::for_age(17), AgeRating::Mature16);
        assert_eq!(AgeRating::for_age(21), AgeRating::Mature);
    }
}
