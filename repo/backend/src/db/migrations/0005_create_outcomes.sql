-- 0005_create_outcomes.sql
-- outcomes (paper/patent/competition_result/software_copyright) plus contributors
-- (whole-percent shares summing to exactly 100) and evidence files (encrypted on disk).
CREATE TABLE IF NOT EXISTS outcomes (
    id                  TEXT PRIMARY KEY NOT NULL,
    type                TEXT NOT NULL CHECK(type IN ('paper','patent','competition_result','software_copyright')),
    title               TEXT NOT NULL,
    abstract_snippet    TEXT NOT NULL DEFAULT '',
    certificate_number  TEXT,
    status              TEXT NOT NULL DEFAULT 'draft',
    submitted_at        TEXT,
    approved_at         TEXT,
    rejected_at         TEXT,
    rejection_reason    TEXT,
    approver_id         TEXT,
    created_by          TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_outcomes_type ON outcomes(type);
CREATE INDEX IF NOT EXISTS idx_outcomes_status ON outcomes(status);
CREATE INDEX IF NOT EXISTS idx_outcomes_certificate ON outcomes(certificate_number);

CREATE TABLE IF NOT EXISTS outcome_contributors (
    id                TEXT PRIMARY KEY NOT NULL,
    outcome_id        TEXT NOT NULL,
    user_id           TEXT NOT NULL,
    share_percentage  INTEGER NOT NULL CHECK(share_percentage BETWEEN 0 AND 100),
    role_in_work      TEXT,
    created_at        TEXT NOT NULL,
    FOREIGN KEY (outcome_id) REFERENCES outcomes(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_contrib_outcome ON outcome_contributors(outcome_id);
CREATE INDEX IF NOT EXISTS idx_contrib_user ON outcome_contributors(user_id);

CREATE TABLE IF NOT EXISTS evidence_files (
    id                  TEXT PRIMARY KEY NOT NULL,
    outcome_id          TEXT NOT NULL,
    filename            TEXT NOT NULL,
    mime_type           TEXT NOT NULL,
    stored_path         TEXT NOT NULL,
    file_size           INTEGER NOT NULL,
    sha256_fingerprint  TEXT NOT NULL UNIQUE,
    uploaded_by         TEXT NOT NULL,
    uploaded_at         TEXT NOT NULL,
    FOREIGN KEY (outcome_id) REFERENCES outcomes(id)
);

CREATE INDEX IF NOT EXISTS idx_evidence_outcome ON evidence_files(outcome_id);
