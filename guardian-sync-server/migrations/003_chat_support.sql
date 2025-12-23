-- Guardian Sync Server - Chat Support Migration
-- Migration: 003_chat_support
-- Date: 2024-12-22
-- Description: Add tables for safe chat, friend approvals, and chat settings

-- ============================================
-- CHAT CHANNELS
-- ============================================

CREATE TABLE IF NOT EXISTS chat_channels (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    channel_uuid CHAR(36) NOT NULL UNIQUE,
    channel_type ENUM('family', 'party', 'direct') NOT NULL,
    name VARCHAR(255),
    family_id BIGINT UNSIGNED,
    created_by VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_family (family_id),
    INDEX idx_type (channel_type),
    FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE
);

-- ============================================
-- CHAT MESSAGES AUDIT (for parent visibility)
-- ============================================

CREATE TABLE IF NOT EXISTS chat_messages_audit (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    message_uuid CHAR(36) NOT NULL UNIQUE,
    channel_id BIGINT UNSIGNED NOT NULL,
    sender_uuid CHAR(36) NOT NULL,
    sender_name VARCHAR(255) NOT NULL,
    sender_age TINYINT UNSIGNED,
    recipient_uuid CHAR(36),
    recipient_name VARCHAR(255),
    
    -- Message content
    message_type ENUM('text', 'voice_phrase', 'emoji', 'image') NOT NULL DEFAULT 'text',
    content TEXT NOT NULL,
    voice_phrase_id VARCHAR(50),  -- For whitelist voice mode
    
    -- Safety metadata
    was_filtered BOOLEAN DEFAULT FALSE,
    was_blocked BOOLEAN DEFAULT FALSE,
    filter_reason VARCHAR(100),
    threat_level ENUM('none', 'low', 'medium', 'high', 'critical') DEFAULT 'none',
    
    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_channel_time (channel_id, created_at),
    INDEX idx_sender (sender_uuid, created_at),
    INDEX idx_blocked (sender_uuid, was_blocked),
    INDEX idx_threat (threat_level, created_at),
    FOREIGN KEY (channel_id) REFERENCES chat_channels(id) ON DELETE CASCADE
);

-- ============================================
-- CHAT ALERTS (forwarded to Supabase for parents)
-- ============================================

CREATE TABLE IF NOT EXISTS chat_alerts (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    alert_uuid CHAR(36) NOT NULL UNIQUE,
    family_id BIGINT UNSIGNED NOT NULL,
    child_uuid CHAR(36) NOT NULL,
    channel_id BIGINT UNSIGNED,
    message_id BIGINT UNSIGNED,
    
    -- Alert details
    alert_type ENUM('profanity', 'pii', 'predator', 'blocked_word', 'suspicious_pattern') NOT NULL,
    severity ENUM('low', 'medium', 'high', 'critical') NOT NULL,
    message_preview TEXT,
    other_user_uuid CHAR(36),
    other_user_name VARCHAR(255),
    
    -- Status
    synced_to_supabase BOOLEAN DEFAULT FALSE,
    synced_at DATETIME,
    acknowledged_by CHAR(36),
    acknowledged_at DATETIME,
    
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_family_unread (family_id, acknowledged_at),
    INDEX idx_sync_pending (synced_to_supabase),
    INDEX idx_severity (severity, created_at),
    FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE,
    FOREIGN KEY (channel_id) REFERENCES chat_channels(id) ON DELETE SET NULL,
    FOREIGN KEY (message_id) REFERENCES chat_messages_audit(id) ON DELETE SET NULL
);

-- ============================================
-- FRIEND REQUESTS & APPROVALS
-- ============================================

CREATE TABLE IF NOT EXISTS friend_requests (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    request_uuid CHAR(36) NOT NULL UNIQUE,
    family_id BIGINT UNSIGNED NOT NULL,
    child_uuid CHAR(36) NOT NULL,
    
    -- Friend info
    friend_uuid CHAR(36) NOT NULL,
    friend_username VARCHAR(255) NOT NULL,
    friend_platform VARCHAR(50) DEFAULT 'guardian',
    
    -- Status
    status ENUM('pending', 'approved', 'denied', 'blocked') DEFAULT 'pending',
    requires_parent_approval BOOLEAN DEFAULT TRUE,
    
    -- Timestamps
    requested_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    decided_at DATETIME,
    decided_by CHAR(36),
    
    -- Sync status
    synced_to_supabase BOOLEAN DEFAULT FALSE,
    synced_at DATETIME,
    
    INDEX idx_child (child_uuid, status),
    INDEX idx_pending (status, requires_parent_approval),
    INDEX idx_sync_pending (synced_to_supabase),
    UNIQUE KEY uk_child_friend (child_uuid, friend_uuid),
    FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE
);

-- ============================================
-- BLOCKED USERS (per child)
-- ============================================

CREATE TABLE IF NOT EXISTS blocked_users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    child_uuid CHAR(36) NOT NULL,
    blocked_uuid CHAR(36) NOT NULL,
    blocked_username VARCHAR(255),
    reason TEXT,
    blocked_by ENUM('child', 'parent', 'system') DEFAULT 'child',
    blocked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE KEY uk_child_blocked (child_uuid, blocked_uuid),
    INDEX idx_child (child_uuid)
);

-- ============================================
-- CHAT SETTINGS (per child)
-- ============================================

CREATE TABLE IF NOT EXISTS chat_settings (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    child_uuid CHAR(36) NOT NULL UNIQUE,
    
    -- Chat mode
    chat_enabled BOOLEAN DEFAULT TRUE,
    chat_mode ENUM('disabled', 'whitelist', 'filtered', 'monitored', 'standard') DEFAULT 'filtered',
    
    -- Voice settings
    voice_enabled BOOLEAN DEFAULT TRUE,
    voice_mode ENUM('disabled', 'phrases_only', 'filtered', 'standard') DEFAULT 'phrases_only',
    
    -- Restrictions
    family_chat_only BOOLEAN DEFAULT FALSE,
    require_friend_approval BOOLEAN DEFAULT TRUE,
    max_friends INTEGER DEFAULT 20,
    
    -- Custom filters
    custom_blocked_words TEXT,  -- JSON array
    custom_allowed_words TEXT,  -- JSON array (override for false positives)
    
    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- ============================================
-- VOICE PHRASE USAGE (for analytics)
-- ============================================

CREATE TABLE IF NOT EXISTS voice_phrase_usage (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    child_uuid CHAR(36) NOT NULL,
    phrase_id VARCHAR(50) NOT NULL,  -- e.g., 'good_game', 'follow_me'
    channel_id BIGINT UNSIGNED,
    used_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_child_time (child_uuid, used_at),
    INDEX idx_phrase (phrase_id),
    FOREIGN KEY (channel_id) REFERENCES chat_channels(id) ON DELETE SET NULL
);

-- ============================================
-- PROFANITY FILTER LOG (for ML improvement)
-- ============================================

CREATE TABLE IF NOT EXISTS filter_log (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    original_text TEXT NOT NULL,
    filtered_text TEXT,
    filter_type ENUM('profanity', 'pii', 'predator', 'custom') NOT NULL,
    action_taken ENUM('allowed', 'sanitized', 'blocked') NOT NULL,
    detection_method VARCHAR(100),  -- 'regex', 'ml_classifier', 'llm_analysis'
    confidence_score DECIMAL(3,2),
    false_positive_reported BOOLEAN DEFAULT FALSE,
    reviewed_by CHAR(36),
    review_result ENUM('correct', 'false_positive', 'false_negative'),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_type (filter_type, action_taken),
    INDEX idx_review (false_positive_reported, review_result)
);

-- ============================================
-- FUNCTIONS
-- ============================================

-- Get chat mode for age
DELIMITER //
CREATE FUNCTION IF NOT EXISTS get_chat_mode_for_age(age INT) 
RETURNS VARCHAR(20)
DETERMINISTIC
BEGIN
    IF age < 10 THEN
        RETURN 'whitelist';
    ELSEIF age < 13 THEN
        RETURN 'filtered';
    ELSEIF age < 16 THEN
        RETURN 'monitored';
    ELSE
        RETURN 'standard';
    END IF;
END//
DELIMITER ;

-- Get voice mode for age
DELIMITER //
CREATE FUNCTION IF NOT EXISTS get_voice_mode_for_age(age INT)
RETURNS VARCHAR(20)
DETERMINISTIC
BEGIN
    IF age < 10 THEN
        RETURN 'phrases_only';
    ELSEIF age < 13 THEN
        RETURN 'filtered';
    ELSEIF age < 16 THEN
        RETURN 'filtered';
    ELSE
        RETURN 'standard';
    END IF;
END//
DELIMITER ;

-- ============================================
-- TRIGGERS
-- ============================================

-- Auto-create chat settings when child is created
DELIMITER //
CREATE TRIGGER IF NOT EXISTS create_chat_settings_for_child
AFTER INSERT ON children
FOR EACH ROW
BEGIN
    DECLARE child_age INT;
    
    -- Calculate age
    SET child_age = TIMESTAMPDIFF(YEAR, NEW.date_of_birth, CURDATE());
    
    -- Create chat settings with age-appropriate defaults
    INSERT INTO chat_settings (
        child_uuid,
        chat_mode,
        voice_mode,
        family_chat_only,
        require_friend_approval
    ) VALUES (
        NEW.child_uuid,
        get_chat_mode_for_age(child_age),
        get_voice_mode_for_age(child_age),
        child_age < 10,  -- Family chat only for under 10
        child_age < 16   -- Require friend approval for under 16
    );
END//
DELIMITER ;

-- ============================================
-- VIEWS
-- ============================================

-- Recent chat activity per child (for parent dashboard)
CREATE OR REPLACE VIEW v_child_chat_activity AS
SELECT 
    c.child_uuid,
    c.name AS child_name,
    COUNT(m.id) AS total_messages_7d,
    SUM(CASE WHEN m.was_blocked THEN 1 ELSE 0 END) AS blocked_messages_7d,
    SUM(CASE WHEN m.threat_level IN ('high', 'critical') THEN 1 ELSE 0 END) AS high_risk_7d,
    COUNT(DISTINCT m.recipient_uuid) AS unique_contacts_7d,
    MAX(m.created_at) AS last_message_at
FROM children c
LEFT JOIN chat_messages_audit m ON c.child_uuid = m.sender_uuid 
    AND m.created_at > DATE_SUB(NOW(), INTERVAL 7 DAY)
GROUP BY c.child_uuid, c.name;

-- Pending friend requests per family
CREATE OR REPLACE VIEW v_pending_friend_requests AS
SELECT 
    fr.family_id,
    fr.child_uuid,
    c.name AS child_name,
    fr.friend_username,
    fr.friend_platform,
    fr.requested_at,
    DATEDIFF(NOW(), fr.requested_at) AS days_pending
FROM friend_requests fr
JOIN children c ON fr.child_uuid = c.child_uuid
WHERE fr.status = 'pending' AND fr.requires_parent_approval = TRUE
ORDER BY fr.requested_at DESC;
