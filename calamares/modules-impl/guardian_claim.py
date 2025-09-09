#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import json
import hashlib
import subprocess
import requests
import libcalamares
from libcalamares.utils import debug, warning, error

# API endpoints - using correct bind-device URL
API_BASE = "https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1"
CLAIM_URL = f"{API_BASE}/bind-device"

def get_device_fingerprint():
    """Generate unique device fingerprint"""
    
    components = []
    
    # CPU info
    try:
        with open("/proc/cpuinfo", "r") as f:
            for line in f:
                if line.startswith("model name"):
                    components.append(line.split(":")[1].strip())
                    break
    except:
        pass
    
    # MAC addresses
    try:
        result = subprocess.run(
            ["ip", "link", "show"],
            capture_output=True,
            text=True
        )
        for line in result.stdout.split("\n"):
            if "link/ether" in line:
                mac = line.split("link/ether")[1].split()[0]
                components.append(mac)
    except:
        pass
    
    # Machine ID
    try:
        with open("/etc/machine-id", "r") as f:
            components.append(f.read().strip())
    except:
        # Generate a random ID if machine-id not available
        import uuid
        components.append(str(uuid.uuid4()))
    
    # Create hash
    fingerprint_data = "|".join(components)
    fingerprint = hashlib.sha256(fingerprint_data.encode()).hexdigest()
    
    return f"sha256:{fingerprint}"

def run():
    """Claim device with backend"""
    
    # Get root mount point for writing config
    root_mount = libcalamares.globalstorage.value("rootMountPoint")
    if not root_mount:
        root_mount = "/"
    
    # Check if offline mode
    offline_mode = libcalamares.globalstorage.value("guardian_offline_mode")
    if offline_mode:
        debug("Offline mode - will claim device after installation")
        
        # Write pending activation file
        guardian_dir = os.path.join(root_mount, "etc/guardian")
        os.makedirs(guardian_dir, mode=0o700, exist_ok=True)
        
        pending_file = os.path.join(guardian_dir, "pending_activation.json")
        pending_data = {
            "email": libcalamares.globalstorage.value("guardian_pending_email"),
            "fingerprint": get_device_fingerprint(),
            "timestamp": subprocess.check_output(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True).strip()
        }
        
        with open(pending_file, "w") as f:
            json.dump(pending_data, f, indent=2)
        os.chmod(pending_file, 0o600)
        
        # Write empty supabase.env
        write_supabase_env(root_mount, "")
        
        return None
    
    # Get parent token
    parent_token = libcalamares.globalstorage.value("guardian_parent_token")
    
    # Try to read from temp file if not in globalstorage
    if not parent_token and os.path.exists("/tmp/guardian_parent_token"):
        try:
            with open("/tmp/guardian_parent_token", "r") as f:
                parent_token = f.read().strip()
        except:
            pass
    
    if not parent_token:
        warning("No parent token available")
        return ("Device claim failed", "Authentication required")
    
    parent_email = libcalamares.globalstorage.value("guardian_parent_email")
    if not parent_email:
        parent_email = os.environ.get("GUARDIAN_TEST_EMAIL", "unknown@example.com")
    
    # Generate device fingerprint
    device_fingerprint = get_device_fingerprint()
    debug(f"Device fingerprint: {device_fingerprint}")
    
    # Claim device with bind-device endpoint
    try:
        response = requests.post(
            CLAIM_URL,
            json={
                "device_fingerprint": device_fingerprint,
                "parent_email": parent_email,
                "installer_version": "1.0.0",
                "os_version": "Guardian Ubuntu 24.04"
            },
            headers={
                "Authorization": f"Bearer {parent_token}",
                "Content-Type": "application/json"
            },
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            
            device_jwt = data.get("device_jwt", "")
            device_code = data.get("device_code", "")
            
            # Write supabase.env with device JWT
            write_supabase_env(root_mount, device_jwt)
            
            # Write device code file
            guardian_dir = os.path.join(root_mount, "etc/guardian")
            os.makedirs(guardian_dir, mode=0o700, exist_ok=True)
            
            device_code_file = os.path.join(guardian_dir, "device_code")
            with open(device_code_file, "w") as f:
                f.write(device_code)
            os.chmod(device_code_file, 0o600)
            
            # Store in globalstorage for other modules
            libcalamares.globalstorage.insert("guardian_device_jwt", device_jwt)
            libcalamares.globalstorage.insert("guardian_device_code", device_code)
            
            debug(f"Device claimed successfully: {device_code}")
            return None
        else:
            error(f"Device claim failed: {response.status_code}")
            error(f"Response: {response.text}")
            return ("Device claim failed", f"Server returned {response.status_code}")
            
    except requests.exceptions.RequestException as e:
        warning(f"Network error during device claim: {e}")
        
        # Write pending activation
        guardian_dir = os.path.join(root_mount, "etc/guardian")
        os.makedirs(guardian_dir, mode=0o700, exist_ok=True)
        
        pending_file = os.path.join(guardian_dir, "pending_activation.json")
        pending_data = {
            "email": parent_email,
            "fingerprint": device_fingerprint,
            "timestamp": subprocess.check_output(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True).strip()
        }
        
        with open(pending_file, "w") as f:
            json.dump(pending_data, f, indent=2)
        os.chmod(pending_file, 0o600)
        
        # Write empty supabase.env
        write_supabase_env(root_mount, "")
        
        debug("Will claim device after installation")
        return None

def write_supabase_env(root_mount, device_jwt):
    """Write the supabase.env file with all endpoints"""
    
    guardian_dir = os.path.join(root_mount, "etc/guardian")
    os.makedirs(guardian_dir, mode=0o700, exist_ok=True)
    
    env_file = os.path.join(guardian_dir, "supabase.env")
    
    content = f"""# Guardian OS Supabase Configuration
# Generated during installation
SUPABASE_URL=https://xzxjwuzwltoapifcyzww.supabase.co
GUARDIAN_API_BASE=https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1

# API Endpoints
GUARDIAN_AUTH_LOGIN_URL=$GUARDIAN_API_BASE/auth-login
GUARDIAN_AUTH_REGISTER_URL=$GUARDIAN_API_BASE/auth-register
GUARDIAN_CLAIM_URL=$GUARDIAN_API_BASE/bind-device
GUARDIAN_HEARTBEAT_URL=$GUARDIAN_API_BASE/device-heartbeat

# Device JWT (obtained during installation)
GUARDIAN_DEVICE_JWT={device_jwt}
"""
    
    with open(env_file, "w") as f:
        f.write(content)
    
    os.chmod(env_file, 0o600)
    debug(f"Wrote supabase.env to {env_file}")
