-- 0011_backup_artifact_kind.sql
-- Additive: split backup artifacts into independent versioned bundles.
-- 'database' → scholarvault.db only
-- 'files'    → uploaded evidence directory only
-- Existing rows keep NULL and are treated as legacy combined bundles by the
-- service layer (read-only; new backups always produce a pair of rows).
ALTER TABLE backup_records ADD COLUMN artifact_kind TEXT;

CREATE INDEX IF NOT EXISTS idx_backup_artifact_kind ON backup_records(artifact_kind);
