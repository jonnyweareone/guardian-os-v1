![Guardian OS Logo](https://gameguardian.ai/lovable-uploads/guardian-logo-shield-text-dark.png)

<p align="center">
  <strong>AI-Powered Family Safety, Built on Pop!_OS</strong>
</p>

<p align="center">
  <a href="https://pop.system76.com/"><img src="https://img.shields.io/badge/Based%20on-Pop!__OS%2024.04-48B9C7" alt="Pop!_OS 24.04"></a>
  <a href="https://github.com/pop-os/cosmic-epoch"><img src="https://img.shields.io/badge/Desktop-COSMIC-orange" alt="COSMIC Desktop"></a>
  <img src="https://img.shields.io/badge/License-Personal%20Use-blue" alt="Personal Use License">
  <img src="https://img.shields.io/badge/Status-Beta-yellow" alt="Beta">
</p>

---

## 🛡️ What is Guardian OS?

Guardian OS is a **family-safe Linux distribution** built on [Pop!_OS](https://pop.system76.com/) by [System76](https://system76.com/). It combines the stability and performance of Pop!_OS with powerful, AI-driven parental controls that work at the operating system level.

Unlike browser extensions or app-based filters that can be bypassed, Guardian OS provides **deep, system-level protection** — monitoring screen content, filtering network traffic, and enforcing healthy digital habits.

---

## 🙏 Built on Pop!_OS

Guardian OS wouldn't be possible without the incredible work of **System76** and their **Pop!_OS** distribution. We're proud to build upon:

- **[Pop!_OS](https://pop.system76.com/)** — A developer-focused Linux distribution known for its polish, performance, and hardware support
- **[COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)** — System76's modern, Rust-based desktop environment
- **[cosmic-sync-server](https://github.com/nicoulaj/cosmic-sync-server)** — Settings sync infrastructure we've adapted for family settings

**System76** has been a pioneer in making Linux accessible and powerful. Guardian OS extends their vision to families, adding safety features while preserving the freedom and privacy that makes Linux great.

> 💙 **Thank you, System76!** Your commitment to open source and user freedom inspires everything we do.

---

## 🌟 Why Guardian OS?

The internet wasn't designed with children in mind. Parents face an impossible choice: over-restrict their kids or expose them to harmful content.

**Guardian OS changes that.**

We believe kids deserve a safe, empowering digital world — and parents deserve peace of mind without constant hovering.

---

## 🚀 Features

### 🔒 Smart Parental Controls
Create parent and child profiles with granular permissions. Parents see everything; kids see what's safe.

### 🧠 AI-Powered Safety (Coming Soon)
- **Screen Sentinel** — Real-time visual content analysis using on-device AI
- **Audio Guardian** — Voice monitoring for grooming detection and emotional distress
- **Network Shield** — Intelligent DNS filtering and traffic analysis
- **Behavior Analyzer** — Pattern recognition for concerning activity

### 🎮 Family App Store
Apps and games with age ratings, safety warnings, and parent approval prompts.

### ☁️ Parent Dashboard
Manage devices, set rules, approve apps, and monitor activity from anywhere.

### ⏰ Screen Time & Routines
Set daily limits, homework hours, and bedtime shutdowns.

### 🚨 Smart Alerts
Get notified about risky searches or concerning behavior — without micromanaging.

### 🔐 Privacy First
- All AI models run **locally on device**
- Screen frames analyzed and immediately discarded
- Only metadata syncs to cloud (timestamps, app names, alerts)
- End-to-end encrypted family data
- **Your family's data stays your family's data**

---

## 🛡️ Guardian Daemon — The Heart of Protection

The **Guardian Daemon** (`guardian-daemon`) is a Rust-based systemd service that runs continuously in the background, providing comprehensive protection for children online. It's designed to be lightweight, efficient, and impossible to bypass at the user level.

### How It Protects Children

#### 1. 📱 Device Registration & Activation
When Guardian OS is first installed, the daemon:
- Generates a unique **6-character activation code** (e.g., `A7X9K2`)
- Registers the device with the Guardian cloud using hardware fingerprinting
- Links the device to a parent's account when they enter the code in the mobile app
- Obtains secure JWT tokens for ongoing cloud communication

#### 2. ⏰ Screen Time Enforcement
The daemon enforces healthy digital habits through:
- **Daily time limits** — Automatically locks the session when time runs out
- **Scheduled bedtimes** — Gradual warnings then session lock at bedtime
- **Homework hours** — Restrict to educational apps during study time
- **Break reminders** — Encourage kids to take breaks from screens
- **Per-app limits** — Set specific limits for games vs. educational content

#### 3. 🚫 Application Control
Parents can control what apps children can use:
- **Allowlist mode** — Only pre-approved apps can run
- **Blocklist mode** — Block specific applications
- **Age-based filtering** — Apps rated above child's age require approval
- **Install protection** — New app installs require parent approval
- **Process monitoring** — Detects and blocks restricted applications in real-time

#### 4. 🌐 Web & Network Protection
The daemon integrates with system DNS to provide:
- **Category-based blocking** — Block adult content, gambling, social media, etc.
- **Safe search enforcement** — Forces Google/Bing/YouTube safe search
- **HTTPS inspection** — Detects bypasses via DNS-over-HTTPS
- **Custom blocklists** — Parents can add specific domains to block
- **Time-based rules** — Social media allowed only after homework

#### 5. 📊 Activity Monitoring & Reporting
The daemon tracks activity and syncs to the parent dashboard:
- **Active window tracking** — Which apps are being used and for how long
- **Website history** — Domains visited (not full URLs for privacy)
- **Search queries** — Flagged if they contain concerning terms
- **Session summaries** — Daily/weekly reports for parents
- **Real-time alerts** — Instant notifications for policy violations

#### 6. 🔒 Anti-Bypass Protection
Guardian Daemon is designed to resist tampering:
- Runs as **root-level systemd service** — cannot be stopped by child users
- **Process watchdog** — automatically restarts if killed
- **Configuration protection** — settings encrypted and require parent PIN
- **Boot persistence** — starts automatically on every boot
- **TTY lockdown** — prevents switching to virtual terminals to bypass

#### 7. 🤖 AI-Powered Detection (Roadmap)
Future versions will include on-device AI models:
- **Screen Sentinel** — Captures screen frames, runs NudeNet/CLIP locally to detect inappropriate visual content, immediately discards frames after analysis
- **Audio Guardian** — Uses Whisper.cpp to monitor microphone for signs of online grooming, bullying, or emotional distress
- **Behavior Analysis** — ML models to detect unusual patterns (e.g., late-night usage, secretive behavior)

### Technical Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Guardian Daemon                             │
│                    (systemd service)                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Scheduler  │  │   Monitor    │  │   Enforcer   │          │
│  │              │  │              │  │              │          │
│  │ • Time rules │  │ • X11/Wayland│  │ • Session    │          │
│  │ • Bedtimes   │  │ • Process    │  │   locking    │          │
│  │ • Breaks     │  │ • Network    │  │ • App block  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  AI Engine   │  │  Cloud Sync  │  │   Alerter    │          │
│  │  (Future)    │  │              │  │              │          │
│  │ • NudeNet    │  │ • Supabase   │  │ • Push notif │          │
│  │ • Whisper    │  │ • JWT auth   │  │ • Email      │          │
│  │ • CLIP       │  │ • Realtime   │  │ • Dashboard  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    System Integration                            │
├─────────────────────────────────────────────────────────────────┤
│  • D-Bus (session control)    • NetworkManager (DNS filtering)  │
│  • logind (session tracking)  • polkit (privilege escalation)   │
│  • X11/Wayland (display)      • systemd (service management)    │
└─────────────────────────────────────────────────────────────────┘
```

### Configuration

The daemon reads configuration from `/etc/guardian/config.toml`:

```toml
[device]
device_id = "auto-generated"
activation_code = "A7X9K2"
registered = true

[cloud]
api_url = "https://gkyspvcafyttfhyjryyk.supabase.co"
sync_interval_secs = 30
realtime_enabled = true

[enforcement]
screen_time_enabled = true
app_blocking_enabled = true
web_filtering_enabled = true
lock_on_limit = true

[monitoring]
track_active_window = true
track_websites = true
track_searches = true
send_alerts = true

[ai]
screen_sentinel_enabled = false  # Coming soon
audio_guardian_enabled = false   # Coming soon
local_inference_only = true      # Never send to cloud
```

### Daemon Commands

```bash
# Check daemon status
sudo systemctl status guardian-daemon

# View real-time logs
sudo journalctl -u guardian-daemon -f

# Restart daemon (requires root)
sudo systemctl restart guardian-daemon

# Show current session info
guardian-cli status

# Manually trigger cloud sync
guardian-cli sync
```

---

## 🏗️ Architecture

Guardian OS consists of several Rust components:

```
guardian-components/
├── guardian-daemon      # Core safety service (systemd daemon)
├── guardian-wizard      # First-boot setup wizard (COSMIC/iced)
├── guardian-settings    # Parental control panel (COSMIC/iced)
└── guardian-store       # Family-safe app store (COSMIC/iced)
```

### System Stack

| Layer | Technology |
|-------|------------|
| Base OS | Pop!_OS 24.04 LTS |
| Desktop | COSMIC (Rust/iced) |
| Init | systemd |
| Safety Daemon | Rust + Tokio |
| Local AI | ONNX Runtime |
| Cloud Sync | Supabase |
| Settings Sync | cosmic-sync-server (adapted) |

---

## 📦 Quick Start

### Install from ISO (Recommended)

Download the latest ISO from [Releases](https://github.com/jonnyweareone/guardian-os-v1/releases) and boot it on any PC.

### Install on Existing Pop!_OS

```bash
# Download the daemon package
wget https://github.com/jonnyweareone/guardian-os-v1/releases/download/v1.0.0/guardian-daemon_1.0.0_amd64.deb

# Install
sudo dpkg -i guardian-daemon_1.0.0_amd64.deb

# Enable and start
sudo systemctl enable --now guardian-daemon
```

### Build from Source

```bash
# Clone
git clone https://github.com/jonnyweareone/guardian-os-v1.git
cd guardian-os-v1

# Build components
cd guardian-components/guardian-daemon
cargo build --release

# Build ISO (requires Linux)
cd ../../iso-builder
sudo ./build-iso.sh
```

---

## 🔐 Security Model

- **No hardcoded secrets** — Devices obtain JWT tokens during activation
- **Hardware fingerprinting** — Unique device identification via machine-id
- **Local-first AI** — Sensitive analysis never leaves the device
- **Encrypted sync** — All cloud communication over TLS
- **LUKS encryption** — Full disk encryption available
- **Tamper resistance** — Daemon protected from child user interference

---

## 🗺️ Roadmap

### ✅ Completed (v1.0)
- [x] Core daemon with screen time enforcement
- [x] Device registration and activation flow  
- [x] Supabase backend integration
- [x] COSMIC-based setup wizard
- [x] Parent mobile app (PWA)
- [x] Real-time cloud sync
- [x] Application monitoring

### 🚧 In Progress (v1.1)
- [ ] Web filtering via DNS
- [ ] Per-app time limits
- [ ] Bedtime enforcement
- [ ] Break reminders

### 🔮 Future (v2.0)
- [ ] Screen Sentinel (NudeNet + CLIP)
- [ ] Audio Guardian (Whisper.cpp)
- [ ] Network Shield (AI DNS filtering)
- [ ] Guardian Agent (Claude-powered assistant)
- [ ] Native mobile apps (iOS/Android)
- [ ] Guardian Router integration

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas we need help:
- AI model optimization for low-power devices
- COSMIC desktop integration
- Accessibility features
- Internationalization

---

## 📄 License

Guardian OS is released under a **Personal Use License**:
- ✅ Free for personal and educational use
- ❌ Commercial use requires a license from We Are One 1 Limited

See [LICENSE](./LICENSE) and [TRADEMARKS.md](./TRADEMARKS.md).

**Note:** Pop!_OS and COSMIC components retain their original open-source licenses (GPL, MPL, etc.).

---

## 🆘 Support

- 📚 **Documentation**: [docs/](docs/)
- 🐛 **Issues**: [GitHub Issues](https://github.com/jonnyweareone/guardian-os-v1/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/jonnyweareone/guardian-os-v1/discussions)
- 🌐 **Website**: [gameguardian.ai](https://gameguardian.ai)

---

## 🙏 Acknowledgments

Guardian OS is built on the shoulders of giants:

- **[System76](https://system76.com/)** & **[Pop!_OS](https://pop.system76.com/)** — For the incredible base OS and COSMIC desktop
- **[COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)** — The beautiful, modern Rust desktop environment
- **[iced](https://iced.rs/)** — The Rust GUI framework powering COSMIC
- **[Supabase](https://supabase.com/)** — Backend infrastructure
- **[NudeNet](https://github.com/notAI-tech/NudeNet)** — NSFW detection model
- **[Whisper](https://github.com/openai/whisper)** — Speech recognition
- **The Rust Community** — For making systems programming safe and enjoyable

---

<p align="center">
  <strong>Made with ❤️ for families everywhere</strong>
</p>

<p align="center">
  <a href="https://pop.system76.com/">
    <img src="https://img.shields.io/badge/Proudly%20Built%20on-Pop!__OS-48B9C7?style=for-the-badge&logo=pop!_os" alt="Built on Pop!_OS">
  </a>
</p>
