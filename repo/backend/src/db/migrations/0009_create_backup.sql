-- 0009_create_backup.sql
-- Records each created backup bundle. backup_type is daily or monthly per the
-- 30-daily / 12-monthly retention rule from CLAUDE.md.
CREATE TABLE IF NOT EXISTS backup_records (
    id           TEXT PRIMARY KEY NOT NULL,
    backup_type  TEXT NOT NULL CHECK(backup_type IN ('daily','monthly')),
    bundle_path  TEXT NOT NULL,
    sha256_hash  TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'complete',
    size_bytes   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL,
    expires_at   TEXT,
    restored_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_backup_type ON backup_records(backup_type);
CREATE INDEX IF NOT EXISTS idx_backup_created ON backup_records(created_at);
CREATE INDEX IF NOT EXISTS idx_backup_status ON backup_records(status);

-- Admin-tunable retention policy. Single-row table edited from the admin UI.
CREATE TABLE IF NOT EXISTS retention_policies (
    id                  TEXT PRIMARY KEY NOT NULL,
    daily_retention     INTEGER NOT NULL DEFAULT 30,
    monthly_retention   INTEGER NOT NULL DEFAULT 12,
    preserve_financial  INTEGER NOT NULL DEFAULT 1,
    preserve_ip         INTEGER NOT NULL DEFAULT 1,
    updated_at          TEXT NOT NULL,
    updated_by          TEXT
);

INSERT INTO retention_policies (id, daily_retention, monthly_retention, preserve_financial, preserve_ip, updated_at, updated_by)
VALUES ('default', 30, 12, 1, 1, datetime('now'), 'system');
