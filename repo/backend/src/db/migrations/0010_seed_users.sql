-- 0010_seed_users.sql
-- Bootstrap five role accounts so QA can log in immediately after `docker compose up`.
-- Each password_hash is a real Argon2id PHC string generated with the argon2 algorithm
-- (m=65536, t=3, p=4) — the rust `argon2` crate's PasswordHash::new parses this format
-- and verifies the configured plaintext from CLAUDE.md.
--
--   admin    -> ScholarAdmin2024!
--   curator  -> Scholar2024!
--   reviewer -> Scholar2024!
--   finance  -> Scholar2024!
--   store    -> Scholar2024!

INSERT OR IGNORE INTO users
    (id, username, password_hash, role, is_active, full_name, email, created_at, updated_at)
VALUES
    ('u-admin',    'admin',    '$argon2id$v=19$m=65536,t=3,p=4$vvQrXjaj9cl1HHfKTiEa+A$pBnCBaBY4D+BT0BzHW4T7oPvZzFOB/xp7VOe/PhrHOY', 'administrator',   1, 'System Administrator', 'admin@scholarvault.local',    datetime('now'), datetime('now')),
    ('u-curator',  'curator',  '$argon2id$v=19$m=65536,t=3,p=4$hjgbTUm8u1vZBx4mm5aXfQ$Xv91p5gqVQflwQY64z/ayzsZkYhZcsgLK2mLzEE8pOc', 'content_curator', 1, 'Content Curator',      'curator@scholarvault.local',  datetime('now'), datetime('now')),
    ('u-reviewer', 'reviewer', '$argon2id$v=19$m=65536,t=3,p=4$2FHXrF9P4ynr+J9LZjB9WQ$jlffWgWmsRYlPpI4nH6mhmfFKPum4mKJ44BLhxRWHNY', 'reviewer',        1, 'Outcome Reviewer',     'reviewer@scholarvault.local', datetime('now'), datetime('now')),
    ('u-finance',  'finance',  '$argon2id$v=19$m=65536,t=3,p=4$IgMLF92oG+58VQTaY1cHig$+4IbksD32mPw2KEP81bF/ea1WkNnHaXr9kZGGSDuapA', 'finance_manager', 1, 'Finance Manager',      'finance@scholarvault.local',  datetime('now'), datetime('now')),
    ('u-store',    'store',    '$argon2id$v=19$m=65536,t=3,p=4$GjLZ46fZfB1adjZpgqwlCw$dvAiAEFb48oWsZhEAa6q9z2A7cNIh5AogiojeGsAe1I', 'store_manager',   1, 'Store Manager',        'store@scholarvault.local',    datetime('now'), datetime('now'));

-- Seed a small starter category tree so the curator UI is non-empty on first login.
INSERT OR IGNORE INTO categories (id, name, parent_id, level, description, created_by, created_at, updated_at)
VALUES
    ('cat-root',       'All Knowledge',       NULL,        0, 'Top-level root category',     'u-admin', datetime('now'), datetime('now')),
    ('cat-mathematics','Mathematics',         'cat-root',  1, 'Mathematical concepts',       'u-admin', datetime('now'), datetime('now')),
    ('cat-physics',    'Physics',             'cat-root',  1, 'Physical sciences',           'u-admin', datetime('now'), datetime('now')),
    ('cat-algebra',    'Linear Algebra',      'cat-mathematics', 2, 'Vectors, matrices',      'u-admin', datetime('now'), datetime('now')),
    ('cat-calculus',   'Calculus',            'cat-mathematics', 2, 'Limits, derivatives',    'u-admin', datetime('now'), datetime('now'));

INSERT OR IGNORE INTO knowledge_points (id, category_id, title, content, difficulty, discrimination, tags, created_by, created_at, updated_at)
VALUES
    ('kp-001', 'cat-algebra',  'Matrix Multiplication',     'Row-by-column multiplication of two matrices.', 3, 0.42, '["matrix","algebra"]',     'u-curator', datetime('now'), datetime('now')),
    ('kp-002', 'cat-calculus', 'Chain Rule',                'Derivative of composed functions.',             4, 0.55, '["calculus","derivative"]','u-curator', datetime('now'), datetime('now')),
    ('kp-003', 'cat-physics',  'Newton''s Second Law',      'F = m * a',                                     2, 0.31, '["mechanics","newton"]',   'u-curator', datetime('now'), datetime('now'));

-- Seed a couple of products and a promotion so the store page renders real data.
INSERT OR IGNORE INTO products (id, name, description, price, stock_quantity, is_active, created_by, created_at)
VALUES
    ('prod-book-1', 'Linear Algebra Textbook',  'Course textbook for cat-algebra',  39.99, 25, 1, 'u-store', datetime('now')),
    ('prod-book-2', 'Physics Workbook',         'Practice problems for cat-physics',24.50, 40, 1, 'u-store', datetime('now')),
    ('prod-kit-1',  'Calculus Study Kit',       'Bundle of calculus material',      59.00, 12, 1, 'u-store', datetime('now'));

INSERT OR IGNORE INTO promotions
    (id, name, description, discount_value, discount_type, effective_from, effective_until, mutual_exclusion_group, priority, is_active, created_by, created_at)
VALUES
    ('promo-spring-10', 'Spring 10% Off', 'Site-wide percentage discount',  10.0, 'percent', '2024-01-01T00:00:00Z', '2099-12-31T23:59:59Z', 'site-wide', 10, 1, 'u-store', datetime('now')),
    ('promo-bundle-5',  'Bundle $5 Off',  'Flat $5 off any order',           5.0, 'fixed',   '2024-01-01T00:00:00Z', '2099-12-31T23:59:59Z', 'site-wide',  5, 1, 'u-store', datetime('now'));

-- A few fund_transactions so the finance dashboard has real data on first login.
INSERT OR IGNORE INTO fund_transactions (id, type, amount, category, description, budget_period, recorded_by, created_at)
VALUES
    ('fund-001', 'income',   1500.00, 'grants',     'Monthly research grant',   '2026-04', 'u-finance', datetime('now')),
    ('fund-002', 'income',    750.00, 'donations',  'Alumni donation',          '2026-04', 'u-finance', datetime('now')),
    ('fund-003', 'expense',   420.00, 'equipment',  'Lab equipment purchase',   '2026-04', 'u-finance', datetime('now')),
    ('fund-004', 'expense',   180.00, 'supplies',   'Office supplies',          '2026-04', 'u-finance', datetime('now'));

-- One member snapshot row so the analytics dashboard isn't empty.
INSERT OR IGNORE INTO member_snapshots (id, snapshot_date, total_members, new_members, churned_members, created_at)
VALUES
    ('snap-001', '2026-04-01', 240, 12, 4, datetime('now')),
    ('snap-002', '2026-03-01', 232, 18, 6, datetime('now')),
    ('snap-003', '2026-02-01', 220, 15, 3, datetime('now'));

INSERT OR IGNORE INTO event_participation (id, event_name, event_date, participant_count, category, created_at)
VALUES
    ('evt-001', 'Spring Research Symposium', '2026-03-15', 145, 'conference', datetime('now')),
    ('evt-002', 'Calculus Workshop',         '2026-03-22',  38, 'workshop',   datetime('now'));
