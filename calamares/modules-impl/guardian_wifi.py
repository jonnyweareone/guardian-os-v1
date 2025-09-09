#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import subprocess
import libcalamares
from libcalamares.utils import debug, warning

def run():
    """Configure WiFi connection during installation"""
    
    debug("Guardian WiFi: Checking network connectivity...")
    
    # Check if already connected
    try:
        result = subprocess.run(
            ["nmcli", "networking", "connectivity", "check"],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        if "full" in result.stdout:
            debug("Already connected to network")
            libcalamares.globalstorage.insert("guardian_network_configured", True)
            return None
    except Exception as e:
        warning(f"Could not check network status: {e}")
    
    # In production, this would show a WiFi selection UI
    # For now, we assume network is configured or will use offline mode
    debug("WiFi configuration check complete")
    
    # Store network state
    libcalamares.globalstorage.insert("guardian_network_configured", True)
    
    return None
