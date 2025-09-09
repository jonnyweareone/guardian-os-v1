# Guardian OS API Integration

## Overview

Guardian OS devices communicate with the Supabase backend using JWT authentication. No API keys are stored on the device ISO - instead, devices obtain a unique JWT during installation.

## Authentication Flow

### 1. Parent Authentication

During installation, the parent either logs in or registers:

```bash
# Login
POST https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth/login
{
  "email": "parent@example.com",
  "password": "SecurePassword123!"
}

# Response
{
  "access_token": "parent_jwt_token",
  "refresh_token": "refresh_token",
  "user": { ... }
}
```

### 2. Device Claim

The installer claims the device using the parent's token:

```bash
POST https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/devices/claim
Authorization: Bearer parent_jwt_token
{
  "device_fingerprint": "sha256:unique_hardware_hash",
  "parent_email": "parent@example.com",
  "installer_version": "1.0.0"
}

# Response
{
  "device_jwt": "device_specific_jwt",
  "device_code": "GUARD-XXXX-XXXX",
  "policy": { ... }
}
```

### 3. Device Operations

All subsequent device operations use the device JWT:

```bash
# Heartbeat
POST https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/device/heartbeat
Authorization: Bearer device_jwt
{
  "status": "online",
  "config_hash": "sha256:current_config_hash",
  "versions": { ... }
}

# Config Acknowledgment
POST https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/device/config-ack
Authorization: Bearer device_jwt
{
  "config_version": "v2",
  "config_hash": "sha256:new_config_hash"
}
```

## Environment Configuration

The device stores its configuration in `/etc/guardian/supabase.env`:

```bash
SUPABASE_URL=https://xzxjwuzwltoapifcyzww.supabase.co
GUARDIAN_API_BASE=https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1
GUARDIAN_AUTH_LOGIN_URL=$GUARDIAN_API_BASE/auth/login
GUARDIAN_AUTH_REGISTER_URL=$GUARDIAN_API_BASE/auth/register
GUARDIAN_CLAIM_URL=$GUARDIAN_API_BASE/devices/claim
GUARDIAN_HEARTBEAT_URL=$GUARDIAN_API_BASE/device/heartbeat
GUARDIAN_CONFIG_ACK_URL=$GUARDIAN_API_BASE/device/config-ack
GUARDIAN_CHILDREN_URL=$GUARDIAN_API_BASE/children
GUARDIAN_TOKEN_REFRESH_URL=$GUARDIAN_API_BASE/token/refresh
GUARDIAN_DEVICE_JWT=<obtained_during_installation>
```

## Offline Activation

If the device cannot connect during installation:

1. Credentials are stored in `/etc/guardian/pending_activation.json`
2. The `guardian-activate.service` runs on boot
3. Once network is available, the service completes registration
4. On success, the pending file is removed

## Security Considerations

### JWT Storage

- Device JWT stored in `/etc/guardian/supabase.env` (mode 0600)
- Only root can read the JWT
- JWT has limited scope (device operations only)

### Device Fingerprinting

Device fingerprint is generated from:
- CPU model
- Network MAC addresses
- Machine ID

This ensures each device has a unique, stable identifier.

### API Rate Limiting

- Heartbeats: Every 10 minutes
- Config checks: Every 5 minutes
- Activation retries: Exponential backoff

## Testing

### Manual Device Claim

```bash
# Get parent token
TOKEN=$(curl -sX POST \
  "https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123!"}' \
  | jq -r '.access_token')

# Claim device
curl -X POST \
  "https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1/devices/claim" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_fingerprint": "sha256:test_fingerprint",
    "parent_email": "test@example.com",
    "installer_version": "1.0.0"
  }'
```

### Verify Heartbeat

```bash
# Source environment
source /etc/guardian/supabase.env

# Send test heartbeat
curl -X POST "$GUARDIAN_HEARTBEAT_URL" \
  -H "Authorization: Bearer $GUARDIAN_DEVICE_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "online",
    "config_hash": "sha256:test",
    "versions": {"os": "test", "agent": "test"}
  }'
```

## Troubleshooting

### No Device JWT

If `/etc/guardian/supabase.env` has empty `GUARDIAN_DEVICE_JWT`:

1. Check `/etc/guardian/pending_activation.json` exists
2. Run `sudo systemctl status guardian-activate.service`
3. Check logs: `sudo journalctl -u guardian-activate`

### Failed Heartbeats

1. Verify JWT is present: `grep DEVICE_JWT /etc/guardian/supabase.env`
2. Test connectivity: `curl -I https://xzxjwuzwltoapifcyzww.supabase.co`
3. Check logs: `sudo journalctl -u guardian-heartbeat`

### Manual Activation

If automatic activation fails:

```bash
sudo /usr/local/bin/guardian-activate
```
