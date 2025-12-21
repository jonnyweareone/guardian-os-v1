// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Guardian Child Selection View
// Parent selects which child will use this device

public class Installer.GuardianChildView : AbstractInstallerView {
    public signal void next_step ();
    public signal void child_selected (string child_id, string child_name);

    private Gtk.ListBox children_list;
    private Gtk.Entry new_child_name;
    private Gtk.ComboBoxText age_group_combo;
    private Gtk.Button create_child_button;
    private Gtk.Button next_button;
    private Gtk.Label error_label;
    private Gtk.Revealer error_revealer;
    private Gtk.Spinner spinner;
    private Gtk.Stack mode_stack;

    private string? _access_token = null;
    private string? _parent_id = null;
    private string? _selected_child_id = null;
    private string? _selected_child_name = null;

    public string? selected_child_id { get { return _selected_child_id; } }
    public string? selected_child_name { get { return _selected_child_name; } }

    private const string SUPABASE_URL = "https://gkyspvcafyttfhyjryyk.supabase.co";
    private const string SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzI4MTI0NzgsImV4cCI6MjA0ODM4ODQ3OH0.HnWPpgU-fKhKZDpzQvPHMM3VGwY2L2rD-TbLIEJFEKI";

    public GuardianChildView () {
        Object (cancellable: true);
    }

    construct {
        cancel_button.label = _("Back");

        var artwork = new Gtk.Grid () { vexpand = true };
        artwork.get_style_context ().add_class ("guardian-child");
        artwork.get_style_context ().add_class ("artwork");

        var title_label = new Gtk.Label (_("Select Child Profile")) { valign = Gtk.Align.START };
        title_label.get_style_context ().add_class ("h2");

        var description = new Gtk.Label (_("Choose which child will use this device, or create a new profile."));
        description.wrap = true;
        description.max_width_chars = 50;
        description.margin_bottom = 12;

        // Existing children list
        var list_label = new Granite.HeaderLabel (_("Your Children"));
        
        children_list = new Gtk.ListBox () {
            selection_mode = Gtk.SelectionMode.SINGLE,
            activate_on_single_click = true
        };
        children_list.get_style_context ().add_class (Gtk.STYLE_CLASS_FRAME);

        var list_scroll = new Gtk.ScrolledWindow (null, null) {
            hscrollbar_policy = Gtk.PolicyType.NEVER,
            vscrollbar_policy = Gtk.PolicyType.AUTOMATIC,
            min_content_height = 150,
            max_content_height = 200
        };
        list_scroll.add (children_list);

        // Create new child section
        var create_label = new Granite.HeaderLabel (_("Or Create New Profile"));
        
        new_child_name = new Gtk.Entry () {
            placeholder_text = _("Child's name")
        };

        age_group_combo = new Gtk.ComboBoxText ();
        age_group_combo.append ("under-7", _("Under 7"));
        age_group_combo.append ("7-12", _("7-12 years"));
        age_group_combo.append ("13-17", _("13-17 years"));
        age_group_combo.active_id = "7-12";

        create_child_button = new Gtk.Button.with_label (_("Create Profile"));

        var create_box = new Gtk.Box (Gtk.Orientation.HORIZONTAL, 6);
        create_box.add (new_child_name);
        create_box.add (age_group_combo);
        create_box.add (create_child_button);

        // Error display
        error_label = new Gtk.Label ("");
        error_label.get_style_context ().add_class (Gtk.STYLE_CLASS_ERROR);
        error_revealer = new Gtk.Revealer () {
            transition_type = Gtk.RevealerTransitionType.SLIDE_DOWN
        };
        error_revealer.add (error_label);

        // Spinner
        spinner = new Gtk.Spinner ();

        // Form container
        var form_box = new Gtk.Box (Gtk.Orientation.VERTICAL, 6) {
            valign = Gtk.Align.CENTER,
            margin = 24
        };
        form_box.add (description);
        form_box.add (list_label);
        form_box.add (list_scroll);
        form_box.add (create_label);
        form_box.add (create_box);
        form_box.add (error_revealer);

        content_area.attach (artwork, 0, 0, 1, 1);
        content_area.attach (title_label, 0, 1, 1, 1);
        content_area.attach (form_box, 1, 0, 1, 2);

        // Next button
        next_button = new Gtk.Button.with_label (_("Link Device")) {
            can_default = true,
            sensitive = false
        };
        next_button.get_style_context ().add_class (Gtk.STYLE_CLASS_SUGGESTED_ACTION);

        action_area.add (spinner);
        action_area.add (next_button);

        // Event handlers
        children_list.row_selected.connect ((row) => {
            if (row != null) {
                var child_row = row as ChildListRow;
                if (child_row != null) {
                    _selected_child_id = child_row.child_id;
                    _selected_child_name = child_row.child_name;
                    next_button.sensitive = true;
                }
            }
        });

        new_child_name.changed.connect (() => {
            create_child_button.sensitive = new_child_name.text.strip ().length >= 2;
        });

        create_child_button.clicked.connect (create_new_child);
        create_child_button.sensitive = false;

        next_button.clicked.connect (claim_device);

        show_all ();
        spinner.hide ();
    }

    public void set_auth_context (string access_token, string parent_id) {
        _access_token = access_token;
        _parent_id = parent_id;
        load_children.begin ();
    }

    private void show_error (string message) {
        error_label.label = message;
        error_revealer.reveal_child = true;
    }

    private void hide_error () {
        error_revealer.reveal_child = false;
    }

    private void set_loading (bool loading) {
        next_button.sensitive = !loading && _selected_child_id != null;
        create_child_button.sensitive = !loading && new_child_name.text.strip ().length >= 2;
        children_list.sensitive = !loading;
        if (loading) {
            spinner.show ();
            spinner.start ();
        } else {
            spinner.stop ();
            spinner.hide ();
        }
    }

    private async void load_children () {
        if (_access_token == null || _parent_id == null) {
            return;
        }

        set_loading (true);
        hide_error ();

        // Clear existing rows
        children_list.foreach ((widget) => widget.destroy ());

        try {
            var session = new Soup.Session ();
            var url = SUPABASE_URL + "/rest/v1/children?parent_id=eq." + _parent_id + "&select=id,name,age_group,created_at";
            var message = new Soup.Message ("GET", url);
            
            message.request_headers.append ("apikey", SUPABASE_ANON_KEY);
            message.request_headers.append ("Authorization", "Bearer " + _access_token);

            session.send_message (message);

            if (message.status_code == 200) {
                var parser = new Json.Parser ();
                parser.load_from_data ((string) message.response_body.data);
                var array = parser.get_root ().get_array ();

                if (array.get_length () == 0) {
                    var empty_row = new Gtk.Label (_("No children found. Create a profile below."));
                    empty_row.margin = 12;
                    children_list.add (empty_row);
                } else {
                    for (int i = 0; i < array.get_length (); i++) {
                        var child = array.get_object_element (i);
                        var row = new ChildListRow (
                            child.get_string_member ("id"),
                            child.get_string_member ("name"),
                            child.get_string_member ("age_group")
                        );
                        children_list.add (row);
                    }
                }
                children_list.show_all ();
            } else {
                show_error (_("Failed to load children. Please try again."));
            }
        } catch (Error e) {
            show_error (_("Network error: ") + e.message);
        }

        set_loading (false);
    }

    private async void create_new_child () {
        string name = new_child_name.text.strip ();
        string age_group = age_group_combo.active_id;

        if (name.length < 2) {
            show_error (_("Please enter a valid name."));
            return;
        }

        set_loading (true);
        hide_error ();

        try {
            var session = new Soup.Session ();
            var message = new Soup.Message ("POST", SUPABASE_URL + "/rest/v1/children");
            
            var json_body = """{"parent_id":"%s","name":"%s","age_group":"%s"}""".printf (_parent_id, name, age_group);
            message.set_request ("application/json", Soup.MemoryUse.COPY, json_body.data);
            message.request_headers.append ("apikey", SUPABASE_ANON_KEY);
            message.request_headers.append ("Authorization", "Bearer " + _access_token);
            message.request_headers.append ("Content-Type", "application/json");
            message.request_headers.append ("Prefer", "return=representation");

            session.send_message (message);

            if (message.status_code == 201) {
                // Parse response to get new child ID
                var parser = new Json.Parser ();
                parser.load_from_data ((string) message.response_body.data);
                var array = parser.get_root ().get_array ();
                if (array.get_length () > 0) {
                    var child = array.get_object_element (0);
                    _selected_child_id = child.get_string_member ("id");
                    _selected_child_name = name;
                }

                new_child_name.text = "";
                yield load_children ();

                // Auto-select the new child
                if (_selected_child_id != null) {
                    children_list.foreach ((widget) => {
                        var row = widget as ChildListRow;
                        if (row != null && row.child_id == _selected_child_id) {
                            children_list.select_row (row);
                        }
                    });
                    next_button.sensitive = true;
                }
            } else {
                show_error (_("Failed to create profile. Please try again."));
            }
        } catch (Error e) {
            show_error (_("Network error: ") + e.message);
        }

        set_loading (false);
    }

    private async void claim_device () {
        if (_selected_child_id == null || _access_token == null) {
            show_error (_("Please select a child profile."));
            return;
        }

        set_loading (true);
        hide_error ();

        // Get device ID (or generate one)
        string device_id = GLib.Uuid.string_random ();
        string hostname = Environment.get_host_name ();

        try {
            var session = new Soup.Session ();
            var message = new Soup.Message ("POST", SUPABASE_URL + "/rest/v1/devices");
            
            var json_body = """{"id":"%s","child_id":"%s","parent_id":"%s","name":"%s","platform":"guardian-os","status":"pending"}""".printf (
                device_id, _selected_child_id, _parent_id, hostname
            );
            message.set_request ("application/json", Soup.MemoryUse.COPY, json_body.data);
            message.request_headers.append ("apikey", SUPABASE_ANON_KEY);
            message.request_headers.append ("Authorization", "Bearer " + _access_token);
            message.request_headers.append ("Content-Type", "application/json");

            session.send_message (message);

            if (message.status_code == 201 || message.status_code == 200) {
                // Save device and child info
                save_device_config (device_id);
                save_child_config ();

                child_selected (_selected_child_id, _selected_child_name);
                set_loading (false);
                next_step ();
            } else {
                set_loading (false);
                show_error (_("Failed to register device. Please try again."));
            }
        } catch (Error e) {
            set_loading (false);
            show_error (_("Network error: ") + e.message);
        }
    }

    private void save_device_config (string device_id) {
        try {
            var dir = File.new_for_path ("/etc/guardian");
            if (!dir.query_exists ()) {
                dir.make_directory_with_parents ();
            }

            var file = File.new_for_path ("/etc/guardian/device.conf");
            var stream = file.replace (null, false, FileCreateFlags.NONE);
            var data = """[device]
id=%s
name=%s
platform=guardian-os
registered=true
""".printf (device_id, Environment.get_host_name ());
            stream.write (data.data);
            stream.close ();
        } catch (Error e) {
            warning ("Failed to save device config: %s", e.message);
        }
    }

    private void save_child_config () {
        try {
            var file = File.new_for_path ("/etc/guardian/child.conf");
            var stream = file.replace (null, false, FileCreateFlags.NONE);
            var data = """[child]
id=%s
name=%s
""".printf (_selected_child_id, _selected_child_name);
            stream.write (data.data);
            stream.close ();
        } catch (Error e) {
            warning ("Failed to save child config: %s", e.message);
        }
    }

    public new void grab_focus () {
        children_list.grab_focus ();
    }

    public void reset () {
        hide_error ();
        new_child_name.text = "";
        _selected_child_id = null;
        _selected_child_name = null;
        next_button.sensitive = false;
        children_list.foreach ((widget) => widget.destroy ());
    }
}


// Child list row widget
public class ChildListRow : Gtk.ListBoxRow {
    public string child_id { get; construct; }
    public string child_name { get; construct; }
    public string age_group { get; construct; }

    public ChildListRow (string id, string name, string age) {
        Object (
            child_id: id,
            child_name: name,
            age_group: age
        );
    }

    construct {
        var icon = new Gtk.Image.from_icon_name ("avatar-default", Gtk.IconSize.DND);
        
        var name_label = new Gtk.Label (child_name) {
            halign = Gtk.Align.START,
            hexpand = true
        };
        name_label.get_style_context ().add_class ("h3");

        var age_label = new Gtk.Label (format_age_group (age_group)) {
            halign = Gtk.Align.END
        };
        age_label.get_style_context ().add_class (Gtk.STYLE_CLASS_DIM_LABEL);

        var box = new Gtk.Box (Gtk.Orientation.HORIZONTAL, 12) {
            margin = 6
        };
        box.add (icon);
        box.add (name_label);
        box.add (age_label);

        add (box);
        show_all ();
    }

    private string format_age_group (string age) {
        switch (age) {
            case "under-7":
                return _("Under 7");
            case "7-12":
                return _("7-12 years");
            case "13-17":
                return _("13-17 years");
            default:
                return age;
        }
    }
}
