# Guardian OS v1.0.2 - Testing Guide

## What We Built This Session

### 1. Architecture Documentation
Complete documentation in `docs/architecture/`:
- System overview
- DNS & Network Shield
- Contact Intelligence (topic logging, risk scoring)
- Alert System (digest + real-time)
- Age Tiers (Under 10, 10-12, 13-15, 16-17)
- Privacy & Data Retention

### 2. Guardian DNS Component
A full DNS filtering server in `guardian-components/guardian-dns/`:
- Domain blocking (adult, malware, gambling)
- Safe search enforcement (Google, Bing, YouTube, DuckDuckGo)
- VPN/proxy detection and blocking
- Query logging (domains only, 30-day retention)

## Testing Guardian DNS on Vultr Server

### Step 1: SSH to the server
```bash
ssh root@136.244.71.108
```

### Step 2: Clone the repository
```bash
cd /root
git clone https://github.com/jonnyweareone/guardian-os-v1.git guardian-os-build
# Or if already cloned:
cd guardian-os-build && git pull
```

### Step 3: Install Rust (if not installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Step 4: Build Guardian DNS
```bash
cd /root/guardian-os-build/guardian-components/guardian-dns
cargo build --release
```

### Step 5: Create config directory
```bash
sudo mkdir -p /etc/guardian
sudo mkdir -p /var/lib/guardian/dns
sudo cp config/dns.toml /etc/guardian/
```

### Step 6: Run Guardian DNS
```bash
# Stop systemd-resolved if running (it uses port 53)
sudo systemctl stop systemd-resolved

# Run guardian-dns (needs root for port 53)
sudo ./target/release/guardian-dns
```

### Step 7: Test DNS filtering
In another terminal:

```bash
# Test normal resolution
dig @127.0.0.1 google.com

# Test safe search (should show forcesafesearch.google.com)
dig @127.0.0.1 www.google.com

# Test VPN blocking (should return NXDOMAIN)
dig @127.0.0.1 nordvpn.com

# Test proxy blocking
dig @127.0.0.1 proxysite.com
```

### Step 8: Install as service (optional)
```bash
sudo cp target/release/guardian-dns /usr/lib/guardian/
sudo cp config/guardian-dns.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable guardian-dns
sudo systemctl start guardian-dns
```

## Key Design Decisions Made

### Contact Intelligence
- **Topic logging**: "Gaming 45%, School 30%" - NOT actual messages
- **Risk scoring**: AI-powered, based on patterns not content
- **Family trust**: Shared scores across siblings
- **Vector memory**: Track contact relationships without storing PII

### Alert System
| Risk Level | Alert Type | Replay Access |
|------------|------------|---------------|
| < 0.3 | Weekly/daily digest | None |
| 0.3-0.5 | Note in digest | None |
| 0.5-0.7 | Push notification | None |
| 0.7-0.85 | Immediate push | 72 hours |
| > 0.85 | Emergency + escalation | 7 days |
| Grooming | Phone call escalation | 30 days |

### Age Tiers
| Age | PII Handling | Monitoring | Autonomy |
|-----|--------------|------------|----------|
| Under 10 | Block all | Full capture | None |
| 10-12 | Block critical, warn others | Full + alerts | Ask parent |
| 13-15 | Warn only (except address) | Pattern detection | Can dismiss warnings |
| 16-17 | Log only | Safety-critical only | Can disable most |

### DNS Features
- **Safe Search**: Google → forcesafesearch.google.com
- **YouTube**: Moderate by default (restrictmoderate.youtube.com)
- **VPN Blocking**: 30+ commercial VPNs blocked
- **DoH Blocking**: Prevents DNS bypass via encrypted DNS
- **Logging**: Domain + timestamp + action (30-day retention)

## Files Created

```
docs/
├── README.md                    # Documentation index
└── architecture/
    ├── OVERVIEW.md              # System architecture
    ├── DNS_NETWORK.md           # DNS filtering docs
    ├── CONTACT_INTELLIGENCE.md  # Contact tracking
    ├── ALERT_SYSTEM.md          # Parent notifications
    ├── AGE_TIERS.md             # Age-based protection
    └── PRIVACY.md               # Data retention policies

guardian-components/guardian-dns/
├── Cargo.toml                   # Rust dependencies
├── README.md                    # Component docs
├── build.sh                     # Build script
├── config/
│   ├── dns.toml                 # Default config
│   └── guardian-dns.service     # Systemd unit
└── src/
    ├── main.rs                  # Entry point
    ├── config.rs                # Configuration
    ├── server.rs                # DNS server
    ├── blocklist.rs             # Domain blocking
    ├── safesearch.rs            # Safe search rewrites
    ├── vpn_detect.rs            # VPN/proxy detection
    └── logger.rs                # Query logging
```

## Next Steps

### Immediate
1. Test guardian-dns build on Vultr server
2. Verify DNS filtering works correctly
3. Test safe search enforcement

### v1.0.2 Goals
1. ✅ Architecture documentation
2. ✅ Guardian DNS component
3. Custom wallpaper/branding (pending)
4. Sync server integration (pending)

### Future Development
1. Guardian daemon (orchestration)
2. Topic classifier (Phi-3-mini)
3. Contact intelligence (vector store)
4. Parent dashboard (web app)
5. Mobile companion app
6. CEOP integration (UK)

## Git Commits This Session
- `e0fabfa` - Add comprehensive architecture docs and guardian-dns component
- `a576f1c` - Add guardian-dns build scripts and systemd service
