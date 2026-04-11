-- 0008_create_audit.sql
-- Append-only audit log. Notice: there is intentionally NO updated_at column,
-- and the AuditService implementation has only a `log()` method — there is no
-- update or delete code path against this table anywhere in the codebase.
CREATE TABLE IF NOT EXISTS audit_logs (
    id           TEXT PRIMARY KEY NOT NULL,
    actor_id     TEXT NOT NULL,
    action       TEXT NOT NULL,
    entity_type  TEXT,
    entity_id    TEXT,
    before_hash  TEXT,
    after_hash   TEXT,
    ip_address   TEXT,
    created_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at);
