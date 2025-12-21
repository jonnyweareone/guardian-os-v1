# Guardian Age-Tiered Protection Model

## Overview

A 9-year-old and a 15-year-old need fundamentally different approaches to online safety. Guardian OS adapts protection based on age, building trust as children demonstrate responsible behavior.

## Age Tiers

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Age-Tiered Protection                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 1: Under 10                                                   │
│  ═══════════════════════════════════════════════════════════════   │
│  Philosophy: Maximum protection, zero autonomy                      │
│                                                                     │
│  • PII: Block ALL personal information                              │
│  • Apps: Whitelist only (parent-approved)                           │
│  • Web: Kids sites only                                             │
│  • Forms: Block all signups                                         │
│  • Monitoring: Full capture, immediate alerts                       │
│  • Replay: Always available to parents                              │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 2: Ages 10-12                                                 │
│  ═══════════════════════════════════════════════════════════════   │
│  Philosophy: High protection with "Ask Parent" option               │
│                                                                     │
│  • PII: Block critical (address, school), warn on others            │
│  • Apps: Approved list (can request new)                            │
│  • Web: Filtered with explanations                                  │
│  • Forms: Warn and notify parent                                    │
│  • Monitoring: Full + real-time alerts                              │
│  • Replay: On elevated risk                                         │
│  • NEW: "Ask Parent" button for blocked content                     │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 3: Ages 13-15                                                 │
│  ═══════════════════════════════════════════════════════════════   │
│  Philosophy: Moderate protection, building trust                    │
│                                                                     │
│  • PII: Warn only (except critical like address)                    │
│  • Apps: All allowed with monitoring                                │
│  • Web: Filter adult content only                                   │
│  • Forms: Log signups, notify parent                                │
│  • Monitoring: Pattern detection (grooming, bullying)               │
│  • Replay: On high risk only                                        │
│  • NEW: Can dismiss warnings (logged for parent review)             │
│  • NEW: Daily digest instead of real-time alerts                    │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 4: Ages 16-17                                                 │
│  ═══════════════════════════════════════════════════════════════   │
│  Philosophy: Light touch, trust-based                               │
│                                                                     │
│  • PII: Log only (no warnings)                                      │
│  • Apps: All allowed, minimal monitoring                            │
│  • Web: Block illegal content only                                  │
│  • Forms: Log only                                                  │
│  • Monitoring: Safety-critical only (self-harm, exploitation)       │
│  • Replay: Emergency only                                           │
│  • NEW: Weekly digest                                               │
│  • NEW: Can disable most monitoring (with parent awareness)         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Feature Matrix by Age

### PII Protection

| PII Type | Under 10 | 10-12 | 13-15 | 16-17 |
|----------|----------|-------|-------|-------|
| Home address | 🚫 Block | 🚫 Block | 🚫 Block | ⚠️ Warn |
| School name | 🚫 Block | 🚫 Block | ⚠️ Warn | 📝 Log |
| Phone number | 🚫 Block | 🚫 Block | ⚠️ Warn | 📝 Log |
| Full name | 🚫 Block | ⚠️ Warn | 📝 Log | - |
| Age | 🚫 Block | ⚠️ Warn | 📝 Log | - |
| Location (city) | 🚫 Block | ⚠️ Warn | 📝 Log | - |
| "Home alone" | 🚫 Block | 🚫 Block | ⚠️ Warn | 📝 Log |

### App & Web Access

| Category | Under 10 | 10-12 | 13-15 | 16-17 |
|----------|----------|-------|-------|-------|
| Kids games | ✅ Allow | ✅ Allow | ✅ Allow | ✅ Allow |
| General games | ⚠️ Approved | ✅ Allow | ✅ Allow | ✅ Allow |
| YouTube | 🔒 Kids only | ⚠️ Restricted | ✅ Moderate | ✅ Allow |
| Social media | 🚫 Block | ⚠️ Approved | ✅ Monitored | ✅ Allow |
| Messaging apps | 🚫 Block | ⚠️ Approved | ✅ Monitored | ✅ Allow |
| Adult content | 🚫 Block | 🚫 Block | 🚫 Block | 🚫 Block |
| Gambling | 🚫 Block | 🚫 Block | 🚫 Block | 🚫 Block |
| VPNs/Proxies | 🚫 Block | 🚫 Block | 🚫 Block | ⚠️ Warn |

### Monitoring Level

| Feature | Under 10 | 10-12 | 13-15 | 16-17 |
|---------|----------|-------|-------|-------|
| Keyboard capture | ✅ Full | ✅ Full | ⚠️ Alerts only | ❌ Off |
| Screen scanning | ✅ Full | ✅ Full | ⚠️ Sampling | ❌ Off |
| Topic logging | ✅ Full | ✅ Full | ✅ Full | ⚠️ Summary |
| Contact tracking | ✅ Full | ✅ Full | ✅ Full | ⚠️ Risk only |
| Browsing logs | ✅ Full | ✅ Full | ⚠️ Categories | 📝 Domains |

### Parent Alerts

| Alert Type | Under 10 | 10-12 | 13-15 | 16-17 |
|------------|----------|-------|-------|-------|
| New contact | Immediate | Immediate | Daily digest | Weekly |
| Risk spike | Immediate | Immediate | Real-time | Real-time |
| Blocked site | Immediate | Daily digest | Weekly | - |
| PII warning | Immediate | Immediate | Daily digest | - |
| VPN attempt | Immediate | Immediate | Real-time | Notify |
| Grooming pattern | EMERGENCY | EMERGENCY | EMERGENCY | EMERGENCY |
| Self-harm | EMERGENCY | EMERGENCY | EMERGENCY | EMERGENCY |

## Implementation

```rust
#[derive(Debug, Clone, Copy)]
pub enum AgeTier {
    Tier1,  // Under 10
    Tier2,  // 10-12
    Tier3,  // 13-15
    Tier4,  // 16-17
}

impl AgeTier {
    pub fn from_age(age: u8) -> Self {
        match age {
            0..=9 => AgeTier::Tier1,
            10..=12 => AgeTier::Tier2,
            13..=15 => AgeTier::Tier3,
            16..=17 => AgeTier::Tier4,
            _ => AgeTier::Tier4, // 18+ shouldn't use child mode
        }
    }
}

pub struct TierPolicy {
    pub tier: AgeTier,
    
    // PII handling
    pub pii_policy: PiiPolicy,
    
    // App/web access
    pub app_policy: AppPolicy,
    
    // Monitoring level
    pub monitoring_policy: MonitoringPolicy,
    
    // Alert settings
    pub alert_policy: AlertPolicy,
    
    // Autonomy features
    pub autonomy: AutonomyPolicy,
}

impl TierPolicy {
    pub fn for_tier(tier: AgeTier) -> Self {
        match tier {
            AgeTier::Tier1 => Self::tier1_policy(),
            AgeTier::Tier2 => Self::tier2_policy(),
            AgeTier::Tier3 => Self::tier3_policy(),
            AgeTier::Tier4 => Self::tier4_policy(),
        }
    }
    
    fn tier1_policy() -> Self {
        Self {
            tier: AgeTier::Tier1,
            pii_policy: PiiPolicy {
                address: PiiAction::Block,
                school: PiiAction::Block,
                phone: PiiAction::Block,
                name: PiiAction::Block,
                age: PiiAction::Block,
                location: PiiAction::Block,
                home_alone: PiiAction::Block,
            },
            app_policy: AppPolicy {
                mode: AppMode::Whitelist,
                social_media: AppAccess::Blocked,
                messaging: AppAccess::Blocked,
                youtube: YoutubeMode::KidsOnly,
                games: AppAccess::Approved,
            },
            monitoring_policy: MonitoringPolicy {
                keyboard: MonitorLevel::Full,
                screen: MonitorLevel::Full,
                topics: MonitorLevel::Full,
                contacts: MonitorLevel::Full,
                browsing: MonitorLevel::Full,
            },
            alert_policy: AlertPolicy {
                new_contact: AlertFrequency::Immediate,
                risk_spike: AlertFrequency::Immediate,
                blocked_site: AlertFrequency::Immediate,
                pii_warning: AlertFrequency::Immediate,
                digest: DigestFrequency::Daily,
            },
            autonomy: AutonomyPolicy {
                can_dismiss_warnings: false,
                can_request_approval: false,
                can_override_blocks: false,
            },
        }
    }
    
    fn tier2_policy() -> Self {
        Self {
            tier: AgeTier::Tier2,
            pii_policy: PiiPolicy {
                address: PiiAction::Block,
                school: PiiAction::Block,
                phone: PiiAction::Block,
                name: PiiAction::Warn,
                age: PiiAction::Warn,
                location: PiiAction::Warn,
                home_alone: PiiAction::Block,
            },
            app_policy: AppPolicy {
                mode: AppMode::Approved,
                social_media: AppAccess::Approved,
                messaging: AppAccess::Approved,
                youtube: YoutubeMode::Restricted,
                games: AppAccess::Allowed,
            },
            monitoring_policy: MonitoringPolicy {
                keyboard: MonitorLevel::Full,
                screen: MonitorLevel::Full,
                topics: MonitorLevel::Full,
                contacts: MonitorLevel::Full,
                browsing: MonitorLevel::Full,
            },
            alert_policy: AlertPolicy {
                new_contact: AlertFrequency::Immediate,
                risk_spike: AlertFrequency::Immediate,
                blocked_site: AlertFrequency::Daily,
                pii_warning: AlertFrequency::Immediate,
                digest: DigestFrequency::Daily,
            },
            autonomy: AutonomyPolicy {
                can_dismiss_warnings: false,
                can_request_approval: true,  // "Ask Parent" button
                can_override_blocks: false,
            },
        }
    }
    
    fn tier3_policy() -> Self {
        Self {
            tier: AgeTier::Tier3,
            pii_policy: PiiPolicy {
                address: PiiAction::Block,  // Always block address
                school: PiiAction::Warn,
                phone: PiiAction::Warn,
                name: PiiAction::Log,
                age: PiiAction::Log,
                location: PiiAction::Log,
                home_alone: PiiAction::Warn,
            },
            app_policy: AppPolicy {
                mode: AppMode::Monitored,
                social_media: AppAccess::Allowed,
                messaging: AppAccess::Allowed,
                youtube: YoutubeMode::Moderate,
                games: AppAccess::Allowed,
            },
            monitoring_policy: MonitoringPolicy {
                keyboard: MonitorLevel::AlertsOnly,
                screen: MonitorLevel::Sampling,
                topics: MonitorLevel::Full,
                contacts: MonitorLevel::Full,
                browsing: MonitorLevel::Categories,
            },
            alert_policy: AlertPolicy {
                new_contact: AlertFrequency::Daily,
                risk_spike: AlertFrequency::Immediate,
                blocked_site: AlertFrequency::Weekly,
                pii_warning: AlertFrequency::Daily,
                digest: DigestFrequency::Daily,
            },
            autonomy: AutonomyPolicy {
                can_dismiss_warnings: true,  // Logged for parent
                can_request_approval: true,
                can_override_blocks: false,
            },
        }
    }
    
    fn tier4_policy() -> Self {
        Self {
            tier: AgeTier::Tier4,
            pii_policy: PiiPolicy {
                address: PiiAction::Warn,
                school: PiiAction::Log,
                phone: PiiAction::Log,
                name: PiiAction::None,
                age: PiiAction::None,
                location: PiiAction::None,
                home_alone: PiiAction::Log,
            },
            app_policy: AppPolicy {
                mode: AppMode::Open,
                social_media: AppAccess::Allowed,
                messaging: AppAccess::Allowed,
                youtube: YoutubeMode::Open,
                games: AppAccess::Allowed,
            },
            monitoring_policy: MonitoringPolicy {
                keyboard: MonitorLevel::Off,
                screen: MonitorLevel::Off,
                topics: MonitorLevel::Summary,
                contacts: MonitorLevel::RiskOnly,
                browsing: MonitorLevel::DomainsOnly,
            },
            alert_policy: AlertPolicy {
                new_contact: AlertFrequency::Weekly,
                risk_spike: AlertFrequency::Immediate,
                blocked_site: AlertFrequency::None,
                pii_warning: AlertFrequency::None,
                digest: DigestFrequency::Weekly,
            },
            autonomy: AutonomyPolicy {
                can_dismiss_warnings: true,
                can_request_approval: true,
                can_override_blocks: true,  // With logging
            },
        }
    }
}
```

## Trust Score Integration

Children can earn more freedom through demonstrated responsible behavior:

```rust
pub struct TrustScore {
    score: f32,  // 0.0 - 1.0
    history: Vec<TrustEvent>,
}

impl TrustScore {
    // Positive events (increase trust)
    pub fn positive_event(&mut self, event: PositiveEvent) {
        match event {
            // Regular safe behavior
            PositiveEvent::WeekWithoutIncident => self.score += 0.02,
            
            // Followed guidance
            PositiveEvent::HeededWarning => self.score += 0.01,
            
            // Self-reported concern
            PositiveEvent::ReportedIssue => self.score += 0.05,
            
            // Completed safety education
            PositiveEvent::CompletedSafetyModule => self.score += 0.03,
        }
        self.score = self.score.min(1.0);
    }
    
    // Negative events (decrease trust)
    pub fn negative_event(&mut self, event: NegativeEvent) {
        match event {
            // Minor: dismissed warnings repeatedly
            NegativeEvent::DismissedWarnings(count) => {
                self.score -= 0.01 * count as f32;
            }
            
            // Moderate: tried to bypass
            NegativeEvent::BypassAttempt => self.score -= 0.10,
            
            // Serious: shared critical PII
            NegativeEvent::SharedCriticalPii => self.score -= 0.15,
            
            // Severe: met stranger
            NegativeEvent::UnsafeContact => self.score -= 0.30,
        }
        self.score = self.score.max(0.0);
    }
    
    // Trust affects tier features
    pub fn adjust_tier_features(&self, policy: &mut TierPolicy) {
        if self.score > 0.8 {
            // High trust: relax some restrictions
            policy.monitoring_policy.screen = MonitorLevel::Sampling;
        } else if self.score < 0.3 {
            // Low trust: tighten restrictions
            policy.autonomy.can_dismiss_warnings = false;
            policy.monitoring_policy.keyboard = MonitorLevel::Full;
        }
    }
}
```

## "Ask Parent" Feature (Tier 2+)

```rust
pub struct ApprovalRequest {
    id: RequestId,
    child: ChildId,
    
    // What they want
    request_type: RequestType,
    details: String,
    
    // Context
    requested_at: DateTime,
    expires_at: DateTime,
    
    // Parent response
    status: RequestStatus,
    responded_by: Option<ParentId>,
    response_note: Option<String>,
}

enum RequestType {
    // Website access
    UnblockSite { domain: String, reason: String },
    
    // App access
    AllowApp { app_name: String, reason: String },
    
    // Contact approval
    ApproveContact { contact_hash: ContactHash, context: String },
    
    // Time extension
    ExtendTime { minutes: u32, reason: String },
    
    // Content access
    ViewContent { content_type: String, reason: String },
}

// Child sees this when blocked:
// ┌─────────────────────────────────────────┐
// │  This site is blocked                   │
// │                                         │
// │  gaming-site.com isn't on your          │
// │  approved list.                         │
// │                                         │
// │  [Ask Parent for Permission]            │
// │                                         │
// │  Want to explain why? (optional)        │
// │  ┌─────────────────────────────────┐   │
// │  │ For school project              │   │
// │  └─────────────────────────────────┘   │
// │                                         │
// │  [Send Request]  [Go Back]              │
// └─────────────────────────────────────────┘

// Parent sees push notification:
// ┌─────────────────────────────────────────┐
// │  📱 Tommy is asking for permission      │
// │                                         │
// │  Site: gaming-site.com                  │
// │  Reason: "For school project"           │
// │                                         │
// │  [Allow]  [Allow Once]  [Deny]          │
// └─────────────────────────────────────────┘
```

## Warning Dismissal Logging (Tier 3+)

```rust
pub struct DismissedWarning {
    warning_type: WarningType,
    context: String,
    dismissed_at: DateTime,
    
    // What was the warning about
    pii_type: Option<PiiType>,
    contact: Option<ContactHash>,
    site: Option<String>,
    
    // Shown in parent digest
    summary: String,
}

// Parent digest shows:
// ┌─────────────────────────────────────────┐
// │  Dismissed Warnings This Week           │
// │                                         │
// │  • Tommy dismissed 2 age warnings when  │
// │    chatting with online contacts        │
// │  • Dismissed 1 school name warning      │
// │                                         │
// │  All contacts are low risk. No action   │
// │  needed.                                │
// └─────────────────────────────────────────┘
```

## Automatic Tier Transitions

```rust
impl ChildProfile {
    pub fn check_tier_transition(&mut self) {
        let current_age = self.calculate_age();
        let new_tier = AgeTier::from_age(current_age);
        
        if new_tier != self.tier {
            // Tier is changing!
            self.handle_tier_transition(new_tier);
        }
    }
    
    fn handle_tier_transition(&mut self, new_tier: AgeTier) {
        // Notify parents of upcoming change
        self.send_tier_change_notification(new_tier);
        
        // Schedule transition (give parents time to adjust)
        self.scheduled_transition = Some(ScheduledTransition {
            new_tier,
            effective_date: now() + Duration::days(7),
            parent_acknowledged: false,
        });
        
        // Parent must acknowledge before transition
        // This prevents surprise changes
    }
}

// Parent notification:
// ┌─────────────────────────────────────────┐
// │  🎂 Tommy is turning 13!                │
// │                                         │
// │  His protection settings will change    │
// │  in 7 days:                             │
// │                                         │
// │  • Social media: Blocked → Monitored    │
// │  • Warnings: Cannot dismiss → Can       │
// │  • Alerts: Immediate → Daily digest     │
// │                                         │
// │  You can adjust these in Settings.      │
// │                                         │
// │  [Review Changes]  [Keep Current]       │
// └─────────────────────────────────────────┘
```

## Custom Overrides

Parents can adjust tier settings:

```toml
# /etc/guardian/children/tommy.toml

[profile]
name = "Tommy"
date_of_birth = "2014-03-15"
tier = "auto"  # or "tier1", "tier2", etc.

[overrides]
# Keep social media blocked even in Tier 3
social_media = "blocked"

# Allow more monitoring than tier default
monitoring_level = "full"

# Custom alert frequency
new_contact_alert = "immediate"

[trusted_contacts]
# Pre-approved contacts (skip approval workflow)
trusted = ["cousin_123abc", "grandma_456def"]

[schedule]
# Time-based restrictions
school_days = ["monday", "tuesday", "wednesday", "thursday", "friday"]
school_hours = "08:00-15:30"
bedtime = "21:00"

# During school hours: stricter
[schedule.school_hours]
social_media = "blocked"
games = "blocked"

# After bedtime: device locks
[schedule.bedtime]
device_locked = true
```

## Related Documentation

- [Contact Intelligence](./CONTACT_INTELLIGENCE.md)
- [Alert System](./ALERT_SYSTEM.md)
- [Privacy & Data Retention](./PRIVACY.md)
