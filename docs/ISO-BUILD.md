# Guardian OS ISO Build Guide

## Prerequisites

Install build dependencies on Ubuntu 24.04:

```bash
sudo apt update
sudo apt install -y \
    live-build \
    debootstrap \
    reprepro \
    dpkg-dev \
    debhelper \
    devscripts \
    equivs \
    curl \
    gnupg2 \
    squashfs-tools \
    xorriso \
    isolinux \
    syslinux-efi \
    grub-pc-bin \
    grub-efi-amd64-bin \
    mtools \
    jq \
    python3-requests
```

## Build Process

### 1. Clone Repository

```bash
git clone https://github.com/jonnyweare/guardian-os-v1.git
cd guardian-os-v1
```

### 2. Fetch Assets

```bash
chmod +x scripts/*.sh
./scripts/fetch-assets.sh
```

### 3. Build Packages

```bash
make debs
# Or manually:
./scripts/build-debs.sh
```

### 4. Create Repository

```bash
make repo
# Or manually:
./scripts/build-repo.sh
```

### 5. Build ISO

```bash
make iso
# Or manually:
./scripts/iso-build.sh
```

The ISO will be created as `guardian-os-1.0.0-amd64.iso`.

## API Endpoints Used

The installer uses these Supabase endpoints:

- **Auth Login**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth-login`
- **Auth Register**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth-register`
- **Device Bind**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/bind-device`
- **Heartbeat**: `https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/device-heartbeat`

## Testing the ISO

### Virtual Machine Testing

```bash
# Using QEMU
qemu-system-x86_64 \
    -m 4096 \
    -cdrom guardian-os-1.0.0-amd64.iso \
    -boot d \
    -enable-kvm

# Using VirtualBox
VBoxManage createvm --name GuardianOS --ostype Ubuntu_64 --register
VBoxManage modifyvm GuardianOS --memory 4096 --vram 128
VBoxManage storagectl GuardianOS --name IDE --add ide
VBoxManage storageattach GuardianOS --storagectl IDE --port 0 --device 0 \
    --type dvddrive --medium guardian-os-1.0.0-amd64.iso
VBoxManage startvm GuardianOS
```

### Verification After Install

```bash
# Check device JWT was obtained
sudo cat /etc/guardian/supabase.env | grep GUARDIAN_DEVICE_JWT

# Check heartbeat service
sudo systemctl status guardian-heartbeat.timer
sudo systemctl status guardian-heartbeat.service

# View heartbeat logs
sudo journalctl -u guardian-heartbeat --since "10 min ago"

# Check device code
sudo cat /etc/guardian/device_code
```

## Troubleshooting

### Build Failures

- Ensure all dependencies are installed
- Check disk space (need ~10GB free)
- Run with sudo if permission errors

### Network Issues During Install

- The installer will create `/etc/guardian/pending_activation.json`
- After boot, `guardian-activate.service` will retry
- Check: `sudo systemctl status guardian-activate`

### Missing Device JWT

If the device JWT is empty after installation:

1. Check pending activation:
   ```bash
   ls -la /etc/guardian/pending_activation.json
   ```

2. Manually trigger activation:
   ```bash
   sudo /usr/local/bin/guardian-activate
   ```

3. Check logs:
   ```bash
   sudo journalctl -u guardian-activate
   sudo cat /var/log/guardian/activate.log
   ```
