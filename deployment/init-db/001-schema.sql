-- ============================================
-- GUARDIAN SYNC SERVER - DATABASE SCHEMA
-- Extends cosmic-sync-server with Guardian OS
-- parental control features
-- ============================================

SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ============================================
-- COSMIC SYNC TABLES (from cosmic-sync-server)
-- ============================================

-- Accounts (parents/users)
CREATE TABLE IF NOT EXISTS accounts (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  account_hash VARCHAR(255) NOT NULL UNIQUE,
  email VARCHAR(255) NOT NULL UNIQUE,
  name VARCHAR(255),
  avatar_url TEXT,
  oauth_provider VARCHAR(50),
  oauth_id VARCHAR(255),
  plan_tier VARCHAR(32) DEFAULT 'free',
  subscription_status ENUM('inactive','active','past_due','cancelled') DEFAULT 'inactive',
  storage_bytes_limit BIGINT UNSIGNED DEFAULT 5242880,
  storage_bytes_soft_limit BIGINT UNSIGNED DEFAULT 4194304,
  bandwidth_monthly_limit BIGINT UNSIGNED DEFAULT 10485760,
  bandwidth_monthly_soft_limit BIGINT UNSIGNED DEFAULT 8388608,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_email (email),
  INDEX idx_oauth (oauth_provider, oauth_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Auth tokens
CREATE TABLE IF NOT EXISTS auth_tokens (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  token VARCHAR(512) NOT NULL UNIQUE,
  account_hash VARCHAR(255) NOT NULL,
  device_hash VARCHAR(255),
  expires_at DATETIME NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_account (account_hash),
  INDEX idx_expires (expires_at),
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Devices (desktops running Guardian OS)
CREATE TABLE IF NOT EXISTS devices (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  device_hash VARCHAR(255) NOT NULL,
  account_hash VARCHAR(255) NOT NULL,
  device_name VARCHAR(255),
  device_type VARCHAR(50) DEFAULT 'desktop',
  os_version VARCHAR(100),
  last_seen_at DATETIME,
  ip_address VARCHAR(45),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_device_account (device_hash, account_hash),
  INDEX idx_account (account_hash),
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Files (synced settings, themes, etc)
CREATE TABLE IF NOT EXISTS files (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  file_hash VARCHAR(255) NOT NULL,
  account_hash VARCHAR(255) NOT NULL,
  device_hash VARCHAR(255),
  file_path VARCHAR(1024) NOT NULL,
  file_name VARCHAR(255) NOT NULL,
  file_size BIGINT UNSIGNED DEFAULT 0,
  mime_type VARCHAR(100),
  encryption_key_hash VARCHAR(255),
  group_id INT DEFAULT 0,
  watcher_id INT DEFAULT 0,
  revision BIGINT DEFAULT 1,
  is_deleted BOOLEAN DEFAULT FALSE,
  upload_time BIGINT,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_account (account_hash),
  INDEX idx_path (account_hash, file_path(255)),
  INDEX idx_group (account_hash, group_id),
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Watcher groups (sync folders)
CREATE TABLE IF NOT EXISTS watcher_groups (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  group_id INT NOT NULL,
  account_hash VARCHAR(255) NOT NULL,
  device_hash VARCHAR(255),
  title VARCHAR(255),
  is_active BOOLEAN DEFAULT TRUE,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_group_account (group_id, account_hash),
  INDEX idx_account (account_hash),
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Encryption keys
CREATE TABLE IF NOT EXISTS encryption_keys (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  account_hash VARCHAR(255) NOT NULL UNIQUE,
  encryption_key TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- GUARDIAN OS TABLES (parental controls)
-- ============================================

-- Families (household grouping)
CREATE TABLE IF NOT EXISTS families (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  family_uuid CHAR(36) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  created_by VARCHAR(255) NOT NULL,
  invite_code VARCHAR(8) UNIQUE,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_created_by (created_by),
  FOREIGN KEY (created_by) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Family members (link parents to family)
CREATE TABLE IF NOT EXISTS family_members (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  family_id BIGINT UNSIGNED NOT NULL,
  account_hash VARCHAR(255) NOT NULL,
  role ENUM('owner','parent','guardian') DEFAULT 'parent',
  joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_family_account (family_id, account_hash),
  FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE,
  FOREIGN KEY (account_hash) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Children (managed by parents, no auth)
CREATE TABLE IF NOT EXISTS children (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_uuid CHAR(36) NOT NULL UNIQUE,
  family_id BIGINT UNSIGNED NOT NULL,
  name VARCHAR(255) NOT NULL,
  date_of_birth DATE,
  avatar_url TEXT,
  pin_hash VARCHAR(255),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_family (family_id),
  FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Child-Device assignment
CREATE TABLE IF NOT EXISTS child_devices (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_id BIGINT UNSIGNED NOT NULL,
  device_id BIGINT UNSIGNED NOT NULL,
  is_primary BOOLEAN DEFAULT FALSE,
  assigned_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_child_device (child_id, device_id),
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE CASCADE,
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Screen time policies
CREATE TABLE IF NOT EXISTS screen_time_policies (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_id BIGINT UNSIGNED NOT NULL UNIQUE,
  weekday_limit_mins INT DEFAULT 120,
  weekend_limit_mins INT DEFAULT 180,
  earliest_start TIME DEFAULT '07:00:00',
  latest_end TIME DEFAULT '21:00:00',
  bedtime_enabled BOOLEAN DEFAULT TRUE,
  bedtime_time TIME DEFAULT '20:30:00',
  bedtime_grace_mins INT DEFAULT 5,
  break_reminder_enabled BOOLEAN DEFAULT TRUE,
  break_after_mins INT DEFAULT 45,
  break_duration_mins INT DEFAULT 10,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- App policies
CREATE TABLE IF NOT EXISTS app_policies (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_id BIGINT UNSIGNED NOT NULL,
  app_id VARCHAR(255) NOT NULL,
  app_name VARCHAR(255),
  policy ENUM('allowed','blocked','time_limited','ask_parent') DEFAULT 'allowed',
  daily_limit_mins INT,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_child_app (child_id, app_id),
  INDEX idx_child (child_id),
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- DNS/Content filtering profiles
CREATE TABLE IF NOT EXISTS dns_profiles (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_id BIGINT UNSIGNED NOT NULL UNIQUE,
  filter_level ENUM('off','adult','teen','child','strict') DEFAULT 'child',
  block_adult BOOLEAN DEFAULT TRUE,
  block_gambling BOOLEAN DEFAULT TRUE,
  block_social_media BOOLEAN DEFAULT FALSE,
  block_gaming BOOLEAN DEFAULT FALSE,
  block_streaming BOOLEAN DEFAULT FALSE,
  blocked_domains JSON,
  allowed_domains JSON,
  enforce_safe_search BOOLEAN DEFAULT TRUE,
  enforce_youtube_restricted BOOLEAN DEFAULT TRUE,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- App sessions (activity logging)
CREATE TABLE IF NOT EXISTS app_sessions (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  device_id BIGINT UNSIGNED NOT NULL,
  child_id BIGINT UNSIGNED,
  app_id VARCHAR(255) NOT NULL,
  app_name VARCHAR(255),
  started_at DATETIME NOT NULL,
  ended_at DATETIME,
  duration_secs INT,
  session_date DATE NOT NULL,
  INDEX idx_device (device_id),
  INDEX idx_child (child_id),
  INDEX idx_date (session_date),
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- URL logs
CREATE TABLE IF NOT EXISTS url_logs (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  device_id BIGINT UNSIGNED NOT NULL,
  child_id BIGINT UNSIGNED,
  url TEXT NOT NULL,
  domain VARCHAR(255) NOT NULL,
  title VARCHAR(500),
  visited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  duration_secs INT,
  category VARCHAR(100),
  risk_score FLOAT,
  flagged BOOLEAN DEFAULT FALSE,
  INDEX idx_device (device_id),
  INDEX idx_child (child_id),
  INDEX idx_domain (domain),
  INDEX idx_flagged (flagged),
  INDEX idx_visited (visited_at),
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Daily screen time aggregates
CREATE TABLE IF NOT EXISTS screen_time_daily (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  child_id BIGINT UNSIGNED NOT NULL,
  device_id BIGINT UNSIGNED,
  date DATE NOT NULL,
  total_mins INT DEFAULT 0,
  gaming_mins INT DEFAULT 0,
  education_mins INT DEFAULT 0,
  entertainment_mins INT DEFAULT 0,
  social_mins INT DEFAULT 0,
  productivity_mins INT DEFAULT 0,
  other_mins INT DEFAULT 0,
  top_apps JSON,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_child_device_date (child_id, device_id, date),
  INDEX idx_child (child_id),
  INDEX idx_date (date),
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE CASCADE,
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Alerts
CREATE TABLE IF NOT EXISTS alerts (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  alert_uuid CHAR(36) NOT NULL UNIQUE,
  family_id BIGINT UNSIGNED NOT NULL,
  child_id BIGINT UNSIGNED,
  device_id BIGINT UNSIGNED,
  alert_type ENUM(
    'inappropriate_content','cyberbullying','self_harm',
    'predator_risk','screen_time_exceeded','blocked_attempt',
    'new_contact','location_alert','device_tamper'
  ) NOT NULL,
  severity ENUM('low','medium','high','critical') DEFAULT 'medium',
  title VARCHAR(255) NOT NULL,
  description TEXT,
  evidence JSON,
  status ENUM('new','viewed','actioned','dismissed') DEFAULT 'new',
  viewed_at DATETIME,
  viewed_by VARCHAR(255),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_family (family_id),
  INDEX idx_child (child_id),
  INDEX idx_status (status),
  INDEX idx_severity (severity),
  FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE,
  FOREIGN KEY (child_id) REFERENCES children(id) ON DELETE SET NULL,
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE SET NULL,
  FOREIGN KEY (viewed_by) REFERENCES accounts(account_hash) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Alert actions
CREATE TABLE IF NOT EXISTS alert_actions (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  alert_id BIGINT UNSIGNED NOT NULL,
  action_by VARCHAR(255) NOT NULL,
  action ENUM('acknowledge','dismiss','block_app','block_domain','restrict_device','contact_child','escalate') NOT NULL,
  notes TEXT,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (alert_id) REFERENCES alerts(id) ON DELETE CASCADE,
  FOREIGN KEY (action_by) REFERENCES accounts(account_hash) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Device commands queue
CREATE TABLE IF NOT EXISTS device_commands (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  command_uuid CHAR(36) NOT NULL UNIQUE,
  device_id BIGINT UNSIGNED NOT NULL,
  command ENUM('lock','unlock','shutdown','message','update_policies','screenshot') NOT NULL,
  payload JSON,
  issued_by VARCHAR(255),
  issued_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  acknowledged_at DATETIME,
  executed_at DATETIME,
  result JSON,
  INDEX idx_device (device_id),
  INDEX idx_pending (device_id, executed_at),
  FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
  FOREIGN KEY (issued_by) REFERENCES accounts(account_hash) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- App catalog (reference data)
CREATE TABLE IF NOT EXISTS app_catalog (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  app_id VARCHAR(255) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  icon_url TEXT,
  category VARCHAR(100),
  age_rating VARCHAR(10),
  guardian_approved BOOLEAN DEFAULT FALSE,
  guardian_notes TEXT,
  flathub_url TEXT,
  developer VARCHAR(255),
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_category (category),
  INDEX idx_approved (guardian_approved)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- TRIGGERS
-- ============================================

-- Auto-create screen_time_policy when child is created
DELIMITER $$
CREATE TRIGGER trg_child_after_insert
AFTER INSERT ON children
FOR EACH ROW
BEGIN
  INSERT INTO screen_time_policies (child_id) VALUES (NEW.id);
  INSERT INTO dns_profiles (child_id) VALUES (NEW.id);
END$$
DELIMITER ;

-- Auto-generate invite code for family
DELIMITER $$
CREATE TRIGGER trg_family_before_insert
BEFORE INSERT ON families
FOR EACH ROW
BEGIN
  IF NEW.invite_code IS NULL THEN
    SET NEW.invite_code = UPPER(SUBSTRING(MD5(RAND()), 1, 8));
  END IF;
  IF NEW.family_uuid IS NULL THEN
    SET NEW.family_uuid = UUID();
  END IF;
END$$
DELIMITER ;

-- ============================================
-- INITIAL DATA
-- ============================================

-- Insert some default app catalog entries
INSERT INTO app_catalog (app_id, name, category, age_rating, guardian_approved, guardian_notes) VALUES
('org.mozilla.firefox', 'Firefox', 'productivity', '3', TRUE, 'Safe browser with parental controls'),
('org.libreoffice.LibreOffice', 'LibreOffice', 'productivity', '3', TRUE, 'Office suite'),
('com.spotify.Client', 'Spotify', 'entertainment', '12', TRUE, 'Music streaming'),
('com.valvesoftware.Steam', 'Steam', 'gaming', '16', FALSE, 'Game store - requires parent approval'),
('org.gnome.Calculator', 'Calculator', 'utility', '3', TRUE, 'Basic calculator'),
('org.gnome.Calendar', 'Calendar', 'productivity', '3', TRUE, 'Calendar app'),
('com.discordapp.Discord', 'Discord', 'social', '13', FALSE, 'Chat app - monitor usage'),
('org.videolan.VLC', 'VLC', 'entertainment', '3', TRUE, 'Media player'),
('org.gimp.GIMP', 'GIMP', 'productivity', '3', TRUE, 'Image editor'),
('org.blender.Blender', 'Blender', 'productivity', '3', TRUE, '3D modeling')
ON DUPLICATE KEY UPDATE updated_at = NOW();
