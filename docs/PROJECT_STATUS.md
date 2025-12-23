# Guardian Games OS - Project Status & Next Steps

## What We Have Built

### 1. Guardian Web Dashboard (Supabase + Next.js)
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-web`
**Database:** Supabase `gkyspvcafyttfhyjryyk`

**Features:**
- ✅ Parent authentication (magic links, OAuth)
- ✅ Family & children management
- ✅ Device enrollment with codes (GOS-XXXX-XXXX)
- ✅ Screen time policies (weekday/weekend limits, bedtime)
- ✅ DNS/content filtering policies
- ✅ App policies and approval workflow
- ✅ Alert system with severity tiers
- ✅ Contact intelligence (hashed, privacy-preserving)
- ✅ Row Level Security (parents only see their data)
- 🆕 Chat alerts table (ready to deploy)
- 🆕 Friend approval workflow (ready to deploy)

### 2. Guardian Sync Server (Rust + MariaDB + Redis)
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-sync-server`
**Server:** `192.248.163.171`

**Features:**
- ✅ gRPC API (auth, sync, file, family services)
- ✅ JWT authentication
- ✅ MariaDB with 21 tables
- ✅ Redis for sessions/cache
- ✅ S3 integration (Peasoup)
- ✅ Device registration
- ✅ Settings sync with conflict resolution
- ✅ Screen time tracking
- 🆕 Chat tables migration (ready to deploy)

### 3. Guardian Games OS (Bazzite Fork)
**Repo:** `/Users/davidsmith/Documents/GitHub/guardian-games-os`
**Registry:** `ghcr.io/jonnyweareone/guardian-games`

**Features:**
- ✅ Containerfile for OCI build
- ✅ 6 variants (main, nvidia, gnome, deck, etc.)
- ✅ GitHub Actions CI/CD
- ✅ Systemd services defined
- ✅ Polkit rules for child accounts
- ❌ Not yet built (needs GitHub Actions run)

### 4. Guardian Daemon (Rust)
**Location:** `/Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-components/guardian-daemon`

**Features:**
- ✅ DBus interface
- ✅ Screen time enforcement
- ✅ DNS filtering
- ⚠️ Some compilation errors to fix

### 5. Safe Chat System (Designed)
**Spec:** `/mnt/user-data/uploads/SAFE_CHAT.md`

**Features:**
- ✅ Nakama integration plan
- ✅ 5-layer safety filter
- ✅ Whitelist mode for young kids
- ✅ Voice safety with Whisper
- ✅ Parent visibility
- ❌ Not yet implemented

---

## Data Store Summary

| Store | Location | Purpose | Tables |
|-------|----------|---------|--------|
| **Supabase** | Cloud | Parent portal, auth, alerts | families, parents, children, devices, alerts, policies |
| **Sync Server** | 192.248.163.171 | Device sync, game saves | accounts, families, children, settings, files |
| **Nakama** | TBD | Real-time chat, friends | (Nakama internal) |

---

## Immediate Next Steps

### Priority 1: Deploy Chat Support

1. **Run chat migration on Sync Server:**
```bash
ssh root@192.248.163.171
docker exec guardian-mariadb mariadb -uguardian -pGuardianDB2025Pass guardian_sync < /path/to/003_chat_support.sql
```

2. **Deploy Supabase migration:**
```bash
cd /Users/davidsmith/Documents/GitHub/guardian-web
supabase db push
```

3. **Add Nakama to sync server:**
```bash
# On 192.248.163.171
docker-compose up -d nakama nakama-postgres
```

### Priority 2: Fix Traefik

The Traefik container has a Docker API version mismatch. Fix:
```bash
docker pull traefik:latest
docker-compose down traefik
docker-compose up -d traefik
```

### Priority 3: Build Guardian Games OS

Trigger GitHub Actions:
```bash
cd /Users/davidsmith/Documents/GitHub/guardian-games-os
git push origin main
```

Add `SIGNING_SECRET` to repo secrets for cosign.

### Priority 4: Complete Heroic Fork

Start the Guardian Games launcher:
1. Fork Heroic Games Launcher
2. Add Guardian authentication
3. Integrate Nakama chat
4. Add age-based game filtering

---

## Architecture Decision: Where Does Data Live?

### Parent-Facing Data → Supabase
- Authentication (magic links, OAuth)
- Family configuration
- Alerts and notifications
- Dashboard analytics
- Real-time subscriptions

### Device-Facing Data → Sync Server
- Settings sync (dconf, KDE, etc.)
- Game save files (S3)
- Chat message audit logs
- Friend lists and blocks
- Screen time usage logs

### Gaming/Social Data → Nakama
- Real-time chat
- Friend presence (online/offline)
- Party voice chat
- Matchmaking
- Leaderboards

### Sync Flow
```
Device → Sync Server → (important alerts) → Supabase → Parent Dashboard
                    ↓
              Nakama (chat)
```

---

## Server Resource Planning

### Current (1GB RAM)
- MariaDB: 200MB
- Redis: 50MB
- Guardian Sync: 100MB
- **Total: ~350MB** ✅

### With Nakama (need 2GB RAM)
- MariaDB: 200MB
- Redis: 50MB
- Guardian Sync: 100MB
- Postgres (Nakama): 200MB
- Nakama: 150MB
- **Total: ~700MB** ✅

### With Voice (need 4GB RAM)
- Above + LiveKit: 300MB
- Above + Whisper: 500MB+
- **Total: ~1.5GB** (need dedicated server for voice)

**Recommendation:** Upgrade VPS to 2GB for Nakama. Consider separate server for voice later.

---

## Repository Structure

```
/Users/davidsmith/Documents/GitHub/
├── guardian-os-v1/             # Main monorepo
│   ├── guardian-sync-server/   # Rust sync server
│   ├── guardian-components/    # OS components
│   │   ├── guardian-daemon/    # System daemon
│   │   ├── guardian-portal/    # First-run wizard (planned)
│   │   └── guardian-shell/     # GNOME extension (planned)
│   ├── branding/               # Logos, wallpapers
│   └── docs/                   # Architecture docs
│
├── guardian-games-os/          # Bazzite fork (separate repo)
│   ├── Containerfile           # OCI build
│   ├── system_files/           # OS customizations
│   └── .github/workflows/      # CI/CD
│
├── guardian-web/               # Parent dashboard
│   ├── src/                    # Next.js app
│   └── supabase/               # DB migrations
│
└── game-guardian/              # Old prototype (deprecated?)
```

---

## Feature Roadmap

### v1.0 - Core (Current)
- [x] Parent dashboard
- [x] Device enrollment
- [x] Screen time
- [x] DNS filtering
- [ ] Basic chat (text only)

### v1.1 - Safe Chat
- [ ] Nakama integration
- [ ] Profanity filter
- [ ] PII detection
- [ ] Whitelist mode
- [ ] Parent chat visibility

### v1.2 - Voice Safety
- [ ] Whisper transcription
- [ ] Voice phrase mode
- [ ] LiveKit integration
- [ ] Real-time voice filtering

### v2.0 - Full Platform
- [ ] Guardian Games launcher
- [ ] Game rating integration
- [ ] Achievements/rewards
- [ ] Family hub features

---

## Key Credentials (for reference)

### Supabase
- Project: `gkyspvcafyttfhyjryyk`
- URL: `https://gkyspvcafyttfhyjryyk.supabase.co`

### Sync Server
- IP: `192.248.163.171`
- SSH: `root` / `j?Q9qZ9]Y##qpafr`
- MariaDB: `guardian` / `GuardianDB2025Pass`
- Redis: `GuardianRedis2025Key`

### S3 (Peasoup)
- Endpoint: `s3.eu-west-1.peasoup.cloud`
- Bucket: `guardian-sync-files`
- Access: `PN29S8ZM3Q4LSAIK93V4`

### Domain
- `gameguardian.ai` (or `sync.gameguardian.ai`)

---

## Summary

We have a solid foundation:
- ✅ Parent dashboard working
- ✅ Sync server running
- ✅ Database schemas designed
- ✅ Bazzite OS configured

Next immediate actions:
1. Deploy chat migrations to both databases
2. Add Nakama container to sync server
3. Build Guardian Games OS via GitHub Actions
4. Start Heroic Games Launcher fork

The goal: **World's safest gaming platform for kids**, with:
- 5-layer chat safety
- Voice monitoring
- 100% parent visibility
- Immutable OS (unbypassable)
