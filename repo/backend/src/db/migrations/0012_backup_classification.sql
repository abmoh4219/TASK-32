-- 0012_backup_classification.sql
-- Replace filename-substring preservation heuristics with structured flags on
-- each backup record so the retention policy can keep audited financial/IP
-- artifacts based on explicit classification rather than path guessing.
ALTER TABLE backup_records ADD COLUMN contains_financial INTEGER NOT NULL DEFAULT 0;
ALTER TABLE backup_records ADD COLUMN contains_ip INTEGER NOT NULL DEFAULT 0;

-- Admin-configurable backup schedule. Single-row table seeded with the
-- CLAUDE.md default (02:00 daily — `0 0 2 * * *` in 6-field tokio_cron format).
CREATE TABLE IF NOT EXISTS backup_schedules (
    id          TEXT PRIMARY KEY NOT NULL,
    cron_expr   TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    updated_by  TEXT
);

INSERT OR IGNORE INTO backup_schedules (id, cron_expr, updated_at, updated_by)
VALUES ('default', '0 0 2 * * *', datetime('now'), 'system');
