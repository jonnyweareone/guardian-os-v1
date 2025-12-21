// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Guardian Authentication View
// Parent signs in with Guardian account before creating child user

public class Installer.GuardianAuthView : AbstractInstallerView {
    public signal void next_step ();
    public signal void auth_success (string access_token, string parent_id, string parent_email);

    private Gtk.Entry email_entry;
    private Gtk.Entry password_entry;
    private Gtk.Button sign_in_button;
    private Gtk.Button create_account_button;
    private Gtk.Label error_label;
    private Gtk.Revealer error_revealer;
    private Gtk.Spinner spinner;
    private Gtk.Stack mode_stack;
    
    private string? _access_token = null;
    private string? _parent_id = null;
    private string? _parent_email = null;

    public string? access_token { get { return _access_token; } }
    public string? parent_id { get { return _parent_id; } }
    public string? parent_email { get { return _parent_email; } }

    // Supabase configuration
    private const string SUPABASE_URL = "https://gkyspvcafyttfhyjryyk.supabase.co";
    private const string SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzI4MTI0NzgsImV4cCI6MjA0ODM4ODQ3OH0.HnWPpgU-fKhKZDpzQvPHMM3VGwY2L2rD-TbLIEJFEKI";

    public GuardianAuthView () {
        Object (cancellable: true);
    }

    construct {
        cancel_button.label = _("Back");

        var artwork = new Gtk.Grid () { vexpand = true };
        artwork.get_style_context ().add_class ("guardian-auth");
        artwork.get_style_context ().add_class ("artwork");

        var title_label = new Gtk.Label (_("Guardian Authentication")) { valign = Gtk.Align.START };
        title_label.get_style_context ().add_class ("h2");

        var description = new Gtk.Label (_("Sign in with your Guardian account to set up this device for your child."));
        description.wrap = true;
        description.max_width_chars = 50;
        description.margin_bottom = 24;

        // Email entry
        var email_label = new Granite.HeaderLabel (_("Email"));
        email_entry = new Gtk.Entry () {
            hexpand = true,
            placeholder_text = "parent@example.com"
        };
        email_entry.set_input_purpose (Gtk.InputPurpose.EMAIL);

        // Password entry
        var password_label = new Granite.HeaderLabel (_("Password"));
        password_entry = new Gtk.Entry () {
            hexpand = true,
            visibility = false,
            placeholder_text = "Enter password"
        };
        password_entry.set_input_purpose (Gtk.InputPurpose.PASSWORD);

        // Error display
        error_label = new Gtk.Label ("");
        error_label.get_style_context ().add_class (Gtk.STYLE_CLASS_ERROR);
        error_revealer = new Gtk.Revealer () {
            transition_type = Gtk.RevealerTransitionType.SLIDE_DOWN
        };
        error_revealer.add (error_label);

        // Spinner for loading state
        spinner = new Gtk.Spinner ();

        // Form container
        var form_box = new Gtk.Box (Gtk.Orientation.VERTICAL, 6) {
            valign = Gtk.Align.CENTER,
            margin = 24
        };
        form_box.add (description);
        form_box.add (email_label);
        form_box.add (email_entry);
        form_box.add (password_label);
        form_box.add (password_entry);
        form_box.add (error_revealer);

        content_area.attach (artwork, 0, 0, 1, 1);
        content_area.attach (title_label, 0, 1, 1, 1);
        content_area.attach (form_box, 1, 0, 1, 2);

        // Sign In button
        sign_in_button = new Gtk.Button.with_label (_("Sign In")) {
            can_default = true,
            sensitive = false
        };
        sign_in_button.get_style_context ().add_class (Gtk.STYLE_CLASS_SUGGESTED_ACTION);

        // Create Account link
        create_account_button = new Gtk.Button.with_label (_("Create Account"));
        create_account_button.get_style_context ().add_class (Gtk.STYLE_CLASS_FLAT);

        action_area.add (spinner);
        action_area.add (create_account_button);
        action_area.add (sign_in_button);

        // Event handlers
        email_entry.changed.connect (update_sign_in_button);
        password_entry.changed.connect (update_sign_in_button);
        
        email_entry.activate.connect (() => {
            password_entry.grab_focus ();
        });

        password_entry.activate.connect (() => {
            if (sign_in_button.sensitive) {
                sign_in_button.clicked ();
            }
        });

        sign_in_button.clicked.connect (do_sign_in);

        create_account_button.clicked.connect (() => {
            // Open browser to Guardian signup page
            try {
                AppInfo.launch_default_for_uri ("https://guardian.network/signup", null);
            } catch (Error e) {
                warning ("Failed to open signup URL: %s", e.message);
            }
        });

        show_all ();
        spinner.hide ();
    }

    private void update_sign_in_button () {
        bool email_valid = email_entry.text.contains ("@") && email_entry.text.contains (".");
        bool password_valid = password_entry.text.length >= 6;
        sign_in_button.sensitive = email_valid && password_valid;
    }

    private void show_error (string message) {
        error_label.label = message;
        error_revealer.reveal_child = true;
    }

    private void hide_error () {
        error_revealer.reveal_child = false;
    }

    private void set_loading (bool loading) {
        sign_in_button.sensitive = !loading;
        email_entry.sensitive = !loading;
        password_entry.sensitive = !loading;
        if (loading) {
            spinner.show ();
            spinner.start ();
        } else {
            spinner.stop ();
            spinner.hide ();
        }
    }

    private async void do_sign_in () {
        hide_error ();
        set_loading (true);

        string email = email_entry.text.strip ();
        string password = password_entry.text;

        try {
            // Call Supabase auth
            var session = new Soup.Session ();
            var message = new Soup.Message ("POST", SUPABASE_URL + "/auth/v1/token?grant_type=password");
            
            var json_body = """{"email":"%s","password":"%s"}""".printf (email, password);
            message.set_request ("application/json", Soup.MemoryUse.COPY, json_body.data);
            message.request_headers.append ("apikey", SUPABASE_ANON_KEY);
            message.request_headers.append ("Content-Type", "application/json");

            session.send_message (message);

            if (message.status_code == 200) {
                // Parse response
                var parser = new Json.Parser ();
                parser.load_from_data ((string) message.response_body.data);
                var root = parser.get_root ().get_object ();
                
                _access_token = root.get_string_member ("access_token");
                
                var user = root.get_object_member ("user");
                _parent_id = user.get_string_member ("id");
                _parent_email = user.get_string_member ("email");

                // Save credentials for guardian-daemon
                save_credentials ();

                set_loading (false);
                auth_success (_access_token, _parent_id, _parent_email);
                next_step ();
            } else if (message.status_code == 400) {
                set_loading (false);
                show_error (_("Invalid email or password. Please try again."));
            } else {
                set_loading (false);
                show_error (_("Authentication failed. Please check your internet connection."));
            }
        } catch (Error e) {
            set_loading (false);
            show_error (_("Network error: ") + e.message);
        }
    }

    private void save_credentials () {
        // Save to /etc/guardian/credentials for the daemon
        try {
            var dir = File.new_for_path ("/etc/guardian");
            if (!dir.query_exists ()) {
                dir.make_directory_with_parents ();
            }

            var file = File.new_for_path ("/etc/guardian/credentials");
            var stream = file.replace (null, false, FileCreateFlags.PRIVATE);
            var data = """[auth]
access_token=%s
parent_id=%s
parent_email=%s
""".printf (_access_token, _parent_id, _parent_email);
            stream.write (data.data);
            stream.close ();

            // Set restrictive permissions
            FileUtils.chmod ("/etc/guardian/credentials", 0600);
        } catch (Error e) {
            warning ("Failed to save credentials: %s", e.message);
        }
    }

    public new void grab_focus () {
        email_entry.grab_focus ();
    }

    public void reset () {
        email_entry.text = "";
        password_entry.text = "";
        hide_error ();
        set_loading (false);
        _access_token = null;
        _parent_id = null;
        _parent_email = null;
    }
}
