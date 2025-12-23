# Guardian OS - Per-Family ISO with GRUB Selection

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│  ONE ISO PER FAMILY                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ISO CONTAINS (Baked in):                                       │
│  ├── family_id                                                  │
│  ├── verification public_key                                    │
│  ├── supabase_url + anon_key                                    │
│  └── Guardian binaries (daemon, selector, launcher)             │
│                                                                 │
│  SUPABASE CONTAINS (Per child, fetched at runtime):             │
│  ├── experience_mode (kiosk/desktop_supervised/desktop_trusted) │
│  ├── unlock_method (ask_parent/face_id/pin/auto)                │
│  ├── trust_mode (supervised/monitored/trusted)                  │
│  ├── screen_time_policies                                       │
│  ├── dns_policies                                               │
│  └── app_policies                                               │
│                                                                 │
│  DEVICE STORES (Local):                                         │
│  ├── Linux user per child (/home/tommy, /home/emma, etc.)       │
│  ├── Face data per child (encrypted)                            │
│  ├── Activation state + signature                               │
│  └── Cached profiles (offline fallback)                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Boot Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  EVERY BOOT                                                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Power on → GRUB (hidden)                                    │
│  2. guardian-selector.service starts                            │
│  3. Shows child selection UI:                                   │
│     ┌─────────────────────────────────────────────┐            │
│     │  🛡️ Guardian OS                             │            │
│     │                                             │            │
│     │  Who's using this device?                   │            │
│     │                                             │            │
│     │  [👦 Tommy]  [👧 Emma]  [🧑 Jake]           │            │
│     └─────────────────────────────────────────────┘            │
│                                                                 │
│  4. Child selects their profile                                 │
│  5. Authentication based on unlock_method:                      │
│     ├── ask_parent → Push to phone → Wait for approve           │
│     ├── face_id → Scan face → Verify                            │
│     ├── pin → Enter PIN → Verify                                │
│     └── auto → No verification                                  │
│                                                                 │
│  6. On success:                                                 │
│     ├── Configure autologin for Linux user                      │
│     ├── Write /run/guardian/current_child                       │
│     └── Exit selector                                           │
│                                                                 │
│  7. Display manager starts → Autologin                          │
│  8. guardian-daemon fetches profile from Supabase               │
│  9. Applies experience_mode (kiosk or desktop)                  │
│  10. Kid can play! 🎮                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Files Created

### Database
- `supabase/migrations/005_build_system.sql` - Per-family builds
- `supabase/migrations/006_experience_modes.sql` - Experience modes, unlock methods, login requests

### Edge Functions
- `supabase/functions/trigger-build/index.ts` - Trigger family ISO build
- `supabase/functions/device-activate/index.ts` - Activate device, create Linux users
- `supabase/functions/login-request/index.ts` - ask_parent approval flow

### GitHub Actions
- `.github/workflows/build-family-iso.yml` - Build family ISO

### Guardian Components (Rust)
- `guardian-selector/` - Boot-time child selection
  - `src/main.rs` - Main logic, activation flow
  - `src/config.rs` - Config loading
  - `src/ui.rs` - Terminal UI (ratatui)
  - `src/auth.rs` - Authentication methods
  - `src/supabase.rs` - Supabase client

### Dashboard (Next.js)
- `src/app/(dashboard)/devices/page.tsx` - Build ISO, view devices
- `src/app/(dashboard)/children/[id]/settings/page.tsx` - Child settings (experience mode, unlock method)

## Unlock Methods

| Method | How it works | Best for |
|--------|--------------|----------|
| ask_parent | Push notification, parent approves | Under 10 |
| face_id | Biometric scan, PIN backup | 10-14 |
| pin | 4-6 digit PIN | 14+ |
| auto | No verification, parent notified | Trusted teens |

## Experience Modes

| Mode | What it does |
|------|--------------|
| kiosk | Game launcher only. No desktop, browser, files. |
| desktop_supervised | Full desktop, heavy monitoring, all activity logged |
| desktop_trusted | Full desktop, light monitoring, alerts on risky activity |

## Security

```
✅ Family-locked ISO (family_id baked in)
✅ Signature verification (ECDSA P-256)
✅ Per-child Linux users (isolated /home)
✅ Reboot required to switch profiles
✅ ask_parent requires phone approval
✅ face_id prevents sibling impersonation
✅ PIN lockout after 5 failed attempts
✅ Parent can reset PIN from dashboard
```

## Deployment

```bash
# 1. Apply database migrations
cd guardian-web
supabase db push

# 2. Deploy edge functions
supabase functions deploy trigger-build
supabase functions deploy device-activate
supabase functions deploy login-request

# 3. Set secrets
supabase secrets set GITHUB_TOKEN=ghp_xxx
supabase secrets set ONESIGNAL_APP_ID=xxx
supabase secrets set ONESIGNAL_API_KEY=xxx

# 4. Test build
# Open dashboard → Devices → Build ISO
```

## What's Next

- [ ] Face enrollment UI (Howdy integration)
- [ ] Push notification handling in parent app
- [ ] Kiosk shell UI (Electron)
- [ ] Voice chat monitoring
- [ ] P2P intervention system
