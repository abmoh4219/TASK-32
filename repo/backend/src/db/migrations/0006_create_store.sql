-- 0006_create_store.sql
-- Storefront tables: products, promotions (with mutual exclusion groups + priority),
-- orders and order_items (each line carries the applied promotion trace as JSON).
CREATE TABLE IF NOT EXISTS products (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    price           REAL NOT NULL CHECK(price >= 0),
    stock_quantity  INTEGER NOT NULL DEFAULT 0,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_by      TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_products_active ON products(is_active);

CREATE TABLE IF NOT EXISTS promotions (
    id                       TEXT PRIMARY KEY NOT NULL,
    name                     TEXT NOT NULL,
    description              TEXT NOT NULL DEFAULT '',
    discount_value           REAL NOT NULL CHECK(discount_value >= 0),
    discount_type            TEXT NOT NULL CHECK(discount_type IN ('percent','fixed')),
    effective_from           TEXT NOT NULL,
    effective_until          TEXT NOT NULL,
    mutual_exclusion_group   TEXT,
    priority                 INTEGER NOT NULL DEFAULT 0,
    is_active                INTEGER NOT NULL DEFAULT 1,
    created_by               TEXT NOT NULL,
    created_at               TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_promotions_active ON promotions(is_active);
CREATE INDEX IF NOT EXISTS idx_promotions_window ON promotions(effective_from, effective_until);
CREATE INDEX IF NOT EXISTS idx_promotions_group ON promotions(mutual_exclusion_group);

CREATE TABLE IF NOT EXISTS orders (
    id                TEXT PRIMARY KEY NOT NULL,
    user_id           TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'completed',
    subtotal          REAL NOT NULL DEFAULT 0,
    discount_applied  REAL NOT NULL DEFAULT 0,
    total             REAL NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_orders_user ON orders(user_id);

CREATE TABLE IF NOT EXISTS order_items (
    id                  TEXT PRIMARY KEY NOT NULL,
    order_id            TEXT NOT NULL,
    product_id          TEXT NOT NULL,
    product_name        TEXT NOT NULL,
    quantity            INTEGER NOT NULL CHECK(quantity > 0),
    unit_price          REAL NOT NULL,
    discount_amount     REAL NOT NULL DEFAULT 0,
    promotion_applied   TEXT,
    promotion_trace     TEXT,
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (product_id) REFERENCES products(id)
);

CREATE INDEX IF NOT EXISTS idx_order_items_order ON order_items(order_id);
