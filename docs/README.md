# Guardian OS Architecture Documentation

## Overview

Guardian OS is a family-safe Linux distribution built on COSMIC Desktop, providing AI-powered protection for children online.

## Documentation Index

### Core Architecture
- [Architecture Overview](./architecture/OVERVIEW.md) - System architecture and component overview
- [DNS & Network Shield](./architecture/DNS_NETWORK.md) - DNS filtering, safe search, VPN detection
- [Contact Intelligence](./architecture/CONTACT_INTELLIGENCE.md) - Contact tracking and risk scoring
- [Alert System](./architecture/ALERT_SYSTEM.md) - Parent notifications and escalation
- [Age Tiers](./architecture/AGE_TIERS.md) - Age-appropriate protection levels
- [Privacy & Data Retention](./architecture/PRIVACY.md) - Data handling and retention policies

### Components

| Component | Description | Status |
|-----------|-------------|--------|
| guardian-daemon | Main orchestration service | 🔨 In Progress |
| guardian-dns | DNS filtering & logging | 🔨 In Progress |
| guardian-installer | Custom setup wizard | ✅ Complete |
| guardian-settings | COSMIC settings panel | 📋 Planned |
| guardian-sync-server | Family sync server | 📋 Planned |

### Quick Links

- [GitHub Repository](https://github.com/jonnyweareone/guardian-os-v1)
- [Build Instructions](../README.md)
- [Contributing Guidelines](../CONTRIBUTING.md)

## Protection Layers

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 1: Network (90% of threats blocked here)                 │
│  • DNS filtering, safe search, VPN detection                    │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: Input (Outbound risk prevention)                      │
│  • PII detection, keyboard monitoring                           │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3: Communication (Contact intelligence)                  │
│  • Topic logging, risk scoring, grooming detection              │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: Content (Visual/audio safety)                         │
│  • Screen scanning, voice chat monitoring                       │
└─────────────────────────────────────────────────────────────────┘
```

## Key Features

### For Parents
- **Topic Logging** - See what kids talk about (Gaming 45%, School 30%), not actual messages
- **Risk Scoring** - AI-powered contact assessment
- **Smart Alerts** - Only notified when it matters
- **Age Tiers** - Appropriate protection for each age
- **Family Trust** - Shared intelligence across siblings

### For Kids
- **Privacy Respected** - Topics logged, not messages
- **Age-Appropriate** - More freedom as they grow
- **Transparency** - Know what's being monitored
- **Ask Parent** - Request access to blocked content
- **Panic Button** - Get help when needed

### Privacy First
- No message content stored (except high-risk)
- Local processing when possible
- Automatic data expiration
- GDPR/UK Children's Code compliant

## Hardware Requirements

| Tier | Requirements | Features |
|------|-------------|----------|
| Minimum | Any quad-core, 8GB RAM | DNS + keyboard + topics |
| Recommended | i5/Ryzen 5, 16GB, GTX 1650 | All features |
| Optimal | i7/Ryzen 7, 32GB, RTX 3060+ | Near-instant detection |
