-- 0001_create_users.sql
-- Core user table. Sensitive PII (phone, national id) are stored AES-256-GCM encrypted.
CREATE TABLE IF NOT EXISTS users (
    id                      TEXT PRIMARY KEY NOT NULL,
    username                TEXT UNIQUE NOT NULL,
    password_hash           TEXT NOT NULL,
    role                    TEXT NOT NULL,
    is_active               INTEGER NOT NULL DEFAULT 1,
    full_name               TEXT,
    email                   TEXT,
    phone_encrypted         TEXT,
    national_id_encrypted   TEXT,
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);
