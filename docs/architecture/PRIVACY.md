# Guardian Privacy & Data Retention Policy

## Core Principles

1. **Minimum Data** - Collect only what's needed for safety
2. **Local First** - Process on device when possible
3. **No Content Storage** - Log topics, not messages
4. **Time-Limited** - Auto-delete after retention period
5. **Transparent** - Parents and children know what's collected

## What We Collect vs. What We Don't

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Data Collection Policy                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ✅ WE COLLECT                    ❌ WE NEVER COLLECT               │
│  ─────────────────────────        ────────────────────────────      │
│                                                                     │
│  • Topic categories               • Message content (stored)        │
│    "Gaming 45%, School 30%"         "hey want to play minecraft"    │
│                                                                     │
│  • Contact patterns               • Real names                      │
│    "New contact, high frequency"    "John Smith"                    │
│                                                                     │
│  • Risk scores                    • Photos/media                    │
│    "0.72 - location questions"      (any images)                    │
│                                                                     │
│  • Domain queries                 • Full URLs/paths                 │
│    "youtube.com"                    "youtube.com/watch?v=..."       │
│                                                                     │
│  • Session timestamps             • Keystroke logs (stored)         │
│    "4:30pm - 5:15pm"                (analyzed in real-time only)    │
│                                                                     │
│  • Hashed identifiers             • Actual usernames                │
│    "a3f8b2c1..."                    "CoolGamer2011"                 │
│                                                                     │
│  • AI-generated summaries         • Verbatim quotes                 │
│    "Discussed gaming and school"    "he said '...'"                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Retention Schedule

| Data Type | Retention | Location | Encryption |
|-----------|-----------|----------|------------|
| Topic sessions | 90 days | Local | AES-256 |
| Risk history | 90 days | Local | AES-256 |
| Contact profiles | While active + 30 days | Local | AES-256 |
| DNS query logs | 30 days | Local | AES-256 |
| Screen captures | Never stored | RAM only | N/A |
| Keyboard input | Never stored | RAM only | N/A |
| Voice transcription | Never stored | RAM only | N/A |
| Conversation replay (HIGH) | 72 hours | Local | AES-256 |
| Conversation replay (CRITICAL) | 7 days | Local | AES-256 |
| Conversation replay (EXTREME) | 30 days | Local | AES-256 |
| Alert history | 1 year | Cloud (sync) | TLS + at rest |
| Parent actions | 1 year | Cloud (sync) | TLS + at rest |

## Automatic Data Cleanup

```rust
impl DataRetention {
    pub async fn cleanup(&self) {
        // Run daily at 3 AM
        
        // 1. Topic sessions older than 90 days
        self.db.execute(
            "DELETE FROM topic_sessions WHERE created_at < datetime('now', '-90 days')"
        ).await;
        
        // 2. Risk history older than 90 days
        self.db.execute(
            "DELETE FROM risk_history WHERE timestamp < datetime('now', '-90 days')"
        ).await;
        
        // 3. Inactive contact profiles
        self.db.execute(
            "DELETE FROM contacts WHERE last_seen < datetime('now', '-30 days')"
        ).await;
        
        // 4. DNS logs older than 30 days
        self.db.execute(
            "DELETE FROM dns_queries WHERE timestamp < datetime('now', '-30 days')"
        ).await;
        
        // 5. Expired conversation replays
        self.db.execute(
            "DELETE FROM conversation_replay WHERE expires_at < datetime('now')"
        ).await;
        
        // 6. Securely wipe deleted data
        self.db.execute("VACUUM").await;
        
        log::info!("Data cleanup completed");
    }
}
```

## Conversation Replay Policy

Replay is only captured when risk threshold is exceeded:

```rust
pub struct ReplayPolicy {
    // Capture thresholds
    pub high_risk_threshold: f32,      // 0.6 - capture for 72h
    pub critical_risk_threshold: f32,  // 0.85 - capture for 7d
    pub extreme_risk_threshold: f32,   // Grooming pattern - capture for 30d
}

impl ReplayPolicy {
    pub fn should_capture(&self, risk_score: f32, triggers: &[RiskTrigger]) -> CaptureDecision {
        // Always capture for extreme cases
        if triggers.contains(&RiskTrigger::GroomingPattern) ||
           triggers.contains(&RiskTrigger::SelfHarmIndicators) ||
           triggers.contains(&RiskTrigger::ExploitationDetected) {
            return CaptureDecision::Capture {
                retention: Duration::days(30),
                reason: "Safety-critical pattern detected",
                export_available: true,
            };
        }
        
        // Capture for critical risk
        if risk_score > self.critical_risk_threshold {
            return CaptureDecision::Capture {
                retention: Duration::days(7),
                reason: "Critical risk score",
                export_available: false,
            };
        }
        
        // Capture for high risk
        if risk_score > self.high_risk_threshold {
            return CaptureDecision::Capture {
                retention: Duration::hours(72),
                reason: "High risk score",
                export_available: false,
            };
        }
        
        // No capture needed
        CaptureDecision::DoNotCapture
    }
}
```

## Export Policy

Export is only available for extreme cases (potential legal action):

```rust
pub struct ExportPolicy;

impl ExportPolicy {
    pub fn can_export(&self, replay: &ConversationReplay) -> ExportDecision {
        match replay.trigger_reason {
            // Extreme cases only
            RiskTrigger::GroomingPattern |
            RiskTrigger::ExploitationDetected |
            RiskTrigger::ChildRequestedHelp => {
                ExportDecision::AvailableWithFriction {
                    requires_confirmation: true,
                    audit_logged: true,
                    expires_after: Duration::hours(72),
                    message: "This export is for reporting to authorities only",
                }
            }
            
            // All other cases - no export
            _ => ExportDecision::NotAvailable {
                reason: "Export only available for safety-critical cases",
            }
        }
    }
}
```

## Cloud Sync - What Goes Where

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Cloud Sync Policy                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  LOCAL ONLY (Never synced)        SYNCED (For multi-device)        │
│  ─────────────────────────        ────────────────────────────      │
│                                                                     │
│  • Message content                • Family configuration           │
│  • Full topic logs                • Child profiles (no PII)        │
│  • DNS query history              • Contact risk scores            │
│  • Conversation replays           • Alert history                  │
│  • Keystroke analysis             • Parent actions                 │
│  • Screen captures                • Approval requests              │
│  • Audio transcriptions           • Trust scores                   │
│                                                                     │
│  Why local only?                  Why synced?                       │
│  • Privacy protection             • Access from any device         │
│  • Legal compliance               • Both parents see alerts        │
│  • Data minimization              • Settings sync                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Encryption

### At Rest (Local)

```rust
// All local databases encrypted with AES-256-GCM
pub struct LocalStorage {
    encryption_key: [u8; 32],  // Derived from device key
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        // Key derived from TPM or secure enclave if available
        // Falls back to PBKDF2 from device-specific seed
        let key = derive_device_key()?;
        Ok(Self { encryption_key: key })
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce = generate_nonce();
        let cipher = Aes256Gcm::new(&self.encryption_key);
        cipher.encrypt(&nonce, data)
    }
}
```

### In Transit (Sync)

- All sync traffic over TLS 1.3
- Certificate pinning for Guardian servers
- Additional envelope encryption for sensitive data

## Child Transparency

Children should know they're being monitored (age-appropriate):

```rust
pub struct ChildTransparency {
    tier: AgeTier,
}

impl ChildTransparency {
    pub fn show_indicator(&self) -> IndicatorStyle {
        match self.tier {
            AgeTier::Tier1 => {
                // Under 10: Simple icon, no details
                IndicatorStyle::SimpleIcon {
                    tooltip: "Guardian is keeping you safe",
                }
            }
            AgeTier::Tier2 => {
                // 10-12: Icon with basic info
                IndicatorStyle::BasicInfo {
                    tooltip: "Guardian is monitoring for your safety",
                    shows_what: vec!["websites", "contacts"],
                }
            }
            AgeTier::Tier3 | AgeTier::Tier4 => {
                // 13+: Full transparency
                IndicatorStyle::Detailed {
                    shows_active_monitoring: true,
                    can_view_own_data: true,
                    explains_why: true,
                }
            }
        }
    }
}
```

### Teen Data Access (13+)

Teens can view their own monitoring data:

```
┌─────────────────────────────────────────────────────────────────────┐
│  Guardian - Your Privacy Dashboard                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  What Guardian sees:                                                │
│  ─────────────────────                                              │
│                                                                     │
│  📊 Topics (not messages)                                           │
│  We see: "Gaming 45%, School 30%, Social 25%"                       │
│  We don't see: Your actual messages                                 │
│                                                                     │
│  👥 Contacts (not names)                                            │
│  We see: "5 regular contacts, 1 new this week"                      │
│  We don't see: Their real usernames                                 │
│                                                                     │
│  🌐 Sites (domains only)                                            │
│  We see: "youtube.com, 47 visits"                                   │
│  We don't see: Which videos you watched                             │
│                                                                     │
│  ⚠️ When we capture more:                                          │
│  If something seems risky, we might temporarily save                │
│  the conversation so your parents can check you're safe.            │
│  This only happens when there's a real concern.                     │
│                                                                     │
│  [View My Data]  [Privacy Settings]                                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Legal Compliance

### GDPR (UK/EU)

- Right to access: Parents can export their family's data
- Right to deletion: Account deletion removes all data
- Data minimization: Only collect what's needed
- Purpose limitation: Data only used for child safety
- Storage limitation: Automatic retention enforcement

### Children's Code (UK ICO)

- Age-appropriate design
- Transparency for children
- Data minimization
- High privacy by default
- Parental controls where appropriate

### COPPA (US, if applicable)

- Parental consent for under-13
- Limited data collection
- Security requirements
- Deletion upon request

## Data Subject Rights

```rust
pub struct DataRights;

impl DataRights {
    // Parent can request all data for their family
    pub async fn export_family_data(&self, family_id: FamilyId) -> ExportPackage {
        ExportPackage {
            children: self.get_child_profiles(family_id).await,
            contacts: self.get_contact_summaries(family_id).await,
            alerts: self.get_alert_history(family_id).await,
            actions: self.get_parent_actions(family_id).await,
            // Note: Does NOT include message content
        }
    }
    
    // Parent can delete all family data
    pub async fn delete_family_data(&self, family_id: FamilyId) -> Result<()> {
        // 1. Delete local data on all devices
        self.broadcast_local_deletion(family_id).await?;
        
        // 2. Delete cloud sync data
        self.delete_sync_data(family_id).await?;
        
        // 3. Delete account
        self.delete_account(family_id).await?;
        
        Ok(())
    }
    
    // Child (13+) can view their own data
    pub async fn child_view_own_data(&self, child_id: ChildId) -> ChildDataView {
        ChildDataView {
            topic_summaries: self.get_topic_summaries(child_id).await,
            contact_count: self.get_contact_count(child_id).await,
            site_categories: self.get_browsing_categories(child_id).await,
            // Note: No raw data, just summaries
        }
    }
}
```

## Audit Logging

All sensitive operations are logged:

```rust
pub struct AuditLog {
    timestamp: DateTime,
    actor: Actor,  // Parent, System, or Child
    action: AuditAction,
    details: String,
}

enum AuditAction {
    // Parent actions
    ViewedReplay { child: ChildId, contact: ContactHash },
    ExportedData { reason: String },
    BlockedContact { child: ChildId, contact: ContactHash },
    AdjustedSettings { setting: String, old: String, new: String },
    
    // System actions
    CapturedReplay { child: ChildId, risk_score: f32 },
    SentAlert { tier: AlertTier, reason: String },
    DeletedExpiredData { count: u32 },
    
    // Data subject actions
    ExportedFamilyData,
    DeletedAccount,
    ChildViewedOwnData { child: ChildId },
}
```

## Related Documentation

- [Architecture Overview](./OVERVIEW.md)
- [Alert System](./ALERT_SYSTEM.md)
- [Contact Intelligence](./CONTACT_INTELLIGENCE.md)
