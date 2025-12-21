#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# Guardian OS - Parent Authentication Module
# Calamares installer module for authenticating parents before installation
#
# SPDX-License-Identifier: GPL-3.0-or-later

import os
import json
import subprocess
from typing import Optional, Dict, Any

import libcalamares
from libcalamares.utils import gettext_path, gettext_languages

import gettext
_ = gettext.translation("calamares-python",
                        localedir=gettext_path(),
                        languages=gettext_languages(),
                        fallback=True).gettext

# Supabase configuration
SUPABASE_URL = "https://gkyspvcafyttfhyjryyk.supabase.co"
SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzQ1NDc1NzIsImV4cCI6MjA1MDEyMzU3Mn0.dummy"


def authenticate_parent(email: str, password: str) -> Dict[str, Any]:
    """
    Authenticate parent with Supabase Auth.
    
    Returns dict with:
        - success: bool
        - token: str (if success)
        - user_id: str (if success)
        - error: str (if failure)
    """
    import urllib.request
    import urllib.error
    
    url = f"{SUPABASE_URL}/auth/v1/token?grant_type=password"
    
    data = json.dumps({
        "email": email,
        "password": password
    }).encode('utf-8')
    
    headers = {
        "Content-Type": "application/json",
        "apikey": SUPABASE_ANON_KEY
    }
    
    req = urllib.request.Request(url, data=data, headers=headers, method='POST')
    
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            result = json.loads(response.read().decode('utf-8'))
            return {
                "success": True,
                "token": result.get("access_token"),
                "user_id": result.get("user", {}).get("id"),
                "email": email
            }
    except urllib.error.HTTPError as e:
        error_body = e.read().decode('utf-8')
        try:
            error_json = json.loads(error_body)
            error_msg = error_json.get("error_description", error_json.get("message", str(e)))
        except:
            error_msg = str(e)
        return {"success": False, "error": error_msg}
    except Exception as e:
        return {"success": False, "error": str(e)}


def get_parent_profile(token: str, user_id: str) -> Dict[str, Any]:
    """
    Get parent profile and children from Supabase.
    """
    import urllib.request
    import urllib.error
    
    # Get parent record
    url = f"{SUPABASE_URL}/rest/v1/parents?user_id=eq.{user_id}&select=*,families(*)"
    
    headers = {
        "apikey": SUPABASE_ANON_KEY,
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            parents = json.loads(response.read().decode('utf-8'))
            if parents:
                return {"success": True, "parent": parents[0]}
            return {"success": False, "error": "No parent profile found"}
    except Exception as e:
        return {"success": False, "error": str(e)}


def get_children(token: str, family_id: str) -> Dict[str, Any]:
    """
    Get children for a family.
    """
    import urllib.request
    
    url = f"{SUPABASE_URL}/rest/v1/children?family_id=eq.{family_id}&select=*"
    
    headers = {
        "apikey": SUPABASE_ANON_KEY,
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            children = json.loads(response.read().decode('utf-8'))
            return {"success": True, "children": children}
    except Exception as e:
        return {"success": False, "error": str(e), "children": []}


def run():
    """
    Main entry point for the module.
    Called by Calamares when the module is executed.
    """
    # Get stored values from global storage
    gs = libcalamares.globalstorage
    
    auth_token = gs.value("guardian_auth_token")
    user_id = gs.value("guardian_user_id")
    parent_email = gs.value("guardian_parent_email")
    selected_child = gs.value("guardian_selected_child")
    family_id = gs.value("guardian_family_id")
    
    if not all([auth_token, user_id, selected_child, family_id]):
        return (_("Guardian authentication incomplete"), 
                _("Please complete parent authentication and child selection."))
    
    # Store configuration for post-install
    root_mount = gs.value("rootMountPoint")
    if root_mount:
        guardian_config_dir = os.path.join(root_mount, "etc", "guardian")
        os.makedirs(guardian_config_dir, mode=0o700, exist_ok=True)
        
        # Write device config
        config = {
            "family_id": family_id,
            "child_id": selected_child.get("id"),
            "child_name": selected_child.get("name"),
            "parent_email": parent_email,
            "registered_at": None,  # Will be set by daemon on first boot
            "api_url": SUPABASE_URL
        }
        
        config_path = os.path.join(guardian_config_dir, "device.json")
        with open(config_path, 'w') as f:
            json.dump(config, f, indent=2)
        os.chmod(config_path, 0o600)
        
        libcalamares.utils.debug(f"Guardian config written to {config_path}")
    
    return None  # Success
