//! Blocklist management for Guardian DNS

use aho_corasick::AhoCorasick;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

/// Blocklist matcher using Aho-Corasick for fast multi-pattern matching
pub struct BlocklistMatcher {
    /// Exact domain matches
    exact_matches: HashSet<String>,
    
    /// Domain to category mapping
    categories: HashMap<String, String>,
    
    /// Wildcard matcher for subdomain blocking
    wildcard_matcher: Option<AhoCorasick>,
    wildcard_domains: Vec<String>,
    
    /// Whitelist (never block these)
    whitelist: HashSet<String>,
}

impl BlocklistMatcher {
    pub fn new() -> Self {
        Self {
            exact_matches: HashSet::new(),
            categories: HashMap::new(),
            wildcard_matcher: None,
            wildcard_domains: Vec::new(),
            whitelist: HashSet::new(),
        }
    }
    
    /// Load blocklists from URLs
    pub async fn load_blocklists(&mut self, urls: &[String]) -> Result<()> {
        let client = reqwest::Client::new();
        
        for url in urls {
            info!("Loading blocklist from: {}", url);
            
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response.text().await?;
                        let count = self.parse_hosts_file(&content);
                        info!("Loaded {} domains from {}", count, url);
                    } else {
                        warn!("Failed to fetch blocklist {}: {}", url, response.status());
                    }
                }
                Err(e) => {
                    warn!("Error fetching blocklist {}: {}", url, e);
                }
            }
        }
        
        // Build wildcard matcher
        self.build_wildcard_matcher();
        
        Ok(())
    }
    
    /// Parse hosts file format
    fn parse_hosts_file(&mut self, content: &str) -> usize {
        let mut count = 0;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse hosts format: "0.0.0.0 domain.com" or "127.0.0.1 domain.com"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let domain = parts[1].to_lowercase();
                
                // Skip localhost entries
                if domain == "localhost" || domain == "localhost.localdomain" {
                    continue;
                }
                
                // Skip IP addresses
                if domain.parse::<std::net::IpAddr>().is_ok() {
                    continue;
                }
                
                self.exact_matches.insert(domain.clone());
                self.categories.insert(domain, "blocklist".to_string());
                count += 1;
            }
        }
        
        count
    }
    
    /// Add custom blocked domains
    pub fn add_custom_blocks(&mut self, domains: &[String], category: &str) {
        for domain in domains {
            let domain = domain.to_lowercase();
            self.exact_matches.insert(domain.clone());
            self.categories.insert(domain, category.to_string());
        }
    }
    
    /// Add wildcard blocks (*.example.com)
    pub fn add_wildcard_blocks(&mut self, domains: &[String], category: &str) {
        for domain in domains {
            let domain = domain.to_lowercase();
            self.wildcard_domains.push(domain.clone());
            self.categories.insert(domain, category.to_string());
        }
        self.build_wildcard_matcher();
    }
    
    /// Set whitelist
    pub fn set_whitelist(&mut self, domains: &[String]) {
        self.whitelist.clear();
        for domain in domains {
            self.whitelist.insert(domain.to_lowercase());
        }
    }
    
    /// Build the wildcard matcher
    fn build_wildcard_matcher(&mut self) {
        if !self.wildcard_domains.is_empty() {
            self.wildcard_matcher = Some(
                AhoCorasick::new(&self.wildcard_domains)
                    .expect("Failed to build wildcard matcher")
            );
        }
    }
    
    /// Check if a domain is blocked
    pub fn check(&self, domain: &str) -> Option<&str> {
        let domain = domain.to_lowercase();
        
        // Check whitelist first
        if self.whitelist.contains(&domain) {
            return None;
        }
        
        // Check parent domains for whitelist too
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 0..parts.len() {
            let parent = parts[i..].join(".");
            if self.whitelist.contains(&parent) {
                return None;
            }
        }
        
        // Check exact match
        if let Some(category) = self.categories.get(&domain) {
            return Some(category);
        }
        
        // Check parent domains (block subdomains of blocked domains)
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if let Some(category) = self.categories.get(&parent) {
                return Some(category);
            }
        }
        
        // Check wildcard matches
        if let Some(ref matcher) = self.wildcard_matcher {
            if matcher.is_match(&domain) {
                // Find the matching pattern's category
                for wd in &self.wildcard_domains {
                    if domain.ends_with(wd) || domain == *wd {
                        if let Some(category) = self.categories.get(wd) {
                            return Some(category);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Get total number of blocked domains
    pub fn count(&self) -> usize {
        self.exact_matches.len() + self.wildcard_domains.len()
    }
}

impl Default for BlocklistMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exact_match() {
        let mut matcher = BlocklistMatcher::new();
        matcher.add_custom_blocks(&["blocked.com".to_string()], "test");
        
        assert!(matcher.check("blocked.com").is_some());
        assert!(matcher.check("sub.blocked.com").is_some());  // Subdomains blocked
        assert!(matcher.check("notblocked.com").is_none());
    }
    
    #[test]
    fn test_whitelist() {
        let mut matcher = BlocklistMatcher::new();
        matcher.add_custom_blocks(&["blocked.com".to_string()], "test");
        matcher.set_whitelist(&["blocked.com".to_string()]);
        
        assert!(matcher.check("blocked.com").is_none());  // Whitelisted
    }
    
    #[test]
    fn test_subdomain_blocking() {
        let mut matcher = BlocklistMatcher::new();
        matcher.add_custom_blocks(&["example.com".to_string()], "test");
        
        assert!(matcher.check("example.com").is_some());
        assert!(matcher.check("www.example.com").is_some());
        assert!(matcher.check("sub.sub.example.com").is_some());
        assert!(matcher.check("notexample.com").is_none());
    }
}
