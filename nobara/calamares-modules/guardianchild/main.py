#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# Guardian OS - Child Selection Module
# Calamares installer module for selecting which child uses this device
#
# SPDX-License-Identifier: GPL-3.0-or-later

import os
import json
from typing import Optional, Dict, Any, List

import libcalamares
from libcalamares.utils import gettext_path, gettext_languages

import gettext
_ = gettext.translation("calamares-python",
                        localedir=gettext_path(),
                        languages=gettext_languages(),
                        fallback=True).gettext

# Supabase configuration  
SUPABASE_URL = "https://gkyspvcafyttfhyjryyk.supabase.co"
SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjYxMDIzMzQsImV4cCI6MjA4MTY3ODMzNH0.Ns5N9Y9uZgWqdhnYiX5IrubOO-Xopl2urBDR1AVD7FI"


def fetch_children(token: str, user_id: str) -> Dict[str, Any]:
    """
    Fetch children for the authenticated parent.
    
    Returns dict with:
        - success: bool
        - children: list of child objects
        - error: str (if failure)
    """
    import urllib.request
    import urllib.error
    
    # First get the parent's family_id
    parent_url = f"{SUPABASE_URL}/rest/v1/parents?user_id=eq.{user_id}&select=family_id"
    
    headers = {
        "apikey": SUPABASE_ANON_KEY,
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    try:
        req = urllib.request.Request(parent_url, headers=headers)
        with urllib.request.urlopen(req, timeout=30) as response:
            parents = json.loads(response.read().decode('utf-8'))
            
            if not parents:
                return {"success": False, "error": "No parent profile found", "children": []}
            
            family_id = parents[0].get("family_id")
            if not family_id:
                return {"success": False, "error": "No family associated", "children": []}
            
            # Store family_id for later use
            libcalamares.globalstorage.insert("guardian_family_id", family_id)
            
            # Now fetch children for this family
            children_url = f"{SUPABASE_URL}/rest/v1/children?family_id=eq.{family_id}&select=id,name,age,gender,avatar_url"
            
            req = urllib.request.Request(children_url, headers=headers)
            with urllib.request.urlopen(req, timeout=30) as response:
                children = json.loads(response.read().decode('utf-8'))
                return {"success": True, "children": children}
                
    except urllib.error.HTTPError as e:
        error_body = e.read().decode('utf-8')
        try:
            error_json = json.loads(error_body)
            error_msg = error_json.get("message", str(e))
        except:
            error_msg = str(e)
        return {"success": False, "error": error_msg, "children": []}
    except Exception as e:
        return {"success": False, "error": str(e), "children": []}


def run():
    """
    Main entry point - validates child selection.
    Called by Calamares when moving to next step.
    """
    gs = libcalamares.globalstorage
    
    selected_child = gs.value("guardian_selected_child")
    demo_mode = gs.value("guardian_demo_mode")
    
    if demo_mode:
        libcalamares.utils.debug("Guardian: Running in demo mode")
        # Set demo child data
        gs.insert("guardian_selected_child", {
            "id": "demo-child",
            "name": "Demo User",
            "age": 10
        })
        gs.insert("guardian_child_name", "Demo User")
        return None
    
    if not selected_child:
        return (_("No child selected"),
                _("Please select which child will use this device."))
    
    # Validate the selection
    child_id = selected_child.get("id")
    child_name = selected_child.get("name")
    
    if not child_id or not child_name:
        return (_("Invalid selection"),
                _("The selected child data is incomplete. Please try again."))
    
    libcalamares.utils.debug(f"Guardian: Child selected - {child_name} ({child_id})")
    
    return None  # Success
