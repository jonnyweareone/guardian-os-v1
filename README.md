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

---

## 🗺️ Roadmap

- [x] Core daemon with screen time enforcement
- [x] Device registration and activation flow
- [x] Supabase backend integration
- [x] COSMIC-based setup wizard
- [ ] Screen Sentinel (NudeNet + CLIP)
- [ ] Audio Guardian (Whisper.cpp)
- [ ] Network Shield (AI DNS filtering)
- [ ] Guardian Agent (Claude-powered assistant)
- [ ] Mobile parent app (iOS/Android)
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
