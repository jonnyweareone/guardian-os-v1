# Guardian Games OS - Complete Architecture v2.0

## The Big Picture

We have **three data stores** that each serve different purposes:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         GUARDIAN PLATFORM ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────────┐ │
│  │    SUPABASE         │  │    SYNC SERVER      │  │      NAKAMA             │ │
│  │    (Cloud)          │  │    (Self-hosted)    │  │      (Self-hosted)      │ │
│  │                     │  │                     │  │                         │ │
│  │  👨‍👩‍👧‍👦 Parent Portal   │  │  🔄 Device Sync     │  │  💬 Real-time Chat     │ │
│  │  📊 Analytics       │  │  📁 File Storage    │  │  👥 Friends/Presence    │ │
│  │  🔔 Alerts          │  │  ⚙️ Settings Sync   │  │  🎮 Matchmaking         │ │
│  │  📱 Web/Mobile App  │  │  🎮 Game Saves      │  │  🏆 Leaderboards        │ │
│  │                     │  │                     │  │                         │ │
│  │  Auth: Supabase     │  │  Auth: JWT (sync)   │  │  Auth: Guardian Token   │ │
│  │  Users: Parents     │  │  Users: Devices     │  │  Users: Kids (gaming)   │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────────┘ │
│           │                         │                         │                │
│           └─────────────────────────┼─────────────────────────┘                │
│                                     │                                          │
│                                     ▼                                          │
│                    ┌─────────────────────────────────┐                         │
│                    │      GUARDIAN GAMES OS          │                         │
│                    │      (Bazzite Fork)             │                         │
│                    │                                 │                         │
│                    │  guardian-syncd → Sync Server   │                         │
│                    │  guardian-games → Nakama        │                         │
│                    │  guardian-portal → Supabase     │                         │
│                    └─────────────────────────────────┘                         │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Data Store Responsibilities

### 1. Supabase (gkyspvcafyttfhyjryyk) - Parent Portal & Control Plane

**Purpose:** Web/mobile parent dashboard, authentication, policies, alerts

**Why Supabase:**
- Built-in auth (magic links, OAuth)
- Real-time subscriptions (instant alerts)
- Row Level Security (parents only see their data)
- Edge Functions (webhooks, notifications)
- Free tier is generous

**Tables (existing):**
```sql
-- Core
families, parents, children, devices

-- Policies  
screen_time_policies, dns_policies, app_policies, app_category_settings

-- Monitoring
alerts, contacts, contact_children, topic_summaries, browsing_summaries

-- Control
device_commands, extension_requests, activity_logs, notification_settings
```

**API Consumers:**
- guardian-web (Next.js parent dashboard)
- guardian-mobile (future Flutter app)
- guardian-daemon (policy sync)

---

### 2. Sync Server (192.248.163.171) - Device Sync & Game Data

**Purpose:** Profile roaming, settings sync, game saves, device-to-device sync

**Why Self-Hosted:**
- Game saves can be large (GB)
- Low latency for settings sync
- Full control over data
- S3-compatible storage (Peasoup)
- No vendor lock-in

**Tables (existing in MariaDB):**
```sql
-- Accounts & Auth
accounts, auth_tokens, devices, device_commands

-- Families & Children
families, family_members, children, child_devices

-- Policies (synced FROM Supabase)
screen_time_policies, screen_time_daily, app_policies, app_catalog, app_sessions

-- Content Filtering
dns_profiles, url_logs

-- Alerts (synced TO Supabase)
alerts, alert_actions

-- Files & Encryption
files, encryption_keys, watcher_groups
```

**External Storage:**
- S3 (Peasoup): `s3.eu-west-1.peasoup.cloud/guardian-sync-files`
  - Profile blobs
  - Game saves
  - Wallpapers
  - Config backups

**API Consumers:**
- guardian-syncd (Rust daemon on devices)
- guardian-portal (first-boot enrollment)

---

### 3. Nakama (NEW) - Real-time Gaming Social

**Purpose:** Chat, friends, presence, matchmaking, leaderboards

**Why Nakama:**
- Purpose-built for games
- WebSocket real-time
- Built-in chat, friends, groups
- Lua/TypeScript hooks for safety
- Self-hosted (privacy)

**Tables (Nakama's Postgres):**
```sql
-- Nakama manages these internally
users, user_edge (friends), channel_message, notification, leaderboard, etc.
```

**Custom Hooks (TypeScript):**
```
/nakama/modules/
├── guardian-auth.ts      # Auth via Guardian token
├── chat-safety.ts        # 5-layer message filter
├── voice-safety.ts       # Whisper transcription filter
├── parent-visibility.ts  # Log all chats for parents
└── age-restrictions.ts   # Feature gating by age
```

**API Consumers:**
- guardian-games (Electron app / Heroic fork)
- In-game chat widgets

---

## Data Flow Architecture

### Flow 1: Parent Creates Family (Web)

```
Parent signs up → Supabase Auth
                      │
                      ▼
              Supabase trigger creates:
              - family
              - parent
                      │
                      ▼
              Parent adds children in dashboard
                      │
                      ▼
              Default policies auto-created
```

### Flow 2: Device Enrollment

```
Guardian OS boots → First-run wizard
                          │
                          ▼
                    Supabase Auth (parent login)
                          │
                          ▼
                    Get family_id, create device in Supabase
                          │
                          ▼
                    Generate Sync Server JWT
                          │
                          ▼
                    Register device with Sync Server
                          │
                          ▼
                    Pull policies, start syncing
```

### Flow 3: Child Uses Device

```
Child selects profile → guardian-syncd loads their policies
                              │
                              ▼
                        Screen time tracking starts
                              │
                              ▼
                        DNS filtering active (via systemd-resolved)
                              │
                              ▼
                        guardian-games launched
                              │
                              ▼
                        Nakama auth (Guardian token → Nakama session)
                              │
                              ▼
                        Chat safety hooks active
```

### Flow 4: Chat Message

```
Child sends message → Nakama beforeChannelMessageSend hook
                              │
                              ▼
                        Layer 1: Whitelist check (if age < 10)
                              │
                              ▼
                        Layer 2: Profanity filter
                              │
                              ▼
                        Layer 3: PII detection
                              │
                              ▼
                        Layer 4: Predator pattern check
                              │
                              ▼
                        Layer 5: Log for parent visibility
                              │
                              ├── PASS → Message delivered
                              │
                              └── BLOCK → Message rejected
                                         │
                                         ▼
                                   Alert to Supabase (if critical)
                                         │
                                         ▼
                                   Push notification to parent
```

### Flow 5: Voice Chat (with Whisper)

```
Child speaks → Local Whisper transcription (whisper.cpp)
                    │
                    ▼
              Same 5-layer filter as text
                    │
                    ├── PASS → Audio transmitted via LiveKit/WebRTC
                    │
                    └── BLOCK → Audio muted, warning tone plays
```

### Flow 6: Parent Views Child Activity

```
Parent opens dashboard → Supabase (alerts, summaries)
                              │
                              ▼
                        Click "View Chat History"
                              │
                              ▼
                        API call to Sync Server
                              │
                              ▼
                        Sync Server queries Nakama
                              │
                              ▼
                        Returns chat logs with safety metadata
```

---

## Server Infrastructure

### Current: 192.248.163.171 (1GB RAM VPS)

```
┌─────────────────────────────────────────────────────────────┐
│  CURRENT STATE                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ✅ Traefik (reverse proxy, TLS)                           │
│  ✅ MariaDB 11.2 (guardian_sync database)                  │
│  ✅ Redis 7 (sessions, cache)                              │
│  ✅ Guardian Sync Server (Rust binary, port 50052)         │
│                                                             │
│  Memory: ~350MB used / 1GB total                           │
│  Disk: 12GB used / 30GB total                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Recommended: Upgrade to 2GB+ RAM

```
┌─────────────────────────────────────────────────────────────┐
│  RECOMMENDED SETUP (2GB RAM minimum)                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Traefik          ~50MB   (reverse proxy)                  │
│  MariaDB         ~200MB   (sync data)                      │
│  Redis            ~50MB   (cache, pubsub)                  │
│  Guardian Sync   ~100MB   (Rust server)                    │
│  Postgres        ~200MB   (Nakama data)                    │
│  Nakama          ~150MB   (chat server)                    │
│  ─────────────────────────                                 │
│  TOTAL           ~750MB   (leaves headroom)                │
│                                                             │
│  Optional (needs 4GB+ or separate server):                 │
│  LiveKit         ~300MB   (voice/video)                    │
│  Whisper API     ~500MB   (voice transcription)            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Database Schema Additions

### For Sync Server (MariaDB) - Chat Support

```sql
-- Chat audit for parent visibility
CREATE TABLE chat_messages_audit (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    message_uuid CHAR(36) NOT NULL UNIQUE,
    child_uuid CHAR(36) NOT NULL,
    channel_type ENUM('family', 'party', 'direct') NOT NULL,
    other_user_uuid CHAR(36),
    other_user_name VARCHAR(255),
    message_text TEXT NOT NULL,
    message_type ENUM('text', 'voice_phrase', 'emoji') DEFAULT 'text',
    was_filtered BOOLEAN DEFAULT FALSE,
    was_blocked BOOLEAN DEFAULT FALSE,
    filter_reason VARCHAR(100),
    threat_level ENUM('none', 'low', 'medium', 'high', 'critical') DEFAULT 'none',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_child_time (child_uuid, created_at),
    INDEX idx_blocked (child_uuid, was_blocked)
);

-- Friend approvals
CREATE TABLE friend_requests (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    request_uuid CHAR(36) NOT NULL UNIQUE,
    child_uuid CHAR(36) NOT NULL,
    friend_uuid CHAR(36) NOT NULL,
    friend_username VARCHAR(255) NOT NULL,
    status ENUM('pending', 'approved', 'denied', 'blocked') DEFAULT 'pending',
    requires_parent_approval BOOLEAN DEFAULT TRUE,
    requested_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    decided_at DATETIME,
    decided_by CHAR(36),
    
    INDEX idx_child (child_uuid, status),
    INDEX idx_pending (status, requires_parent_approval)
);

-- Chat settings per child
CREATE TABLE chat_settings (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    child_uuid CHAR(36) NOT NULL UNIQUE,
    chat_mode ENUM('disabled', 'whitelist', 'filtered', 'monitored', 'standard') DEFAULT 'filtered',
    voice_enabled BOOLEAN DEFAULT TRUE,
    voice_mode ENUM('disabled', 'phrases_only', 'filtered', 'standard') DEFAULT 'phrases_only',
    family_chat_only BOOLEAN DEFAULT FALSE,
    require_friend_approval BOOLEAN DEFAULT TRUE,
    max_friends INTEGER DEFAULT 20,
    custom_blocked_words TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### For Supabase - Chat Alerts

```sql
-- Chat alerts (synced from Nakama via Sync Server)
CREATE TABLE chat_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    family_id UUID NOT NULL REFERENCES families(id) ON DELETE CASCADE,
    child_id UUID NOT NULL REFERENCES children(id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL, -- 'profanity', 'pii', 'predator', 'blocked'
    severity TEXT NOT NULL,   -- 'low', 'medium', 'high', 'critical'
    other_user_name TEXT,
    message_preview TEXT,
    full_message_available BOOLEAN DEFAULT TRUE,
    acknowledged_by UUID REFERENCES parents(id),
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Friend approval requests (synced from Sync Server)
CREATE TABLE pending_friends (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    child_id UUID NOT NULL REFERENCES children(id) ON DELETE CASCADE,
    friend_username TEXT NOT NULL,
    friend_platform TEXT DEFAULT 'guardian', -- for future cross-platform
    status TEXT DEFAULT 'pending',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_by UUID REFERENCES parents(id),
    decided_at TIMESTAMPTZ
);

-- Enable RLS
ALTER TABLE chat_alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE pending_friends ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Parents see own chat_alerts" ON chat_alerts
    FOR ALL USING (
        family_id IN (SELECT family_id FROM parents WHERE user_id = auth.uid())
    );

CREATE POLICY "Parents manage pending_friends" ON pending_friends
    FOR ALL USING (
        child_id IN (
            SELECT id FROM children WHERE family_id IN (
                SELECT family_id FROM parents WHERE user_id = auth.uid()
            )
        )
    );
```

---

## Chat Safety Age Matrix

| Age | Chat Mode | Voice Mode | Friends | Parent Visibility |
|-----|-----------|------------|---------|-------------------|
| 6-9 | Whitelist | Phrases Only | Family Only | 100% |
| 10-12 | Filtered | Filtered | Parent Approved | 100% |
| 13-15 | Monitored | Filtered | Anyone (reviewable) | On Request |
| 16+ | Standard | Standard | Anyone | On Request |

### Whitelist Mode (~150 phrases)

```
Greetings: hi, hello, hey, bye, goodbye, see you, later
Gaming: good game, gg, nice shot, well played, lets play, ready, go
Gameplay: left, right, up, down, here, follow me, behind you, help
Reactions: yes, no, ok, cool, awesome, nice, wow, oops, lol
Social: friend, team, group, party, join, invite, thanks
```

### Filtered Mode

All messages pass through:
1. Profanity filter (block + replace)
2. PII detection (block phone/email/address)
3. Predator patterns (block + alert parents)
4. Rate limiting (prevent spam)

### Voice Phrases Only (Age 6-9)

Pre-recorded voice clips that kids can trigger with buttons:
- No free-form speech
- 100% safe content
- Still social and fun

---

## Service Communication

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           SERVICE COMMUNICATION                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  guardian-web (Next.js)                                                         │
│       │                                                                         │
│       ├──► Supabase (direct, RLS protected)                                    │
│       │    - Auth, families, children, alerts                                  │
│       │                                                                         │
│       └──► Supabase Edge Function ──► Sync Server                              │
│            - Chat history requests                                             │
│            - Friend approval sync                                              │
│                                                                                 │
│  guardian-syncd (Rust daemon)                                                   │
│       │                                                                         │
│       ├──► Sync Server (gRPC, port 50051)                                      │
│       │    - Settings sync, file sync                                          │
│       │                                                                         │
│       └──► Supabase (via Sync Server proxy)                                    │
│            - Policy fetch, alert push                                          │
│                                                                                 │
│  guardian-games (Electron)                                                      │
│       │                                                                         │
│       ├──► Nakama (WebSocket, port 7350)                                       │
│       │    - Chat, friends, presence                                           │
│       │                                                                         │
│       └──► Sync Server (gRPC)                                                  │
│            - Game save sync                                                    │
│                                                                                 │
│  Nakama                                                                         │
│       │                                                                         │
│       └──► Sync Server (HTTP callback)                                         │
│            - Log messages for parent visibility                                │
│            - Push critical alerts                                              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Deployment Priority

### Phase 1: Core (Now)
- [x] Supabase schema deployed
- [x] Sync Server running (MariaDB + Redis)
- [ ] Add chat tables to Sync Server
- [ ] Fix Traefik Docker API issue
- [ ] Deploy Nakama container

### Phase 2: Chat Safety
- [ ] Nakama safety hooks (TypeScript)
- [ ] Chat audit logging
- [ ] Parent chat visibility API
- [ ] Friend approval flow

### Phase 3: Voice Safety
- [ ] Whisper integration (whisper.cpp)
- [ ] Voice phrase mode for young kids
- [ ] LiveKit for voice transport

### Phase 4: Full Integration
- [ ] Guardian Games launcher with chat
- [ ] Parent dashboard chat view
- [ ] Push notifications for alerts

---

## Cost Analysis

| Service | Current | With Nakama | Notes |
|---------|---------|-------------|-------|
| Supabase | Free tier | Free tier | Under limits |
| Sync VPS | $6/mo (1GB) | $12/mo (2GB) | Need more RAM |
| Peasoup S3 | ~$5/mo | ~$5/mo | Game saves |
| Domain | $12/yr | $12/yr | gameguardian.ai |
| **Total** | **~$12/mo** | **~$18/mo** | Very affordable |

---

## Summary

**Three-tier architecture:**

1. **Supabase** = Parent-facing (auth, dashboard, alerts)
2. **Sync Server** = Device-facing (settings, files, game saves)  
3. **Nakama** = Gaming-facing (chat, friends, matchmaking)

**All connected by:**
- JWT tokens (Supabase → Sync Server → Nakama)
- Webhooks (Nakama → Sync Server → Supabase)
- Edge Functions (Supabase → Sync Server)

**Result:** The safest gaming platform for kids, with:
- 5-layer chat safety
- Voice monitoring with Whisper
- 100% parent visibility
- Age-appropriate restrictions
- Immutable OS (can't bypass)
