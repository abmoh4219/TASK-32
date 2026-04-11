-- 0004_create_question_bank.sql
-- questions live alongside knowledge points. The link table is many-to-many.
CREATE TABLE IF NOT EXISTS questions (
    id                  TEXT PRIMARY KEY NOT NULL,
    knowledge_point_id  TEXT,
    question_text       TEXT NOT NULL,
    question_type       TEXT NOT NULL DEFAULT 'multiple_choice',
    options             TEXT NOT NULL DEFAULT '[]',
    correct_answer      TEXT NOT NULL DEFAULT '',
    explanation         TEXT,
    chapter             TEXT,
    created_by          TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    FOREIGN KEY (knowledge_point_id) REFERENCES knowledge_points(id)
);

CREATE INDEX IF NOT EXISTS idx_questions_kp ON questions(knowledge_point_id);
CREATE INDEX IF NOT EXISTS idx_questions_chapter ON questions(chapter);

CREATE TABLE IF NOT EXISTS knowledge_question_links (
    knowledge_point_id  TEXT NOT NULL,
    question_id         TEXT NOT NULL,
    linked_at           TEXT NOT NULL,
    PRIMARY KEY (knowledge_point_id, question_id),
    FOREIGN KEY (knowledge_point_id) REFERENCES knowledge_points(id),
    FOREIGN KEY (question_id) REFERENCES questions(id)
);
