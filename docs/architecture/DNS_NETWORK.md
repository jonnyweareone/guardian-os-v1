# Guardian DNS & Network Shield

## Overview

guardian-dns is a local DNS server that provides:
- Domain-level filtering (block inappropriate sites)
- Safe search enforcement (Google, Bing, YouTube, DuckDuckGo)
- VPN/proxy detection and blocking
- Browsing logs (domains only)
- Category-based blocking

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        guardian-dns                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌────────────────┐  │
│  │  DNS Listener   │    │  Filter Engine  │    │  Upstream DNS  │  │
│  │  (port 53)      │───▶│                 │───▶│  (Cloudflare)  │  │
│  └─────────────────┘    └─────────────────┘    └────────────────┘  │
│                               │                                     │
│                               ▼                                     │
│                    ┌─────────────────────┐                         │
│                    │   Rule Processors   │                         │
│                    ├─────────────────────┤                         │
│                    │ • Blocklist matcher │                         │
│                    │ • Safe search force │                         │
│                    │ • VPN detection     │                         │
│                    │ • Category lookup   │                         │
│                    │ • Custom rules      │                         │
│                    └─────────────────────┘                         │
│                               │                                     │
│                               ▼                                     │
│                    ┌─────────────────────┐                         │
│                    │    Query Logger     │                         │
│                    │  (domain + time)    │                         │
│                    └─────────────────────┘                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Features

### 1. Domain Blocking

Uses multiple blocklists:
- **Adult content** - Pornography, adult material
- **Malware** - Known malicious domains
- **Gambling** - Betting, casino sites
- **Social (optional)** - Social media (per-child setting)
- **Gaming (optional)** - Game sites (per-child setting)

Sources:
- Steven Black's hosts (https://github.com/StevenBlack/hosts)
- OISD (https://oisd.nl/)
- Custom Guardian blocklist

### 2. Safe Search Enforcement

Forces safe search on major search engines by DNS rewrite:

| Service | Method |
|---------|--------|
| Google | Rewrite to `forcesafesearch.google.com` |
| Bing | Rewrite to `strict.bing.com` |
| YouTube | Rewrite to `restrictmoderate.youtube.com` |
| DuckDuckGo | Rewrite to `safe.duckduckgo.com` |

### 3. VPN/Proxy Detection

Blocks known VPN/proxy services to prevent bypass:

**DNS-level blocks:**
- Commercial VPNs (NordVPN, ExpressVPN, etc.)
- Proxy services (hide.me, proxysite.com, etc.)
- Tor entry nodes
- DNS-over-HTTPS endpoints (prevent DNS bypass)

**Application detection:**
- Monitor for VPN app installations
- Detect VPN network interfaces
- Alert parent if bypass attempted

### 4. Browsing Logs

Logs domain queries (NOT full URLs):

```rust
struct DnsQueryLog {
    timestamp: DateTime,
    child_id: ChildId,
    domain: String,           // "youtube.com"
    category: Option<String>, // "video"
    action: QueryAction,      // Allowed, Blocked, SafeSearch
}

// We DON'T log:
// - Full URLs (path, query params)
// - POST data
// - Page content
```

### 5. Category-Based Controls

| Category | Description | Default |
|----------|-------------|---------|
| adult | Pornography, adult content | 🚫 Block all |
| malware | Malicious domains | 🚫 Block all |
| gambling | Betting, casinos | 🚫 Block all |
| social | Social media | ⚙️ Per-child |
| gaming | Game sites | ⚙️ Per-child |
| video | YouTube, streaming | ⚙️ Per-child |
| shopping | E-commerce | ✅ Allow |
| news | News sites | ✅ Allow |
| education | Educational | ✅ Allow |

## Configuration

```toml
# /etc/guardian/dns.toml

[server]
listen_addr = "127.0.0.1:53"
upstream_dns = ["1.1.1.3", "1.0.0.3"]  # Cloudflare Family
cache_size = 10000
cache_ttl = 300

[safesearch]
enabled = true
google = true
bing = true
youtube = "moderate"  # "strict" or "moderate"
duckduckgo = true

[blocking]
# Block categories for ALL users
always_block = ["adult", "malware", "gambling"]

# Blocklist sources (auto-updated daily)
blocklists = [
    "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts",
    "https://big.oisd.nl/domainswild",
]

# Custom blocks
custom_block = [
    "example-bad-site.com",
]

# Never block (even if in blocklist)
whitelist = [
    "educational-site.com",
]

[vpn_detection]
enabled = true
block_vpn_domains = true
block_doh = true  # Block DNS-over-HTTPS providers
alert_on_attempt = true

[logging]
enabled = true
retention_days = 30
log_path = "/var/lib/guardian/dns/queries.db"
```

## Per-Child Overrides

```toml
# /etc/guardian/children/tommy.toml

[dns]
# Additional blocks for this child
block_categories = ["social", "gaming"]

# Time-based rules
[dns.schedule]
# Block gaming sites during homework hours
gaming_blocked = ["15:00-17:00"]  # Mon-Fri implied

# Allow social media on weekends only
social_allowed = ["saturday", "sunday"]
```

## Implementation

### DNS Server (using trust-dns)

```rust
use trust_dns_server::ServerFuture;
use trust_dns_resolver::TokioAsyncResolver;

pub struct GuardianDns {
    config: DnsConfig,
    blocklist: BlocklistMatcher,
    safesearch: SafeSearchRewriter,
    vpn_detector: VpnDetector,
    logger: QueryLogger,
    upstream: TokioAsyncResolver,
}

impl GuardianDns {
    pub async fn handle_query(&self, query: &Query) -> DnsResponse {
        let domain = query.name().to_string();
        let child = self.identify_child(query.source_ip());
        
        // 1. Check blocklist
        if let Some(category) = self.blocklist.check(&domain) {
            if self.should_block(&child, &category) {
                self.logger.log_blocked(&child, &domain, &category);
                return DnsResponse::blocked();
            }
        }
        
        // 2. Apply safe search rewrites
        if let Some(rewrite) = self.safesearch.rewrite(&domain) {
            self.logger.log_safesearch(&child, &domain, &rewrite);
            return self.resolve(&rewrite).await;
        }
        
        // 3. Check VPN/proxy
        if self.vpn_detector.is_vpn_domain(&domain) {
            self.logger.log_vpn_attempt(&child, &domain);
            self.alert_parent(&child, VpnAttempt(&domain));
            return DnsResponse::blocked();
        }
        
        // 4. Allow and log
        self.logger.log_allowed(&child, &domain);
        self.resolve(&domain).await
    }
}
```

### Blocklist Matching (Aho-Corasick for speed)

```rust
use aho_corasick::AhoCorasick;

pub struct BlocklistMatcher {
    // Fast multi-pattern matching
    exact_matcher: HashSet<String>,
    wildcard_matcher: AhoCorasick,
    category_map: HashMap<String, String>,
}

impl BlocklistMatcher {
    pub fn check(&self, domain: &str) -> Option<&str> {
        // Exact match first (fastest)
        if let Some(cat) = self.exact_matcher.get(domain) {
            return Some(cat);
        }
        
        // Wildcard match (*.example.com)
        if self.wildcard_matcher.is_match(domain) {
            return self.category_map.get(domain).map(|s| s.as_str());
        }
        
        None
    }
    
    // Load blocklists (millions of domains, <100MB RAM)
    pub async fn load_blocklists(&mut self, urls: &[String]) -> Result<()> {
        for url in urls {
            let content = reqwest::get(url).await?.text().await?;
            for line in content.lines() {
                if let Some(domain) = self.parse_hosts_line(line) {
                    self.exact_matcher.insert(domain, "blocklist".into());
                }
            }
        }
        Ok(())
    }
}
```

### Safe Search Rewriting

```rust
pub struct SafeSearchRewriter {
    rewrites: HashMap<String, String>,
}

impl SafeSearchRewriter {
    pub fn new() -> Self {
        let mut rewrites = HashMap::new();
        
        // Google Safe Search
        rewrites.insert("www.google.com".into(), "forcesafesearch.google.com".into());
        rewrites.insert("google.com".into(), "forcesafesearch.google.com".into());
        // ... all Google TLDs
        
        // Bing Strict
        rewrites.insert("www.bing.com".into(), "strict.bing.com".into());
        
        // YouTube Restricted
        rewrites.insert("www.youtube.com".into(), "restrictmoderate.youtube.com".into());
        rewrites.insert("youtube.com".into(), "restrictmoderate.youtube.com".into());
        rewrites.insert("m.youtube.com".into(), "restrictmoderate.youtube.com".into());
        
        // DuckDuckGo Safe
        rewrites.insert("duckduckgo.com".into(), "safe.duckduckgo.com".into());
        
        Self { rewrites }
    }
    
    pub fn rewrite(&self, domain: &str) -> Option<&str> {
        self.rewrites.get(domain).map(|s| s.as_str())
    }
}
```

### VPN Detection

```rust
pub struct VpnDetector {
    // Known VPN provider domains
    vpn_domains: HashSet<String>,
    
    // DNS-over-HTTPS providers (prevent DNS bypass)
    doh_domains: HashSet<String>,
    
    // Proxy services
    proxy_domains: HashSet<String>,
}

impl VpnDetector {
    pub fn new() -> Self {
        let mut vpn_domains = HashSet::new();
        
        // Commercial VPNs
        vpn_domains.insert("nordvpn.com".into());
        vpn_domains.insert("expressvpn.com".into());
        vpn_domains.insert("surfshark.com".into());
        vpn_domains.insert("privateinternetaccess.com".into());
        vpn_domains.insert("cyberghostvpn.com".into());
        vpn_domains.insert("protonvpn.com".into());
        // ... hundreds more
        
        let mut doh_domains = HashSet::new();
        // Block DoH to prevent DNS bypass
        doh_domains.insert("dns.google".into());
        doh_domains.insert("cloudflare-dns.com".into());
        doh_domains.insert("dns.quad9.net".into());
        doh_domains.insert("doh.opendns.com".into());
        
        Self { vpn_domains, doh_domains, proxy_domains: HashSet::new() }
    }
    
    pub fn is_vpn_domain(&self, domain: &str) -> bool {
        self.vpn_domains.contains(domain) ||
        self.doh_domains.contains(domain) ||
        self.proxy_domains.contains(domain)
    }
}
```

## System Integration

### 1. Set as System DNS

```bash
# /etc/systemd/resolved.conf.d/guardian.conf
[Resolve]
DNS=127.0.0.1
FallbackDNS=
DNSStubListener=no
```

### 2. Prevent DNS Bypass

```bash
# Firewall rules to force all DNS through guardian-dns
iptables -t nat -A OUTPUT -p udp --dport 53 ! -d 127.0.0.1 -j REDIRECT --to-port 53
iptables -t nat -A OUTPUT -p tcp --dport 53 ! -d 127.0.0.1 -j REDIRECT --to-port 53

# Block DNS-over-HTTPS (port 443 to known DoH providers)
# This is handled by blocking DoH domains in guardian-dns
```

### 3. Systemd Service

```ini
# /etc/systemd/system/guardian-dns.service
[Unit]
Description=Guardian DNS Filter
After=network.target
Before=guardian-daemon.service

[Service]
Type=simple
ExecStart=/usr/lib/guardian/guardian-dns
Restart=always
RestartSec=5

# Security hardening
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/guardian/dns

[Install]
WantedBy=multi-user.target
```

## Browsing Logs Schema

```sql
-- /var/lib/guardian/dns/queries.db

CREATE TABLE dns_queries (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    child_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    category TEXT,
    action TEXT NOT NULL,  -- 'allowed', 'blocked', 'safesearch'
    
    INDEX idx_timestamp (timestamp),
    INDEX idx_child (child_id),
    INDEX idx_domain (domain)
);

-- Automatic cleanup (30 day retention)
CREATE TRIGGER cleanup_old_queries
AFTER INSERT ON dns_queries
BEGIN
    DELETE FROM dns_queries 
    WHERE timestamp < datetime('now', '-30 days');
END;
```

## Parent Dashboard - Browsing View

```
┌─────────────────────────────────────────────────────────────────────┐
│  Guardian - Tommy's Browsing Activity                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  📅 Today                                                           │
│                                                                     │
│  Top Sites:                                                         │
│  ████████████████████ youtube.com (47 queries)                     │
│  ██████████████ minecraft.net (32 queries)                         │
│  ████████████ google.com (28 queries)                              │
│  ████████ roblox.com (18 queries)                                  │
│  ██████ discord.com (14 queries)                                   │
│                                                                     │
│  Categories:                                                        │
│  🎮 Gaming: 45%                                                     │
│  📺 Video: 30%                                                      │
│  🔍 Search: 15%                                                     │
│  💬 Social: 10%                                                     │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                     │
│  ⚠️ Blocked Attempts (3 today)                                     │
│                                                                     │
│  🚫 11:23 AM - nordvpn.com (VPN detected)                          │
│  🚫 2:45 PM - adult-site.com (Adult content)                       │
│  🚫 4:12 PM - proxysite.com (Proxy detected)                       │
│                                                                     │
│  [View Full History]  [Adjust Filters]  [Whitelist Site]           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Performance

| Metric | Value |
|--------|-------|
| Query latency | <10ms (cached), <50ms (upstream) |
| Memory usage | ~100MB (with 1M domain blocklist) |
| Blocklist load time | ~5 seconds |
| CPU usage | <1% idle, <5% under load |

## Related Documentation

- [Architecture Overview](./OVERVIEW.md)
- [VPN Detection Details](./VPN_DETECTION.md)
- [Router Integration](./ROUTER_INTEGRATION.md)
