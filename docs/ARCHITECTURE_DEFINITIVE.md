# Guardian Platform - Definitive Architecture Guide

## The Core Question: Where Does Each Feature Live?

After reviewing FEATURE_BRAINSTORM.md and AI_SAFETY_REALITY.md, here's the clear separation:

---

## The Three Pillars

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│   SUPABASE (Cloud)              SYNC SERVER (VPS)           NAKAMA (VPS)       │
│   ─────────────────             ─────────────────           ───────────        │
│                                                                                 │
│   👨‍👩‍👧‍👦 WHO USES IT:              🖥️ WHO USES IT:             🎮 WHO USES IT:    │
│   Parents (web/mobile)          Devices (OS daemon)         Kids (gaming)      │
│                                                                                 │
│   🔐 AUTH:                      🔐 AUTH:                    🔐 AUTH:           │
│   Supabase Auth                 JWT from Supabase           Token from Sync    │
│   (magic link, Google)          (device enrollment)         (Guardian token)   │
│                                                                                 │
│   💾 DATA STORED:               💾 DATA STORED:             💾 DATA STORED:    │
│   - Family config               - Device settings           - Real-time chat   │
│   - Alert summaries             - Game saves (S3)           - Friend lists     │
│   - Dashboard data              - Full chat logs            - Presence         │
│   - Push notification IDs       - Screen time logs          - Matchmaking      │
│                                                                                 │
│   ⚡ REAL-TIME:                 ⚡ REAL-TIME:               ⚡ REAL-TIME:      │
│   Supabase Realtime             gRPC streaming              WebSocket          │
│   (alert notifications)         (settings sync)             (chat, presence)   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Feature-by-Feature Breakdown

### 1. 🎮 Gaming (Heroic Fork)

| Component | Where | Why |
|-----------|-------|-----|
| Game library | Device (local) | Fast access, offline support |
| Game saves | Sync Server → S3 | Large files, cross-device sync |
| Play time tracking | Sync Server | Real-time enforcement |
| Age ratings | Sync Server | PEGI/ESRB database cached locally |
| Game approval workflow | Supabase | Parents approve via web |

### 2. 💬 Safe Chat

| Component | Where | Why |
|-----------|-------|-----|
| Real-time messaging | Nakama | Purpose-built for games |
| Message filtering | Nakama hooks | Intercept before delivery |
| Chat logs (audit) | Sync Server | Parent visibility, compliance |
| Alert summaries | Supabase | Push to parent dashboard |
| Friend approvals | Supabase | Parents decide via web |
| Blocked users | Sync Server | Enforced on device |

### 3. 🌐 Safe Browsing

| Component | Where | Why |
|-----------|-------|-----|
| DNS filtering | Device (systemd-resolved) | No latency |
| Blocklist sync | Sync Server | Centralized updates |
| Browsing history | Sync Server | Summarized for parents |
| Category policies | Supabase | Parents configure via web |
| VPN detection | Device | Local network monitoring |

### 4. ⏰ Screen Time

| Component | Where | Why |
|-----------|-------|-----|
| Time tracking | Device (daemon) | Accurate, offline-capable |
| Usage logs | Sync Server | Historical data |
| Policies | Supabase → Sync Server | Parents set, devices enforce |
| Extension requests | Supabase | Real-time parent approval |
| Bedtime enforcement | Device | Must work offline |

### 5. 📱 Device Management

| Component | Where | Why |
|-----------|-------|-----|
| Device registration | Supabase | Parent enrolls device |
| Device status | Sync Server | Heartbeat every 5 min |
| Remote commands | Supabase → Sync Server | Lock, message, etc. |
| OS updates | Device (rpm-ostree) | Immutable updates |
| App installs | Device (Flatpak) | Parent-approved list |

### 6. 🔔 Alerts & Notifications

| Component | Where | Why |
|-----------|-------|-----|
| Alert generation | Device/Nakama | Detected locally |
| Alert storage | Supabase | Parent dashboard |
| Push notifications | Supabase → OneSignal | Real-time to phone |
| Weekly reports | Supabase Edge Functions | Email digest |

### 7. 🏆 Gamification (Future)

| Component | Where | Why |
|-----------|-------|-----|
| Points/XP | Sync Server | Calculated from usage |
| Achievements | Nakama | Built-in leaderboards |
| Rewards | Supabase | Parents define rewards |
| Chore tracking | Supabase | Parents assign/verify |

### 8. 🤖 AI Safety (Future)

| Component | Where | Why |
|-----------|-------|-----|
| Text classification | Nakama hooks | Real-time filtering |
| LLM analysis | Sync Server (batch) | Expensive, async |
| Behavioral analysis | Sync Server | Cross-conversation patterns |
| Model updates | Sync Server | Centralized ML models |

---

## Data Flow Diagrams

### Flow A: Parent Sets Screen Time Limit

```
Parent (Web) → Supabase (policies table)
                    │
                    ▼
              Supabase Realtime trigger
                    │
                    ▼
              Edge Function → Sync Server API
                    │
                    ▼
              Sync Server stores in MariaDB
                    │
                    ▼
              Device daemon polls (every 60s) or receives push
                    │
                    ▼
              Device enforces new limit
```

### Flow B: Child Sends Chat Message

```
Child types message → Guardian Games (Electron)
                           │
                           ▼
                      Nakama WebSocket
                           │
                           ▼
                      beforeChannelMessageSend hook
                           │
                           ├── Layer 1: Whitelist check (age < 10)
                           ├── Layer 2: Profanity filter
                           ├── Layer 3: PII detection
                           ├── Layer 4: Predator patterns
                           └── Layer 5: Log for parents
                           │
                           ├── PASS → Deliver to recipient
                           │
                           └── BLOCK → Reject + log
                                  │
                                  ▼
                            Nakama calls Sync Server HTTP
                                  │
                                  ▼
                            Sync Server stores in chat_messages_audit
                                  │
                                  ▼ (if high severity)
                            Sync Server → Supabase (chat_alerts)
                                  │
                                  ▼
                            Supabase Realtime → Parent Dashboard
                                  │
                                  ▼
                            OneSignal Push → Parent Phone
```

### Flow C: Voice Chat with Whisper

```
Child speaks → Microphone → VAD (voice activity detection)
                                 │
                                 ▼
                           Local Whisper (whisper.cpp tiny model)
                                 │
                                 ▼
                           Transcript text
                                 │
                                 ▼
                           Same 5-layer filter as text chat
                                 │
                                 ├── PASS → Audio → LiveKit → Other players
                                 │
                                 └── BLOCK → Mute + warning tone
                                        │
                                        ▼
                                  Log + alert (same as text)
```

---

## Why This Split?

### Supabase is for Parents
- **Auth:** Magic links, Google OAuth - no passwords to remember
- **RLS:** Row Level Security means families can't see each other's data
- **Realtime:** Instant alerts when something happens
- **Edge Functions:** Webhooks, email, push notifications
- **Free tier:** Generous for our use case

### Sync Server is for Devices
- **Self-hosted:** Full control, no data leaves our servers
- **Game saves:** Can be gigabytes, need S3-compatible storage
- **Low latency:** Settings sync needs to be fast
- **Offline support:** Devices cache policies locally
- **Compliance:** Full audit logs for COPPA

### Nakama is for Gaming Social
- **Purpose-built:** Chat, friends, matchmaking out of the box
- **WebSocket:** True real-time, not polling
- **Hooks:** TypeScript/Lua for custom safety logic
- **Self-hosted:** Critical for child safety (no third-party sees chats)
- **Scales:** Handles thousands of concurrent users

---

## What We're NOT Using

| Technology | Why Not |
|------------|---------|
| Firebase | Google lock-in, harder to self-host |
| MongoDB | We need relational data (families → children → devices) |
| Socket.io | Nakama already handles WebSocket better |
| Discord bots | Third-party sees all messages |
| Roblox API | Can't filter messages before delivery |
| Cloud ML APIs | Data leaves our control, latency, cost |

---

## Cost Comparison

| Service | Monthly Cost | Notes |
|---------|--------------|-------|
| **Supabase** | $0 | Free tier (500MB DB, 2GB storage) |
| **Sync Server VPS** | $12 | 2GB RAM, enough for MariaDB + Nakama |
| **Peasoup S3** | ~$5 | Game saves, backups |
| **Domain** | $1 | gameguardian.ai |
| **Total** | **~$18/mo** | Scales to 1000s of families |

Compare to:
- Qustodio: $100/year per family
- Bark: $100/year per family
- Circle: $130 hardware + $10/month

We can run the **entire platform** for less than **one family's** subscription to competitors.

---

## Migration Path from Current State

### Already Done ✅
1. Supabase schema (families, children, policies, alerts)
2. Sync Server running (MariaDB, Redis, Rust binary)
3. Chat tables added to Sync Server
4. Guardian Games OS Containerfile

### Next Steps 🔜
1. Deploy Supabase chat_alerts migration
2. Add Nakama container to Sync Server
3. Build Nakama safety hooks
4. Connect Guardian Games to Nakama
5. Build parent chat visibility in dashboard

### Future 🔮
1. Whisper voice filtering
2. LiveKit for voice chat
3. Behavioral ML models
4. Mobile parent app (Flutter)

---

## Summary

| Question | Answer |
|----------|--------|
| Where do parents manage things? | **Supabase** (web dashboard) |
| Where do devices sync settings? | **Sync Server** (gRPC) |
| Where do kids chat? | **Nakama** (WebSocket) |
| Where are game saves? | **S3 via Sync Server** |
| Where are chat logs? | **Sync Server** (audit) |
| Where do alerts appear? | **Supabase** (real-time to parents) |
| What does the device cache locally? | Policies, blocklists, game saves |
| What requires internet? | Chat, alerts, game downloads |
| What works offline? | Screen time, DNS filtering, local games |

This architecture gives us:
- **Privacy:** Self-hosted where it matters
- **Performance:** Low latency for enforcement
- **Scalability:** Each tier scales independently
- **Cost:** Minimal cloud spend
- **Compliance:** Full audit trail for COPPA/GDPR
