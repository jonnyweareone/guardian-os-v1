# Guardian OS - Supabase Integration Summary

## ✅ Endpoints Configured

All components now use the correct `/functions/v1` URL format:

- **Auth Login**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth-login`
- **Auth Register**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth-register`
- **Bind Device**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/bind-device`
- **Device Heartbeat**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/device-heartbeat`

## 📁 Repository Structure

```
guardian-os-v1/
├── Makefile                    # Build system
├── scripts/
│   ├── fetch-assets.sh        # Downloads brand assets
│   ├── build-debs.sh          # Builds Debian packages
│   ├── build-repo.sh          # Creates APT repository
│   └── iso-build.sh           # Builds ISO image
├── calamares/
│   ├── settings.conf          # Installer configuration
│   ├── branding/guardian/     # Branding assets
│   ├── modules/*.conf         # Module configs
│   └── modules-impl/*.py      # Python implementations
├── packages/
│   ├── guardian-heartbeat/    # Heartbeat service
│   └── guardian-activate/     # Offline activation
├── iso/includes.chroot/       # System configuration
│   └── etc/
│       ├── dconf/            # GNOME settings
│       └── skel/.config/     # Chrome as default
├── docs/
│   ├── ISO-BUILD.md          # Build instructions
│   └── APT-REPO.md           # Repository guide
└── test-endpoints.sh          # Endpoint testing script
```

## 🔑 Key Features Implemented

### 1. **Calamares Modules**
- `guardian_wifi` - Network setup
- `guardian_auth` - Parent login/registration (calls `/auth-login` or `/auth-register`)
- `guardian_claim` - Device registration (calls `/bind-device`)
- `guardian_apply` - System configuration

### 2. **Device Environment** (`/etc/guardian/supabase.env`)
```bash
SUPABASE_URL=https://xzxjwuzwltoapifcyzww.supabase.co
GUARDIAN_API_BASE=https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1
GUARDIAN_AUTH_LOGIN_URL=$GUARDIAN_API_BASE/auth-login
GUARDIAN_AUTH_REGISTER_URL=$GUARDIAN_API_BASE/auth-register
GUARDIAN_CLAIM_URL=$GUARDIAN_API_BASE/bind-device
GUARDIAN_HEARTBEAT_URL=$GUARDIAN_API_BASE/device-heartbeat
GUARDIAN_DEVICE_JWT=<filled_after_claim>
```

### 3. **Heartbeat Service**
- Runs every 10 minutes via systemd timer
- Posts to `/device-heartbeat` with JWT auth
- Sends: `{"status":"online","versions":{"os":"Guardian Ubuntu 24.04","agent":"0.1.0"},"config_hash":"sha256:..."}`

### 4. **Offline Activation**
- Creates `/etc/guardian/pending_activation.json` if no network during install
- `guardian-activate.service` retries on boot
- Falls back to manual activation wizard if needed

### 5. **System Configuration**
- GNOME wallpaper set to Guardian branded image
- Chrome as default browser via mimeapps.list
- Dock favorites: Chrome, LibreOffice, Files, Settings

## 🚀 Quick Start Commands

### Test Endpoints
```bash
chmod +x test-endpoints.sh
./test-endpoints.sh
```

### Build ISO (on Ubuntu 24.04)
```bash
# Install dependencies
sudo apt install -y live-build debootstrap reprepro dpkg-dev \
    debhelper devscripts curl gnupg2 jq python3-requests

# Build
chmod +x scripts/*.sh
./scripts/fetch-assets.sh
make debs
make repo
make iso
```

### Test Installation Flow
```bash
# During install, the flow is:
# 1. Auth: POST /auth-login → parent_access_token
# 2. Claim: POST /bind-device (Bearer token) → device_jwt
# 3. Write: /etc/guardian/supabase.env with GUARDIAN_DEVICE_JWT

# After install, verify:
sudo cat /etc/guardian/supabase.env | grep JWT
sudo systemctl status guardian-heartbeat.timer
sudo journalctl -u guardian-heartbeat --since "10 min ago"
```

## 📝 Important Notes

1. **NO ANON KEY ON ISO** - The ISO never contains the Supabase anon key
2. **JWT Auth Only** - Devices use only their device JWT after registration
3. **Correct URL Format** - All endpoints use `/functions/v1/` (not functions.supabase.co)
4. **Offline Support** - Full offline activation path implemented

## 🔧 Next Steps

1. **Deploy Edge Functions** (Lovable side):
```bash
supabase functions deploy auth-login
supabase functions deploy auth-register
# bind-device and device-heartbeat should already exist
```

2. **Set Function Environment Variables**:
- `SUPABASE_URL`
- `SUPABASE_ANON_KEY`
- `SUPABASE_SERVICE_ROLE_KEY`
- `DEVICE_JWT_SECRET`

3. **Configure verify_jwt in config.toml**:
```toml
[functions.auth-login]
verify_jwt = false
[functions.auth-register]
verify_jwt = false
[functions.bind-device]
verify_jwt = true
[functions.device-heartbeat]
verify_jwt = false
```

4. **Build and Test ISO**:
```bash
cd guardian-os-v1
make iso
# Test in VM
qemu-system-x86_64 -m 4096 -cdrom guardian-os-*.iso -boot d
```

## ✅ Checklist

- [x] Scripts use correct `/functions/v1` endpoints
- [x] Calamares modules call auth-login, auth-register, bind-device
- [x] Heartbeat posts to device-heartbeat with JWT
- [x] supabase.env written with all URLs and device JWT
- [x] Offline activation with pending_activation.json
- [x] Chrome set as default browser
- [x] GNOME configured with Guardian wallpaper
- [x] No anon key stored anywhere on ISO

The system is ready for integration with your Supabase backend!
