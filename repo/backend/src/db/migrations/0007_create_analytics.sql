-- 0007_create_analytics.sql
-- Member, event, fund, approval-cycle telemetry plus scheduled report records.
CREATE TABLE IF NOT EXISTS member_snapshots (
    id               TEXT PRIMARY KEY NOT NULL,
    snapshot_date    TEXT NOT NULL,
    total_members    INTEGER NOT NULL DEFAULT 0,
    new_members      INTEGER NOT NULL DEFAULT 0,
    churned_members  INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_member_snapshots_date ON member_snapshots(snapshot_date);

CREATE TABLE IF NOT EXISTS event_participation (
    id                TEXT PRIMARY KEY NOT NULL,
    event_name        TEXT NOT NULL,
    event_date        TEXT NOT NULL,
    participant_count INTEGER NOT NULL DEFAULT 0,
    category          TEXT,
    created_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_event_date ON event_participation(event_date);
CREATE INDEX IF NOT EXISTS idx_event_category ON event_participation(category);

CREATE TABLE IF NOT EXISTS fund_transactions (
    id            TEXT PRIMARY KEY NOT NULL,
    type          TEXT NOT NULL CHECK(type IN ('income','expense')),
    amount        REAL NOT NULL CHECK(amount >= 0),
    category      TEXT NOT NULL DEFAULT 'general',
    description   TEXT NOT NULL DEFAULT '',
    budget_period TEXT NOT NULL,
    recorded_by   TEXT NOT NULL,
    created_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fund_period ON fund_transactions(budget_period);
CREATE INDEX IF NOT EXISTS idx_fund_type ON fund_transactions(type);

CREATE TABLE IF NOT EXISTS approval_cycle_records (
    id                  TEXT PRIMARY KEY NOT NULL,
    entity_type         TEXT NOT NULL,
    entity_id           TEXT NOT NULL,
    submitted_at        TEXT NOT NULL,
    approved_at         TEXT,
    approver_id         TEXT,
    cycle_time_minutes  INTEGER
);

CREATE INDEX IF NOT EXISTS idx_approval_entity ON approval_cycle_records(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS scheduled_reports (
    id              TEXT PRIMARY KEY NOT NULL,
    report_type     TEXT NOT NULL,
    filters         TEXT NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'pending',
    file_path       TEXT,
    download_token  TEXT UNIQUE,
    created_by      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    completed_at    TEXT
);

CREATE INDEX IF NOT EXISTS idx_reports_user ON scheduled_reports(created_by);
CREATE INDEX IF NOT EXISTS idx_reports_status ON scheduled_reports(status);
