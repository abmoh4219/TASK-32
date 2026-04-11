-- 0003_create_knowledge.sql
-- categories form a DAG (parent_id may be NULL for roots). Cycle detection
-- happens at the service layer via DFS.
CREATE TABLE IF NOT EXISTS categories (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    parent_id    TEXT,
    level        INTEGER NOT NULL DEFAULT 0,
    description  TEXT,
    created_by   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    deleted_at   TEXT,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_active ON categories(deleted_at);

-- knowledge_points belong to a category and carry difficulty + discrimination metrics.
CREATE TABLE IF NOT EXISTS knowledge_points (
    id              TEXT PRIMARY KEY NOT NULL,
    category_id     TEXT NOT NULL,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL DEFAULT '',
    difficulty      INTEGER NOT NULL CHECK(difficulty BETWEEN 1 AND 5),
    discrimination  REAL NOT NULL CHECK(discrimination BETWEEN -1.0 AND 1.0),
    tags            TEXT NOT NULL DEFAULT '[]',
    created_by      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE INDEX IF NOT EXISTS idx_kp_category ON knowledge_points(category_id);
CREATE INDEX IF NOT EXISTS idx_kp_difficulty ON knowledge_points(difficulty);
CREATE INDEX IF NOT EXISTS idx_kp_discrimination ON knowledge_points(discrimination);
