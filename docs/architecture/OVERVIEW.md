# Guardian OS - Protection Architecture

## Overview

Guardian OS provides multi-layered protection for children through intelligent, context-aware monitoring that respects privacy while ensuring safety.

## Core Design Principles

1. **Context-Aware Monitoring** - Don't waste resources scanning safe contexts
2. **Privacy-First** - Log topics, not content. Store patterns, not data.
3. **Age-Appropriate** - Different protection levels for different ages
4. **Family Trust Network** - Shared intelligence across siblings
5. **Minimal Resource Usage** - Works on any hardware

## Protection Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Layer 1: Network                             │
│                    (Blocks 90% of threats)                           │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ guardian-dns                                                 │   │
│  │ • DNS filtering (block inappropriate domains)                │   │
│  │ • Safe search enforcement (Google, Bing, YouTube)            │   │
│  │ • VPN/proxy detection and blocking                           │   │
│  │ • Browsing logs (domains only, not full URLs)                │   │
│  │ • Latency: 10-50ms                                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                         Layer 2: Input                               │
│                    (Catches outbound risks)                          │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Keyboard Monitor                                             │   │
│  │ • PII detection (address, phone, school)                     │   │
│  │ • Real-time interception before send                         │   │
│  │ • Context-aware (only in browsers/social apps)               │   │
│  │ • Zero CPU overhead                                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                         Layer 3: Communication                       │
│                    (Contact intelligence)                            │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Contact Tracker                                              │   │
│  │ • Vector memory for contacts (no PII stored)                 │   │
│  │ • Topic logging (Gaming 45%, School 30%)                     │   │
│  │ • Risk scoring with AI moderation                            │   │
│  │ • Grooming pattern detection                                 │   │
│  │ • Family trust sharing across siblings                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                         Layer 4: Content                             │
│                    (Visual/audio safety)                             │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Screen Sentinel                                              │   │
│  │ • NudeNet for inappropriate image detection                  │   │
│  │ • Context-aware (skip gameplay, scan browsers)               │   │
│  │ • GPU accelerated (200-500ms with GTX 1650+)                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Audio Guardian                                               │   │
│  │ • Whisper.cpp for voice chat transcription                   │   │
│  │ • Local-only processing (privacy)                            │   │
│  │ • Triggered by voice chat detection                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Overview

| Component | Purpose | Location |
|-----------|---------|----------|
| guardian-daemon | Main orchestration service | `/usr/lib/guardian/` |
| guardian-dns | DNS filtering & logging | `/usr/lib/guardian/` |
| guardian-context | App/window detection | Part of daemon |
| guardian-keyboard | Input monitoring | Part of daemon |
| guardian-screen | Visual content scanning | Part of daemon |
| guardian-audio | Voice chat monitoring | Part of daemon |
| guardian-contacts | Contact intelligence | Part of daemon |

## Data Flow

```
User Activity
     │
     ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Context   │────▶│  Decision   │────▶│  Monitors   │
│   Engine    │     │   Engine    │     │  (active)   │
└─────────────┘     └─────────────┘     └─────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │   Analysis  │
                   │   (AI/ML)   │
                   └─────────────┘
                          │
                          ▼
                   ┌─────────────┐     ┌─────────────┐
                   │   Storage   │────▶│   Alerts    │
                   │  (topics)   │     │  (parents)  │
                   └─────────────┘     └─────────────┘
```

## Hardware Requirements

| Tier | CPU | RAM | GPU | Experience |
|------|-----|-----|-----|------------|
| Minimum | Any quad-core | 8GB | None | DNS + keyboard + topics |
| Recommended | i5/Ryzen 5 | 16GB | GTX 1650 | All features |
| Optimal | i7/Ryzen 7 | 32GB | RTX 3060+ | Near-instant detection |

## Related Documentation

- [DNS & Network Shield](./DNS_NETWORK.md)
- [Contact Intelligence](./CONTACT_INTELLIGENCE.md)
- [Topic Analysis](./TOPIC_ANALYSIS.md)
- [Alert System](./ALERT_SYSTEM.md)
- [Age Tiers](./AGE_TIERS.md)
- [Privacy & Data Retention](./PRIVACY.md)
