//! VPN and proxy detection for Guardian DNS

use std::collections::HashSet;

/// VPN and proxy detector
pub struct VpnDetector {
    /// Known VPN provider domains
    vpn_domains: HashSet<String>,
    
    /// DNS-over-HTTPS provider domains (to prevent DNS bypass)
    doh_domains: HashSet<String>,
    
    /// Known proxy service domains
    proxy_domains: HashSet<String>,
    
    /// Tor-related domains
    tor_domains: HashSet<String>,
    
    /// Whether detection is enabled
    enabled: bool,
    
    /// Whether to block VPN domains
    block_vpn: bool,
    
    /// Whether to block DoH domains
    block_doh: bool,
}

impl VpnDetector {
    pub fn new(enabled: bool, block_vpn: bool, block_doh: bool) -> Self {
        let mut vpn_domains = HashSet::new();
        let mut doh_domains = HashSet::new();
        let mut proxy_domains = HashSet::new();
        let mut tor_domains = HashSet::new();
        
        // Major VPN providers
        let vpn_list = [
            // Commercial VPNs
            "nordvpn.com",
            "expressvpn.com",
            "surfshark.com",
            "privateinternetaccess.com",
            "cyberghostvpn.com",
            "protonvpn.com",
            "ipvanish.com",
            "purevpn.com",
            "vyprvpn.com",
            "hotspotshield.com",
            "tunnelbear.com",
            "windscribe.com",
            "mullvad.net",
            "airvpn.org",
            "privatevpn.com",
            "hide.me",
            "zenmate.com",
            "strongvpn.com",
            "torguard.net",
            "astrill.com",
            "buffered.com",
            "safervpn.com",
            "ivpn.net",
            "perfectprivacy.com",
            "ovpn.com",
            "goosevpn.com",
            "atlasvpn.com",
            // VPN software
            "openvpn.net",
            "wireguard.com",
        ];
        
        for domain in vpn_list {
            vpn_domains.insert(domain.to_string());
        }
        
        // DNS-over-HTTPS providers (blocking these prevents DNS bypass)
        let doh_list = [
            "dns.google",
            "dns.google.com",
            "cloudflare-dns.com",
            "1dot1dot1dot1.cloudflare-dns.com",
            "dns.quad9.net",
            "doh.opendns.com",
            "dns.nextdns.io",
            "doh.cleanbrowsing.org",
            "dns.adguard.com",
            "doh.dns.sb",
            "dns.switch.ch",
            "doh.li",
            "dns.twnic.tw",
            "doh.tiar.app",
            "jp.tiar.app",
            "doh.tiarap.org",
            "odvr.nic.cz",
            "dns.aa.net.uk",
            "dns.digitale-gesellschaft.ch",
            "dns.oszx.co",
            "dns.pumplex.com",
        ];
        
        for domain in doh_list {
            doh_domains.insert(domain.to_string());
        }
        
        // Proxy services
        let proxy_list = [
            "hide.me",
            "proxysite.com",
            "hidester.com",
            "kproxy.com",
            "proxify.com",
            "anonymouse.org",
            "filterbypass.me",
            "zend2.com",
            "webproxy.to",
            "proxy-list.org",
            "proxybay.github.io",
            "unblocker.us",
            "unblockvideos.com",
            "croxyproxy.com",
            "proxysite.io",
            "blockaway.net",
            "plain-proxies.com",
            "sitenable.com",
            "weboas.is",
            "freevpn.us",
        ];
        
        for domain in proxy_list {
            proxy_domains.insert(domain.to_string());
        }
        
        // Tor-related domains
        let tor_list = [
            "torproject.org",
            "tor.eff.org",
            "tails.boum.org",
            "onion.foundation",
        ];
        
        for domain in tor_list {
            tor_domains.insert(domain.to_string());
        }
        
        Self {
            vpn_domains,
            doh_domains,
            proxy_domains,
            tor_domains,
            enabled,
            block_vpn,
            block_doh,
        }
    }
    
    /// Check if a domain is a VPN service
    pub fn is_vpn_domain(&self, domain: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let domain = domain.to_lowercase();
        
        // Check exact match and parent domains
        if self.vpn_domains.contains(&domain) {
            return true;
        }
        
        // Check if it's a subdomain of a VPN domain
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if self.vpn_domains.contains(&parent) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a domain is a DoH provider
    pub fn is_doh_domain(&self, domain: &str) -> bool {
        if !self.enabled || !self.block_doh {
            return false;
        }
        
        let domain = domain.to_lowercase();
        
        if self.doh_domains.contains(&domain) {
            return true;
        }
        
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if self.doh_domains.contains(&parent) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a domain is a proxy service
    pub fn is_proxy_domain(&self, domain: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let domain = domain.to_lowercase();
        
        if self.proxy_domains.contains(&domain) {
            return true;
        }
        
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if self.proxy_domains.contains(&parent) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a domain is Tor-related
    pub fn is_tor_domain(&self, domain: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let domain = domain.to_lowercase();
        
        if self.tor_domains.contains(&domain) {
            return true;
        }
        
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if self.tor_domains.contains(&parent) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if domain should be blocked (VPN, DoH, or proxy)
    pub fn should_block(&self, domain: &str) -> Option<VpnBlockReason> {
        if !self.enabled {
            return None;
        }
        
        if self.block_vpn && self.is_vpn_domain(domain) {
            return Some(VpnBlockReason::VpnService);
        }
        
        if self.block_doh && self.is_doh_domain(domain) {
            return Some(VpnBlockReason::DnsOverHttps);
        }
        
        if self.is_proxy_domain(domain) {
            return Some(VpnBlockReason::ProxyService);
        }
        
        if self.is_tor_domain(domain) {
            return Some(VpnBlockReason::TorService);
        }
        
        None
    }
    
    /// Check if detection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnBlockReason {
    VpnService,
    DnsOverHttps,
    ProxyService,
    TorService,
}

impl VpnBlockReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            VpnBlockReason::VpnService => "VPN service",
            VpnBlockReason::DnsOverHttps => "DNS-over-HTTPS provider",
            VpnBlockReason::ProxyService => "Proxy service",
            VpnBlockReason::TorService => "Tor network",
        }
    }
}

impl Default for VpnDetector {
    fn default() -> Self {
        Self::new(true, true, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vpn_detection() {
        let detector = VpnDetector::default();
        
        assert!(detector.is_vpn_domain("nordvpn.com"));
        assert!(detector.is_vpn_domain("www.nordvpn.com"));
        assert!(detector.is_vpn_domain("app.nordvpn.com"));
        assert!(!detector.is_vpn_domain("google.com"));
    }
    
    #[test]
    fn test_doh_detection() {
        let detector = VpnDetector::default();
        
        assert!(detector.is_doh_domain("dns.google"));
        assert!(detector.is_doh_domain("cloudflare-dns.com"));
        assert!(!detector.is_doh_domain("google.com"));
    }
    
    #[test]
    fn test_proxy_detection() {
        let detector = VpnDetector::default();
        
        assert!(detector.is_proxy_domain("proxysite.com"));
        assert!(!detector.is_proxy_domain("youtube.com"));
    }
    
    #[test]
    fn test_disabled() {
        let detector = VpnDetector::new(false, true, true);
        
        assert!(!detector.is_vpn_domain("nordvpn.com"));
        assert!(detector.should_block("nordvpn.com").is_none());
    }
}
