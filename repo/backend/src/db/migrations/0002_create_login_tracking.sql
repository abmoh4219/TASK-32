-- 0002_create_login_tracking.sql
-- login_attempts powers the 5-failures-in-15-minutes account lockout check.
-- sessions stores active session tokens together with their CSRF token.
CREATE TABLE IF NOT EXISTS login_attempts (
    id            TEXT PRIMARY KEY NOT NULL,
    username      TEXT NOT NULL,
    ip_address    TEXT,
    attempted_at  TEXT NOT NULL,
    success       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_login_attempts_username ON login_attempts(username);
CREATE INDEX IF NOT EXISTS idx_login_attempts_ip ON login_attempts(ip_address);
CREATE INDEX IF NOT EXISTS idx_login_attempts_at ON login_attempts(attempted_at);

CREATE TABLE IF NOT EXISTS sessions (
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT NOT NULL,
    csrf_token   TEXT NOT NULL,
    ip_address   TEXT,
    user_agent   TEXT,
    expires_at   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
