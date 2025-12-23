# Guardian Platform - Complete Inventory

## Everything We've Built

### 1. Guardian Web Dashboard
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-web`  
**Tech:** Next.js 14, TypeScript, Tailwind, Supabase  
**Status:** ✅ Working

**Pages:**
- `/` - Landing page
- `/(auth)/login` - Magic link login
- `/(auth)/signup` - Parent registration
- `/(dashboard)/dashboard` - Main dashboard
- `/(dashboard)/children` - Manage children
- `/(dashboard)/devices` - Device enrollment & management
- `/(dashboard)/alerts` - Alert center
- `/(dashboard)/contacts` - Contact intelligence
- `/(dashboard)/browsing` - Browsing activity
- `/(dashboard)/settings` - Family settings
- `/(pwa)` - PWA manifest for mobile

**Components:**
- Alert cards with severity badges
- Child profile cards
- Device status indicators
- Screen time charts
- Contact risk scores

### 2. Guardian Sync Server
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-sync-server`  
**Tech:** Rust, Tonic (gRPC), SQLx, Redis  
**Status:** ✅ Running on 192.248.163.171

**Services:**
- `AuthService` - JWT tokens, device auth
- `SyncService` - Settings sync with conflict resolution
- `FileService` - S3 upload/download for game saves
- `FamilyService` - Family management

**Database Tables (25):**
```
accounts, auth_tokens, devices, device_commands
families, family_members, children, child_devices
screen_time_policies, screen_time_daily
app_catalog, app_policies, app_sessions
alerts, alert_actions
dns_profiles, url_logs
files, encryption_keys, watcher_groups
chat_channels, chat_messages_audit, chat_settings, friend_requests
```

### 3. Guardian Games OS
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-games-os`  
**Tech:** Bazzite (Fedora Atomic), Containerfile  
**Status:** 🔨 Ready to build

**Variants:**
- `guardian-games` - Base with integrated GPU
- `guardian-games-nvidia` - NVIDIA drivers
- `guardian-games-gnome` - GNOME desktop
- `guardian-games-deck` - Steam Deck optimized
- `guardian-games-htpc` - Living room PC
- `guardian-games-nvidia-open` - Open NVIDIA drivers

**Custom Files:**
- `/etc/guardian/` - Config directory
- `/usr/lib/systemd/system/guardian-*.service` - Systemd units
- `/usr/share/polkit-1/rules.d/guardian.rules` - Child account restrictions

### 4. Guardian Daemon
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-daemon`  
**Tech:** Rust, D-Bus, inotify  
**Status:** ⚠️ Needs compilation fixes

**Features:**
- Screen time tracking
- App monitoring
- DNS filtering (systemd-resolved)
- Policy enforcement
- Sync client

### 5. Guardian DNS
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-dns`  
**Tech:** Rust  
**Status:** 📋 Planned

**Features:**
- Local DNS cache
- Category-based blocking
- Blocklist sync from server

### 6. Guardian Settings
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-settings`  
**Tech:** GTK4, Rust  
**Status:** 📋 Planned

**Features:**
- Child profile switcher
- Time remaining display
- Extension requests

### 7. Guardian Store
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-store`  
**Tech:** TBD  
**Status:** 📋 Planned

**Features:**
- Curated game browser
- Age-appropriate filtering
- Parent approval workflow

### 8. Guardian Wizard
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-wizard`  
**Tech:** GTK4, Rust  
**Status:** 📋 Planned

**Features:**
- First-run setup
- Family enrollment
- Device pairing

---

## Supabase Schema (gkyspvcafyttfhyjryyk)

### Migration 001 - Core
```sql
families, parents, children, devices
contacts, contact_children
alerts, topic_summaries, browsing_summaries
```

### Migration 002 - Policies
```sql
screen_time_policies, dns_policies
app_policies, app_category_settings
device_commands, notification_settings
extension_requests, activity_logs
```

### Migration 003 - Chat (Ready to deploy)
```sql
chat_alerts, pending_friends, chat_settings_summary
```

---

## Infrastructure

### Sync Server (192.248.163.171)
- **OS:** Ubuntu 22.04.5 LTS
- **RAM:** 1GB (should upgrade to 2GB)
- **Disk:** 30GB (40% used)
- **Services:**
  - Traefik (reverse proxy, TLS)
  - MariaDB 11.2 (guardian_sync)
  - Redis 7 (sessions)
  - Guardian Sync Server (Rust, port 50052)

### Supabase (Cloud)
- **Project:** gkyspvcafyttfhyjryyk
- **Region:** (default)
- **Plan:** Free tier

### S3 Storage (Peasoup)
- **Endpoint:** s3.eu-west-1.peasoup.cloud
- **Bucket:** guardian-sync-files
- **Usage:** Game saves, profile blobs

### Domain
- **gameguardian.ai** (sync.gameguardian.ai for API)

---

## What's Missing for MVP

### Critical Path
1. ❌ Nakama container (chat server)
2. ❌ Nakama safety hooks (message filtering)
3. ❌ Guardian Games launcher (Heroic fork)
4. ❌ Parent chat visibility in dashboard
5. ❌ Build Guardian Games OS image

### Nice to Have
1. ❌ Voice chat with Whisper
2. ❌ Mobile parent app
3. ❌ Gamification/rewards
4. ❌ AI behavioral analysis

---

## Credentials Reference

| Service | Host | User | Password/Key |
|---------|------|------|--------------|
| Sync VPS | 192.248.163.171 | root | j?Q9qZ9]Y##qpafr |
| MariaDB | localhost:3306 | guardian | GuardianDB2025Pass |
| Redis | localhost:6379 | - | GuardianRedis2025Key |
| S3 | s3.eu-west-1.peasoup.cloud | PN29S8ZM3Q4LSAIK93V4 | bWnW9hQFm... |
| Supabase | gkyspvcafyttfhyjryyk | - | (dashboard) |

---

## Git Repositories

| Repo | GitHub | Description |
|------|--------|-------------|
| guardian-os-v1 | jonnyweareone/guardian-os-v1 | Main monorepo |
| guardian-games-os | jonnyweareone/guardian-games-os | Bazzite fork |
| guardian-web | (local only?) | Parent dashboard |

---

## Third-Party Dependencies

### Required
- Heroic Games Launcher (fork for guardian-games)
- Nakama (Apache 2.0, game backend)
- Bazzite (upstream OS)
- Supabase (auth, realtime)

### Optional
- LiveKit (voice chat)
- whisper.cpp (voice transcription)
- Llama Guard (content moderation)
- OneSignal (push notifications)

---

## Feature Ideas from Brainstorm

### From FEATURE_BRAINSTORM.md

| Feature | Priority | Complexity | Status |
|---------|----------|------------|--------|
| Gaming + Screen Time | P0 | Done | ✅ |
| Safe Chat | P0 | Medium | 🔨 In Progress |
| Safe Browsing (DNS) | P1 | Low | 📋 Planned |
| System Lockdown | P1 | Low | 📋 Planned |
| Device Management | P2 | Medium | 🔨 Partial |
| Advanced Screen Time | P2 | Medium | 📋 Planned |
| Media Controls | P2 | Low | 📋 Planned |
| Learning Apps | P3 | Low | 📋 Planned |
| Gamification | P3 | Medium | 📋 Planned |
| Family Hub | P4 | High | 💭 Ideas |
| AI Assistant | P4 | High | 💭 Ideas |
| Location Safety | P4 | Medium | 💭 Ideas |

### From AI_SAFETY_REALITY.md

| Layer | Implementation | Status |
|-------|----------------|--------|
| Deterministic filters | Regex, blocklists | 📋 Planned |
| ML classification | Perspective API | 📋 Planned |
| LLM analysis | Llama Guard | 💭 Future |
| Behavioral analysis | Custom ML | 💭 Future |
| Human review | Parent dashboard | ✅ Done |
| Structural controls | Whitelist mode | 📋 Planned |

---

## Summary

**What works today:**
- Parent dashboard (web)
- Family & child management
- Device enrollment
- Policy configuration
- Alert viewing
- Sync server running

**What's next:**
1. Deploy chat tables to Supabase
2. Add Nakama to sync server
3. Build safety hooks
4. Fork Heroic launcher
5. Build OS image

**Estimated time to MVP:**
- Chat working: 2-3 days
- Launcher ready: 1 week
- OS image: 1 day (once launcher ready)
- **Total: ~2 weeks**
