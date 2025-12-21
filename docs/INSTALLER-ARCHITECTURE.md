# Guardian OS Installer Architecture - CORRECTED

## The Two Wizards

### 1. Live ISO Installer: `pop-installer` (Vala/GTK)
**When:** During live ISO session, user clicks "Install"
**Purpose:** Actual OS installation to disk
**Current flow:**
```
Language → Keyboard → Try/Install → User → Disk → Encrypt → Progress → Success
```

**Guardian modification needed:**
```
Language → Keyboard → Try/Install → GUARDIAN AUTH → CHILD SELECT → User → Disk → Encrypt → Progress → Success
```

We need to **fork pop-os/installer (Vala)** and add:
- GuardianAuthView.vala - Parent sign-in
- GuardianChildView.vala - Child selection
- Modify User.vala to auto-fill from child profile

### 2. First Boot Wizard: `guardian-installer` (Rust/COSMIC)  
**When:** First boot after installation
**Purpose:** Post-install customization
**Current flow:**
```
Welcome → WiFi → Language → Keyboard → Timezone → Appearance → Layout
```

**Guardian modification:**
```
Welcome → WiFi → Appearance → Layout
```
(Stripped down - just cosmetic customization, no auth needed)

## Summary

| Component | Language | Purpose | Guardian Changes |
|-----------|----------|---------|------------------|
| `pop-installer` | Vala/GTK | Install OS to disk | ADD auth + child selection |
| `guardian-installer` | Rust/COSMIC | Post-install setup | SIMPLIFY to just appearance |

## What We Built (Current State)

We've been working on `guardian-installer` (the Rust/COSMIC one) which is the POST-install wizard.
- Added Guardian auth pages
- Added child selection
- Stripped languages

**But this is wrong!** The Guardian auth needs to happen DURING installation (in pop-installer), 
not after first boot.

## What We Need To Do

### Option A: Fork pop-installer (Vala)
- Clone https://github.com/pop-os/installer
- Add Vala pages for Guardian auth
- Requires learning Vala, maintaining legacy code
- User creation happens in installer

### Option B: Replace pop-installer with Rust installer
- Extend guardian-installer to do full installation
- Add disk partitioning, distinst integration
- More work but modern codebase
- Matches COSMIC architecture

### Option C: Pre-flight Guardian app
- Create separate "Guardian Setup" app that runs BEFORE pop-installer
- Stores credentials, pop-installer reads them
- Least invasive but awkward UX

## Recommended: Option A (Fork pop-installer)

The pop-installer is still used in Pop!_OS 24.04. It's stable, handles all the complex 
disk/bootloader stuff via distinst. We just need to inject 2-3 screens.

## Post-Install Wizard (guardian-installer)

This stays simple - just:
1. Welcome to Guardian OS
2. Appearance (theme)
3. Layout (panel/dock)

No auth needed - device is already registered during installation.
