# Guardian Contact Intelligence System

## Overview

The Contact Intelligence system tracks WHO children communicate with, without storing personal data. It uses vector embeddings, behavioral patterns, and AI topic analysis to assess risk and build family trust networks.

## Core Principle: No PII Storage

```
What We Store              What We NEVER Store
─────────────────          ────────────────────
✅ Hashed contact ID       ❌ Actual usernames
✅ Conversation patterns   ❌ Message content
✅ Topic categories        ❌ Personal details
✅ Risk scores             ❌ Photos/media
✅ Inferred tags           ❌ Location data
✅ Family trust scores     ❌ Real names
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Contact Intelligence Engine                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Message Stream                                                     │
│       │                                                             │
│       ▼                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐        │
│  │   Topic     │───▶│   Contact   │───▶│  Family Trust   │        │
│  │  Analyzer   │    │   Profile   │    │    Network      │        │
│  └─────────────┘    └─────────────┘    └─────────────────┘        │
│       │                   │                     │                   │
│       ▼                   ▼                     ▼                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐        │
│  │ Topic Log   │    │ Risk Score  │    │ Shared Scores   │        │
│  │ (stored)    │    │ (computed)  │    │ (siblings)      │        │
│  └─────────────┘    └─────────────┘    └─────────────────┘        │
│                           │                                         │
│                           ▼                                         │
│                    ┌─────────────┐                                  │
│                    │   Alerts    │                                  │
│                    │ (if needed) │                                  │
│                    └─────────────┘                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Contact Profile Structure

```rust
struct ContactProfile {
    // Anonymous identifier (SHA256 of username+platform)
    id: ContactHash,
    
    // Vector embedding of conversation style
    style_embedding: Vec<f32>,  // 384-dim from sentence transformer
    
    // Inferred relationship tags
    tags: Vec<InferredTag>,
    
    // Behavioral patterns (no content)
    patterns: ConversationPatterns,
    
    // Risk assessment
    risk_score: f32,
    risk_factors: Vec<RiskFactor>,
    
    // Topic history
    topic_history: Vec<TopicSession>,
    
    // Family context
    family_interactions: HashMap<ChildId, ChildInteraction>,
}
```

## Inferred Tags

Tags are probabilistic inferences, not facts:

### Relationship Tags
| Tag | Signals | Confidence Threshold |
|-----|---------|---------------------|
| `LikelySchoolFriend` | Shared school context, after-school chats, long-term | 70% |
| `LikelyOnlineFriend` | Gaming topics, evening/weekend, met in-game | 60% |
| `LikelyFamily` | Mentioned relationship, any-time chats | 80% |
| `LikelySimilarAge` | Vocabulary match, topic overlap | 70% |
| `LikelyOlder` | Advanced vocabulary, different topics | 60% |

### Interest Tags
| Tag | Detection |
|-----|-----------|
| `SharedInterestGaming` | Gaming topics > 40% |
| `SharedInterestSports` | Sports topics > 30% |
| `SharedInterestMusic` | Music topics > 30% |
| `SharedInterestSchool` | School topics > 30% |

### Behavioral Tags
| Tag | Detection |
|-----|-----------|
| `KnownLongTerm` | First seen > 3 months ago |
| `RecentContact` | First seen < 2 weeks ago |
| `HighFrequency` | > 20 messages/day average |
| `LowFrequency` | < 5 messages/week |

### Risk Tags
| Tag | Detection | Impact |
|-----|-----------|--------|
| `AsksPersonalQuestions` | Location/age/school topics > 20% | +0.25 |
| `SuggestsPrivatePlatform` | Platform switch detected | +0.30 |
| `LateNightChatter` | > 30% messages after 10pm | +0.15 |
| `IntenseMessaging` | New contact + high frequency | +0.35 |
| `AgeInappropriateLanguage` | Vocabulary analysis | +0.20 |

## Risk Scoring Algorithm

```rust
impl ContactProfile {
    pub fn calculate_risk(&mut self) {
        let mut risk = 0.0;
        self.risk_factors.clear();
        
        // ═══════════════════════════════════════════
        // POSITIVE factors (reduce risk)
        // ═══════════════════════════════════════════
        
        if self.has_tag(LikelySchoolFriend, 0.7) {
            risk -= 0.20;
        }
        
        if self.has_tag(LikelySimilarAge, 0.7) {
            risk -= 0.10;
        }
        
        if self.has_tag(KnownLongTerm) {
            risk -= 0.15;
        }
        
        if self.patterns.mutual_friends > 2 {
            risk -= 0.10;
        }
        
        // Family trust boost
        if let Some(family_trust) = self.family_trust_score {
            risk -= family_trust * 0.20;
        }
        
        // ═══════════════════════════════════════════
        // NEGATIVE factors (increase risk)
        // ═══════════════════════════════════════════
        
        if self.has_tag(AsksPersonalQuestions) {
            risk += 0.25;
            self.risk_factors.push(RiskFactor::AsksPersonalInfo);
        }
        
        if self.has_tag(SuggestsPrivatePlatform) {
            risk += 0.30;
            self.risk_factors.push(RiskFactor::WantsPrivatePlatform);
        }
        
        if self.has_tag(LikelyOlder, 0.6) {
            risk += 0.20;
            self.risk_factors.push(RiskFactor::PossiblyOlder);
        }
        
        if self.has_tag(LateNightChatter) {
            risk += 0.15;
            self.risk_factors.push(RiskFactor::LateNightContact);
        }
        
        // New contact with intense messaging
        if self.has_tag(RecentContact) && self.patterns.messages_per_day > 20.0 {
            risk += 0.35;
            self.risk_factors.push(RiskFactor::IntenseNewContact);
        }
        
        // High personal topic ratio
        if self.topic_history.personal_ratio() > 0.4 {
            risk += 0.20;
            self.risk_factors.push(RiskFactor::HighlyPersonalConversation);
        }
        
        // Grooming pattern detection
        if self.detect_grooming_escalation() {
            risk += 0.50;
            self.risk_factors.push(RiskFactor::GroomingPattern);
        }
        
        self.risk_score = risk.clamp(0.0, 1.0);
    }
}
```

## Topic Analysis

### Topic Categories

```rust
// SAFE TOPICS (reduce risk)
const SAFE_TOPICS: &[&str] = &[
    "gaming",      // Minecraft, Roblox, Fortnite
    "school",      // Homework, teachers, classes
    "sports",      // Football, activities
    "music",       // Songs, artists
    "movies",      // Films, shows
    "pets",        // Animals
    "food",        // Eating, cooking
    "family",      // Parents, siblings (neutral)
];

// CAUTION TOPICS (monitor)
const CAUTION_TOPICS: &[&str] = &[
    "personal",    // About themselves
    "appearance",  // Looks, body
    "secrets",     // Private things
    "compliments", // Flattery (grooming signal)
    "age",         // How old they are
    "politics",    // Political discussion
    "religion",    // Religious topics
];

// HIGH RISK TOPICS (alert)
const HIGH_RISK_TOPICS: &[&str] = &[
    "location",        // Where they live/are
    "meeting_up",      // Plans to meet
    "private_platform", // Moving to WhatsApp etc
    "gifts_money",     // Offering things
    "photos_request",  // Asking for pictures
    "keep_secret",     // Don't tell parents
    "relationship",    // Romantic interest
];

// CRITICAL TOPICS (immediate alert)
const CRITICAL_TOPICS: &[&str] = &[
    "sexual",       // Sexual content
    "drugs",        // Drug references
    "self_harm",    // Self-harm discussion
    "violence",     // Violent content
    "bullying",     // Being bullied
    "exploitation", // Grooming/exploitation
];
```

### Topic Session Storage

```rust
struct TopicSession {
    contact_hash: ContactHash,
    session_start: DateTime,
    session_end: DateTime,
    
    // Topic breakdown (not content!)
    topics: Vec<TopicEntry>,
    
    // Risk events during session
    risk_events: Vec<RiskEvent>,
    
    // Message counts only
    message_count: u32,
    child_message_count: u32,
    contact_message_count: u32,
}

struct TopicEntry {
    topic: String,          // "gaming"
    percentage: f32,        // 0.45
    description: String,    // "Discussed Minecraft builds"
    risk_level: RiskLevel,  // Safe/Caution/High/Critical
}
```

## Family Trust Network

```rust
struct FamilyTrustNetwork {
    family_id: FamilyId,
    children: Vec<ChildId>,
    shared_contacts: HashMap<ContactHash, FamilyContactProfile>,
}

struct FamilyContactProfile {
    contact_hash: ContactHash,
    
    // Combined scores from all children
    family_trust_score: f32,
    family_risk_score: f32,
    
    // Which children know this contact
    known_by: Vec<ChildId>,
    
    // Combined topic history
    family_topic_history: Vec<TopicEntry>,
}

impl FamilyTrustNetwork {
    pub fn calculate_family_scores(&mut self) {
        for (_, profile) in &mut self.shared_contacts {
            let interactions: Vec<_> = profile.known_by.iter()
                .filter_map(|child| self.get_child_interaction(child, &profile.contact_hash))
                .collect();
            
            // If older sibling trusts contact, boost trust for younger
            let oldest_trust = interactions.iter()
                .filter(|i| i.child_age >= 13)
                .map(|i| i.trust_score)
                .max_by(|a, b| a.partial_cmp(b).unwrap());
            
            if let Some(trust) = oldest_trust {
                // 80% trust transfer to younger siblings
                profile.family_trust_score = trust * 0.8;
            }
            
            // But risk flags from ANY child apply to ALL
            profile.family_risk_score = interactions.iter()
                .map(|i| i.risk_score)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
        }
    }
}
```

## Grooming Pattern Detection

Classic grooming follows predictable stages:

```
Week 1: Trust Building
├── Gaming, shared interests
├── Friendly, age-appropriate
└── Signal: High safe topics, new contact

Week 2: Boundary Testing  
├── Personal questions start
├── Compliments increase
└── Signal: Personal topics rise 20%+

Week 3: Isolation
├── "Keep this between us"
├── Special relationship language
└── Signal: Secrecy topics appear

Week 4: Exploitation
├── Location questions
├── Photo requests
├── Meeting suggestions
└── Signal: HIGH/CRITICAL topics
```

```rust
impl ContactProfile {
    pub fn detect_grooming_escalation(&self) -> bool {
        let history = self.get_weekly_topic_trends();
        
        // Need at least 2 weeks of data
        if history.len() < 2 {
            return false;
        }
        
        let mut stage = 0;
        
        for week in &history {
            match stage {
                0 => {
                    // Stage 1: Trust building (safe topics, new contact)
                    if week.safe_ratio > 0.7 && self.is_recent() {
                        stage = 1;
                    }
                }
                1 => {
                    // Stage 2: Boundary testing (personal increase)
                    if week.personal_ratio > week.prev_personal_ratio + 0.15 {
                        stage = 2;
                    }
                }
                2 => {
                    // Stage 3: Isolation (secrecy appears)
                    if week.has_topic("secrets") || week.has_topic("keep_secret") {
                        stage = 3;
                    }
                }
                3 => {
                    // Stage 4: Exploitation (high-risk topics)
                    if week.has_any_topic(&["location", "photos_request", "meeting_up"]) {
                        return true; // GROOMING PATTERN CONFIRMED
                    }
                }
                _ => {}
            }
        }
        
        false
    }
}
```

## Conversation Replay (High Risk Only)

```rust
struct ConversationReplay {
    // Only captured when risk > 0.6
    trigger_reason: RiskTrigger,
    captured_at: DateTime,
    
    // Auto-expiry
    expires_at: DateTime,  // 72h for HIGH, 7d for CRITICAL
    
    // Full messages with highlights
    messages: Vec<ReplayMessage>,
    
    // Access tracking
    parent_viewed: bool,
    action_taken: Option<ParentAction>,
}

struct ReplayMessage {
    timestamp: DateTime,
    direction: Direction,  // ChildSent / ContactSent
    content: String,       // Full text
    risk_phrases: Vec<Highlight>,
}

// Retention rules
impl ConversationReplay {
    pub fn retention_period(risk_level: RiskLevel) -> Duration {
        match risk_level {
            RiskLevel::High => Duration::hours(72),
            RiskLevel::Critical => Duration::days(7),
            RiskLevel::Extreme => Duration::days(30), // For authorities
            _ => Duration::zero(), // No capture
        }
    }
    
    pub fn should_capture(risk_score: f32, topics: &[TopicEntry]) -> bool {
        risk_score > 0.6 ||
        topics.iter().any(|t| t.risk_level >= RiskLevel::Critical)
    }
}
```

## Alert Tiers

| Risk Score | Tier | Notification | Replay | Export |
|------------|------|--------------|--------|--------|
| < 0.3 | LOW | Weekly digest | ❌ | ❌ |
| 0.3 - 0.6 | MEDIUM | Daily digest + note | ❌ | ❌ |
| 0.6 - 0.8 | HIGH | Real-time push | 72 hours | ❌ |
| > 0.8 | CRITICAL | Immediate + sound | 7 days | With friction |
| Grooming | EXTREME | Emergency escalation | 30 days | For authorities |

## Data Retention

| Data Type | Retention | Encryption |
|-----------|-----------|------------|
| Topic sessions | 90 days | At rest |
| Risk history | 90 days | At rest |
| Contact profiles | While active + 30 days | At rest |
| Conversation replay | Per risk tier | At rest + in transit |
| Vector embeddings | While active | At rest |

## Parent Dashboard Integration

Parents see:
- Contact list with tags and risk scores
- Topic breakdown per conversation
- Trend analysis (weekly changes)
- Risk alerts with context
- Friend group detection
- Conversation replay (when unlocked)

Parents DON'T see:
- Actual messages (except high-risk replay)
- Real contact names
- Personal information

## Related Documentation

- [Topic Analysis Details](./TOPIC_ANALYSIS.md)
- [Alert System](./ALERT_SYSTEM.md)
- [Privacy & Data Retention](./PRIVACY.md)
