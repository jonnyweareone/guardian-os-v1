//! Safe search enforcement for Guardian DNS

use std::collections::HashMap;

/// Safe search rewriter - forces safe search on popular search engines
pub struct SafeSearchRewriter {
    /// Domain rewrites (original -> safe)
    rewrites: HashMap<String, String>,
    
    /// Whether safe search is enabled
    enabled: bool,
}

impl SafeSearchRewriter {
    pub fn new(
        enabled: bool,
        google: bool,
        bing: bool,
        youtube: &str,
        duckduckgo: bool,
    ) -> Self {
        let mut rewrites = HashMap::new();
        
        if google {
            // Google Safe Search - rewrite to forcesafesearch.google.com
            let google_domains = [
                "www.google.com",
                "google.com",
                "www.google.co.uk",
                "google.co.uk",
                "www.google.ca",
                "google.ca",
                "www.google.com.au",
                "google.com.au",
                "www.google.de",
                "google.de",
                "www.google.fr",
                "google.fr",
                "www.google.es",
                "google.es",
                "www.google.it",
                "google.it",
                "www.google.nl",
                "google.nl",
                "www.google.ie",
                "google.ie",
            ];
            
            for domain in google_domains {
                rewrites.insert(domain.to_string(), "forcesafesearch.google.com".to_string());
            }
        }
        
        if bing {
            // Bing Strict Mode
            rewrites.insert("www.bing.com".to_string(), "strict.bing.com".to_string());
            rewrites.insert("bing.com".to_string(), "strict.bing.com".to_string());
        }
        
        // YouTube restriction modes
        let youtube_target = match youtube {
            "strict" => Some("restrict.youtube.com"),
            "moderate" => Some("restrictmoderate.youtube.com"),
            _ => None,
        };
        
        if let Some(target) = youtube_target {
            let youtube_domains = [
                "www.youtube.com",
                "youtube.com",
                "m.youtube.com",
                "www.youtube-nocookie.com",
                "youtube-nocookie.com",
            ];
            
            for domain in youtube_domains {
                rewrites.insert(domain.to_string(), target.to_string());
            }
        }
        
        if duckduckgo {
            // DuckDuckGo Safe Search
            rewrites.insert("duckduckgo.com".to_string(), "safe.duckduckgo.com".to_string());
            rewrites.insert("www.duckduckgo.com".to_string(), "safe.duckduckgo.com".to_string());
        }
        
        Self { rewrites, enabled }
    }
    
    /// Check if a domain should be rewritten for safe search
    pub fn rewrite(&self, domain: &str) -> Option<&str> {
        if !self.enabled {
            return None;
        }
        
        self.rewrites.get(&domain.to_lowercase()).map(|s| s.as_str())
    }
    
    /// Check if safe search is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get the number of rewrite rules
    pub fn rule_count(&self) -> usize {
        self.rewrites.len()
    }
}

impl Default for SafeSearchRewriter {
    fn default() -> Self {
        Self::new(true, true, true, "moderate", true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_google_rewrite() {
        let rewriter = SafeSearchRewriter::default();
        
        assert_eq!(
            rewriter.rewrite("www.google.com"),
            Some("forcesafesearch.google.com")
        );
        assert_eq!(
            rewriter.rewrite("google.co.uk"),
            Some("forcesafesearch.google.com")
        );
    }
    
    #[test]
    fn test_youtube_moderate() {
        let rewriter = SafeSearchRewriter::new(true, false, false, "moderate", false);
        
        assert_eq!(
            rewriter.rewrite("www.youtube.com"),
            Some("restrictmoderate.youtube.com")
        );
    }
    
    #[test]
    fn test_youtube_strict() {
        let rewriter = SafeSearchRewriter::new(true, false, false, "strict", false);
        
        assert_eq!(
            rewriter.rewrite("youtube.com"),
            Some("restrict.youtube.com")
        );
    }
    
    #[test]
    fn test_disabled() {
        let rewriter = SafeSearchRewriter::new(false, true, true, "moderate", true);
        
        assert_eq!(rewriter.rewrite("www.google.com"), None);
    }
}
