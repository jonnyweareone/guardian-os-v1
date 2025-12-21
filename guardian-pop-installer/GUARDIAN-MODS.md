# Guardian OS pop-installer modifications

## Summary

This fork of pop-os/installer adds Guardian parent authentication and child selection
BEFORE the user account creation step, enabling parental control from the moment of installation.

## New Files Added

- `src/Views/GuardianAuthView.vala` - Parent authentication with Supabase
- `src/Views/GuardianChildView.vala` - Child profile selection/creation

## Modified Files

- `src/MainWindow.vala` - Added Guardian view declarations and load functions
- `src/meson.build` - Include new Vala source files

## Installation Flow (Guardian OS)

```
Original Pop!_OS:
Language → Keyboard → Try/Install → Disk → User → Encrypt → Install

Guardian OS:
Language → Keyboard → Try/Install → Disk → GUARDIAN AUTH → CHILD SELECT → User → Encrypt → Install
```

## Key Changes

### MainWindow.vala additions:

1. Add variable declarations:
```vala
// Guardian OS views
private GuardianAuthView guardian_auth_view;
private GuardianChildView guardian_child_view;
```

2. Add Guardian view loading functions:
```vala
private void load_guardian_auth_view (Gtk.Widget prev_view, Fn load_next_view) {
    if (guardian_auth_view == null) {
        guardian_auth_view = new GuardianAuthView ();
        
        guardian_auth_view.next_step.connect (() => {
            load_guardian_child_view (guardian_auth_view, load_next_view);
        });
        
        guardian_auth_view.auth_success.connect ((token, parent_id, email) => {
            // Pass auth context to child view
            if (guardian_child_view != null) {
                guardian_child_view.set_auth_context (token, parent_id);
            }
        });
        
        stack.add (guardian_auth_view);
    }
    
    guardian_auth_view.previous_view = prev_view;
    stack.visible_child = guardian_auth_view;
    guardian_auth_view.grab_focus ();
}

private void load_guardian_child_view (Gtk.Widget prev_view, Fn load_next_view) {
    if (guardian_child_view == null) {
        guardian_child_view = new GuardianChildView ();
        
        guardian_child_view.next_step.connect (() => load_next_view ());
        
        guardian_child_view.child_selected.connect ((child_id, child_name) => {
            // Pre-fill username from child name
            // This will be used in UserView
        });
        
        stack.add (guardian_child_view);
    }
    
    guardian_child_view.previous_view = prev_view;
    stack.visible_child = guardian_child_view;
    guardian_child_view.grab_focus ();
}
```

3. Modify load_disk_view to route through Guardian auth:
```vala
private void load_disk_view () {
    if (disk_view == null) {
        disk_view = new DiskView ();
        disk_view.cancel.connect (() => this.load_option_select_view());
        
        // GUARDIAN OS: Route through auth before user creation
        disk_view.next_step.connect (() => {
            load_guardian_auth_view (disk_view, () => {
                load_user_view (guardian_child_view, load_disk_view, load_encrypt_view);
            });
        });
        
        stack.add (disk_view);
    }
    // ... rest unchanged
}
```

### meson.build additions:

Add to sources list:
```meson
'src/Views/GuardianAuthView.vala',
'src/Views/GuardianChildView.vala',
```

## Building

```bash
meson setup build
cd build
ninja
```

## Dependencies

- libsoup-2.4 (for HTTP requests to Supabase)
- libjson-glib-1.0 (for JSON parsing)
- Standard Pop!_OS installer dependencies

## Testing

1. Build the modified installer
2. Boot Guardian OS ISO in VM
3. Click Install
4. Verify Guardian Auth screen appears after disk selection
5. Sign in with test Guardian account
6. Select/create child profile
7. Continue to user creation (should be pre-filled)
8. Complete installation
9. Verify /etc/guardian/ files created
