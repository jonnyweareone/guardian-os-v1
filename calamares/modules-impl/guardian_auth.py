#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import json
import os
import tempfile
import requests
import libcalamares
from libcalamares.utils import debug, warning, error

# API endpoints - using correct URLs
API_BASE = "https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1"
AUTH_LOGIN_URL = f"{API_BASE}/auth-login"
AUTH_REGISTER_URL = f"{API_BASE}/auth-register"

def run():
    """Guardian Authentication Module"""
    
    # Get user input (in production, from UI)
    auth_mode = libcalamares.globalstorage.value("guardian_auth_mode")  # "login" or "register"
    email = libcalamares.globalstorage.value("guardian_auth_email")
    password = libcalamares.globalstorage.value("guardian_auth_password")
    
    # For testing, use environment variables if UI values not set
    if not email:
        email = os.environ.get("GUARDIAN_TEST_EMAIL", "")
    if not password:
        password = os.environ.get("GUARDIAN_TEST_PASSWORD", "")
    
    if not email or not password:
        warning("Authentication credentials not provided")
        # Store for offline activation
        libcalamares.globalstorage.insert("guardian_offline_mode", True)
        return None
    
    debug(f"Authenticating as {email}...")
    
    try:
        if auth_mode == "register":
            # Register new account
            response = requests.post(
                AUTH_REGISTER_URL,
                json={
                    "email": email,
                    "password": password
                },
                headers={"Content-Type": "application/json"},
                timeout=30
            )
        else:
            # Login existing account
            response = requests.post(
                AUTH_LOGIN_URL,
                json={
                    "email": email,
                    "password": password
                },
                headers={"Content-Type": "application/json"},
                timeout=30
            )
        
        if response.status_code == 200:
            data = response.json()
            
            # Store parent access token in memory/temp file (not on disk)
            parent_token = data.get("parent_access_token")
            if parent_token:
                # Store in globalstorage for next module
                libcalamares.globalstorage.insert("guardian_parent_token", parent_token)
                libcalamares.globalstorage.insert("guardian_parent_email", email)
                
                # Also write to temp file for guardian_claim to read
                temp_token_file = "/tmp/guardian_parent_token"
                with open(temp_token_file, "w") as f:
                    f.write(parent_token)
                os.chmod(temp_token_file, 0o600)
                
                debug("Authentication successful")
                return None
            else:
                error("No access token in response")
                return ("Authentication failed", "Invalid response from server")
        else:
            error(f"Authentication failed: {response.status_code}")
            error(f"Response: {response.text}")
            return ("Authentication failed", f"Server returned {response.status_code}")
            
    except requests.exceptions.RequestException as e:
        warning(f"Network error during authentication: {e}")
        
        # Store for offline activation
        libcalamares.globalstorage.insert("guardian_offline_mode", True)
        libcalamares.globalstorage.insert("guardian_pending_email", email)
        # Don't store password, just a marker
        libcalamares.globalstorage.insert("guardian_pending_auth", True)
        
        debug("Will activate after installation")
        return None
