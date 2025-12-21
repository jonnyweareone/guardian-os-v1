# Guardian Alert System

## Overview

The Guardian Alert System delivers contextual notifications to parents based on risk level. It's designed to inform without overwhelming - parents should feel confident, not anxious.

## Alert Philosophy

1. **Don't cry wolf** - Only alert when it matters
2. **Provide context** - Tell parents WHY, not just WHAT
3. **Enable action** - Give clear next steps
4. **Respect privacy** - Minimum data for informed decisions

## Alert Tiers

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Alert Tiers                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  📊 DIGEST (Risk < 0.3)                                            │
│  ────────────────────────                                           │
│  • Weekly summary (13+ children)                                    │
│  • Daily summary (<13 children)                                     │
│  • Topics, contacts, screen time                                    │
│  • "All clear" or gentle notes                                      │
│  • No notification - check at leisure                               │
│                                                                     │
│  📝 NOTE (Risk 0.3 - 0.5)                                          │
│  ────────────────────────                                           │
│  • Included in next digest                                          │
│  • "Something to be aware of"                                       │
│  • No immediate action needed                                       │
│  • Example: "New contact made"                                      │
│                                                                     │
│  ⚠️ ELEVATED (Risk 0.5 - 0.7)                                      │
│  ────────────────────────                                           │
│  • Push notification (not urgent)                                   │
│  • "Review when convenient"                                         │
│  • No replay unlocked                                               │
│  • Example: "Contact asking personal questions"                     │
│                                                                     │
│  🔴 HIGH (Risk 0.7 - 0.85)                                         │
│  ────────────────────────                                           │
│  • Immediate push notification                                      │
│  • Conversation replay unlocked (72 hours)                          │
│  • "Review recommended"                                             │
│  • Example: "Multiple risk signals detected"                        │
│                                                                     │
│  🚨 CRITICAL (Risk > 0.85)                                         │
│  ────────────────────────                                           │
│  • Immediate push + sound                                           │
│  • Conversation replay unlocked (7 days)                            │
│  • "Action required"                                                │
│  • Example: "Grooming pattern detected"                             │
│                                                                     │
│  🆘 EMERGENCY                                                       │
│  ────────────────────────                                           │
│  • All contact methods attempted                                    │
│  • Escalation to secondary contacts                                 │
│  • Conversation replay unlocked (30 days)                           │
│  • Export available for authorities                                 │
│  • Example: "Child safety crisis"                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Alert Structure

```rust
struct Alert {
    id: AlertId,
    tier: AlertTier,
    child: ChildId,
    
    // What triggered it
    trigger: AlertTrigger,
    risk_score: f32,
    
    // Context
    contact: Option<ContactHash>,
    topics: Vec<String>,
    risk_factors: Vec<RiskFactor>,
    
    // AI-generated summary
    summary: String,
    recommended_action: String,
    
    // Timing
    created_at: DateTime,
    expires_at: Option<DateTime>,
    
    // Status
    acknowledged: bool,
    action_taken: Option<ParentAction>,
    
    // Replay access (if unlocked)
    replay_available: bool,
    replay_expires: Option<DateTime>,
}

enum AlertTrigger {
    // Contact-based
    NewHighRiskContact,
    RiskScoreSpike { from: f32, to: f32 },
    GroomingPatternDetected,
    
    // Topic-based
    CriticalTopicDetected { topic: String },
    PersonalInfoShared,
    
    // Behavioral
    LateNightActivity,
    VpnBypassAttempt,
    BlockedSiteAttempt { domain: String },
    
    // Safety
    SelfHarmIndicators,
    BullyingDetected,
    
    // Child-initiated
    PanicButtonPressed,
}
```

## Weekly Digest Format

```
┌─────────────────────────────────────────────────────────────────────┐
│  Guardian Weekly Digest - Tommy                                      │
│  Week of Dec 15-21, 2024                                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  📊 Activity Summary                                                │
│  ─────────────────────                                              │
│  Screen time: 14h 32m (▼ 2h from last week)                        │
│  Conversations: 23 across 6 contacts                                │
│  Sites visited: 47 unique domains                                   │
│                                                                     │
│  🎯 Top Topics                                                      │
│  ─────────────────────                                              │
│  🎮 Gaming: 48% (Minecraft, Roblox)                                │
│  🏫 School: 28% (homework help, class chat)                        │
│  📺 Video: 15% (YouTube)                                           │
│  💬 Social: 9% (friends)                                           │
│                                                                     │
│  👥 Contact Health                                                  │
│  ─────────────────────                                              │
│  ✅ 5 contacts in good standing                                     │
│  📝 1 new contact (CoolDude99) - monitoring                         │
│                                                                     │
│  🌐 Browsing Highlights                                             │
│  ─────────────────────                                              │
│  Top sites: youtube.com, minecraft.net, roblox.com                  │
│  Safe search: Active ✅                                             │
│  Blocked attempts: 2 (adult content, VPN site)                      │
│                                                                     │
│  ✅ Overall: Normal activity, no concerns                           │
│                                                                     │
│  [View Details]  [Adjust Settings]                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Real-Time Alert Format

### Elevated Alert (Push)
```
┌─────────────────────────────────────────────────────────────────────┐
│  Guardian                                              3:45 PM 📱   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  📝 Note about Tommy's activity                                     │
│                                                                     │
│  A new contact (CoolDude99) has been asking about                   │
│  Tommy's school and age. Currently low risk, but                    │
│  we're keeping an eye on it.                                        │
│                                                                     │
│  [View Contact]  [Dismiss]                                          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### High Alert (Push + Sound)
```
┌─────────────────────────────────────────────────────────────────────┐
│  🔔 Guardian Alert                                     11:45 PM 📱  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ⚠️ Concerning conversation detected                                │
│                                                                     │
│  Tommy is chatting with xX_Shadow_Xx                                │
│  Risk Score: 0.72 (HIGH)                                            │
│                                                                     │
│  Flags detected:                                                    │
│  • Asked for location (3 times)                                     │
│  • Suggested moving to WhatsApp                                     │
│  • Late night conversation                                          │
│                                                                     │
│  Conversation replay available for 72 hours.                        │
│                                                                     │
│  [Review Now]  [Block Contact]  [Remind Later]                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Critical Alert (Immediate)
```
┌─────────────────────────────────────────────────────────────────────┐
│  🚨 GUARDIAN EMERGENCY                                 11:52 PM 📱  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  🔴 Grooming pattern confirmed                                      │
│                                                                     │
│  Contact xX_Shadow_Xx has exhibited classic                         │
│  grooming behavior with Tommy over 3 weeks:                         │
│                                                                     │
│  ✓ Built trust through gaming                                       │
│  ✓ Escalated to personal questions                                  │
│  ✓ Now asking to meet & "keep it secret"                           │
│                                                                     │
│  IMMEDIATE ACTION RECOMMENDED                                       │
│                                                                     │
│  [🚫 Block Now]  [📱 Call Tommy]  [📋 View Chat]                   │
│                                                                     │
│  If not acknowledged in 15 minutes, we'll contact                   │
│  your backup (Sarah - Mom).                                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Escalation Chain

```rust
struct EscalationChain {
    primary: ParentContact,
    secondary: Option<ParentContact>,
    emergency: Option<ParentContact>,
}

impl AlertSystem {
    async fn escalate(&self, alert: &Alert) {
        match alert.tier {
            AlertTier::Critical | AlertTier::Emergency => {
                // 1. Try primary parent
                self.send_push(&alert, &self.chain.primary).await;
                self.send_sms(&alert, &self.chain.primary).await;
                
                // 2. Wait 15 minutes
                tokio::time::sleep(Duration::minutes(15)).await;
                
                if !alert.acknowledged {
                    // 3. Try secondary parent
                    if let Some(secondary) = &self.chain.secondary {
                        self.send_push(&alert, secondary).await;
                        self.send_sms(&alert, secondary).await;
                    }
                    
                    // 4. Wait another 15 minutes
                    tokio::time::sleep(Duration::minutes(15)).await;
                    
                    if !alert.acknowledged {
                        // 5. Try emergency contact
                        if let Some(emergency) = &self.chain.emergency {
                            self.send_call(&alert, emergency).await;
                        }
                    }
                }
            }
            _ => {
                // Lower tiers - just notify primary
                self.send_push(&alert, &self.chain.primary).await;
            }
        }
    }
}
```

## AI Summary Generation

```rust
impl AlertSystem {
    async fn generate_summary(&self, context: &AlertContext) -> String {
        let prompt = format!(r#"
Generate a brief, parent-friendly alert summary.
Be clear but not alarmist. Focus on facts and recommended actions.

Context:
- Child: {} (age {})
- Contact risk score: {}
- Topics detected: {:?}
- Risk factors: {:?}
- Time of activity: {}

Write 2-3 sentences. Be specific about concerns.
End with a clear recommended action.
"#,
            context.child_name,
            context.child_age,
            context.risk_score,
            context.topics,
            context.risk_factors,
            context.timestamp,
        );
        
        self.llm.generate(&prompt).await
    }
}

// Example outputs:

// LOW risk:
// "Tommy had a typical week, mostly chatting about gaming and school 
//  with established friends. No concerns detected."

// ELEVATED risk:
// "A new contact (CoolDude99) has been chatting with Tommy for 3 days.
//  Conversations are mostly about gaming, but they've asked about 
//  Tommy's school twice. We recommend keeping an eye on this contact."

// HIGH risk:
// "Contact xX_Shadow_Xx has asked Tommy about his location 3 times
//  and suggested moving to WhatsApp. These are common grooming signals.
//  We recommend reviewing the conversation and discussing online safety
//  with Tommy."

// CRITICAL risk:
// "URGENT: Contact xX_Shadow_Xx is exhibiting a classic grooming pattern.
//  Over 3 weeks, they've built trust, asked increasingly personal questions,
//  and are now requesting to meet in person. Immediate action recommended:
//  block this contact and talk to Tommy."
```

## Parent Actions

```rust
enum ParentAction {
    // Contact actions
    ApproveContact,
    BlockContact,
    MonitorClosely,
    
    // Alert actions
    Acknowledge,
    Dismiss,
    SnoozeHours(u32),
    
    // Conversation actions
    ViewReplay,
    ExportEvidence,  // Only for extreme cases
    ReportToCEOP,    // UK child protection
    
    // Child actions
    CallChild,
    SendMessage,
    LockDevice,      // Emergency only
}

// Action audit log
struct ActionLog {
    alert_id: AlertId,
    action: ParentAction,
    taken_at: DateTime,
    taken_by: ParentId,
    notes: Option<String>,
}
```

## Notification Channels

| Channel | Used For | Latency |
|---------|----------|---------|
| Push notification | All alerts | Immediate |
| SMS | HIGH and above | Immediate |
| Email | Digests only | Batched |
| Phone call | EMERGENCY escalation | After 30min no response |
| In-app | All alerts | On app open |

## Alert Preferences

```toml
# Per-family alert preferences

[alerts]
# Digest schedule
digest_day = "sunday"        # Day of week for weekly digest
digest_time = "09:00"        # Time to send
daily_digest_for_under_13 = true

# Notification preferences
push_enabled = true
sms_enabled = true
email_enabled = true

# Quiet hours (no non-emergency alerts)
quiet_start = "22:00"
quiet_end = "07:00"

# Escalation
escalation_timeout_minutes = 15
secondary_contact = "+44..."
emergency_contact = "+44..."

[alerts.thresholds]
# Customize when to alert (defaults shown)
elevated_threshold = 0.5
high_threshold = 0.7
critical_threshold = 0.85
```

## Do Not Alert List

Some things should NOT trigger alerts:

```rust
impl AlertSystem {
    fn should_suppress(&self, event: &Event) -> bool {
        // Don't alert for:
        
        // 1. Parent-approved contacts
        if self.is_approved_contact(&event.contact) {
            return true;
        }
        
        // 2. Family members
        if self.is_family_contact(&event.contact) {
            return true;
        }
        
        // 3. Educational sites during school hours
        if event.is_educational() && self.is_school_hours() {
            return true;
        }
        
        // 4. Known school friends with long history
        if self.is_established_school_friend(&event.contact) &&
           event.risk_score < 0.5 {
            return true;
        }
        
        // 5. Already alerted for same issue today
        if self.already_alerted_today(&event.contact, &event.trigger) {
            return true;
        }
        
        false
    }
}
```

## Related Documentation

- [Contact Intelligence](./CONTACT_INTELLIGENCE.md)
- [Topic Analysis](./TOPIC_ANALYSIS.md)
- [Privacy & Data Retention](./PRIVACY.md)
