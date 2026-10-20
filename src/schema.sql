PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS schema_version(version INTEGER PRIMARY KEY);
INSERT OR IGNORE INTO schema_version VALUES(1);
CREATE TABLE IF NOT EXISTS users(
 id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL COLLATE NOCASE UNIQUE,
 password_hash TEXT NOT NULL, role TEXT NOT NULL CHECK(role IN ('admin','volunteer')), active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)));
CREATE TABLE IF NOT EXISTS sessions(
 token_hash TEXT PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), csrf TEXT NOT NULL,
 expires_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS login_attempts(key TEXT PRIMARY KEY, failures INTEGER NOT NULL DEFAULT 0, locked_until INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS cycles(
 id INTEGER PRIMARY KEY, name TEXT NOT NULL, start_month TEXT NOT NULL UNIQUE,
 capacity INTEGER NOT NULL DEFAULT 100 CHECK(capacity BETWEEN 2 AND 1000));
CREATE TABLE IF NOT EXISTS families(
 id INTEGER PRIMARY KEY, name TEXT NOT NULL, reference TEXT NOT NULL COLLATE NOCASE UNIQUE,
 members INTEGER NOT NULL CHECK(members BETWEEN 1 AND 30), created_at TEXT NOT NULL DEFAULT(datetime('now')));
CREATE TABLE IF NOT EXISTS enrollments(
 id INTEGER PRIMARY KEY, family_id INTEGER NOT NULL REFERENCES families(id), cycle_id INTEGER NOT NULL REFERENCES cycles(id),
 group_code TEXT NOT NULL CHECK(group_code IN ('A','B')), monthly_credit INTEGER NOT NULL CHECK(monthly_credit>0 AND monthly_credit<=100000000),
 criteria TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)), UNIQUE(family_id,cycle_id));
CREATE TABLE IF NOT EXISTS products(
 id INTEGER PRIMARY KEY, name TEXT NOT NULL COLLATE NOCASE UNIQUE, unit TEXT NOT NULL,
 minimum INTEGER NOT NULL CHECK(minimum>=0), active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)));
CREATE TABLE IF NOT EXISTS prices(
 id INTEGER PRIMARY KEY, product_id INTEGER NOT NULL REFERENCES products(id), effective_date TEXT NOT NULL,
 credits INTEGER NOT NULL CHECK(credits>0 AND credits<=100000000), UNIQUE(product_id,effective_date));
CREATE TABLE IF NOT EXISTS donations(
 id INTEGER PRIMARY KEY, donor TEXT NOT NULL, received_on TEXT NOT NULL, user_id INTEGER NOT NULL REFERENCES users(id),
 created_at TEXT NOT NULL DEFAULT(datetime('now')));
CREATE TABLE IF NOT EXISTS lots(
 id INTEGER PRIMARY KEY, donation_id INTEGER NOT NULL REFERENCES donations(id), product_id INTEGER NOT NULL REFERENCES products(id),
 code TEXT NOT NULL, expires_on TEXT NOT NULL, received INTEGER NOT NULL CHECK(received>0),
 remaining INTEGER NOT NULL CHECK(remaining>=0 AND remaining<=received), value_cents INTEGER NOT NULL CHECK(value_cents>=0));
CREATE TABLE IF NOT EXISTS checkouts(
 id INTEGER PRIMARY KEY, enrollment_id INTEGER NOT NULL REFERENCES enrollments(id), month TEXT NOT NULL,
 total_credits INTEGER NOT NULL CHECK(total_credits>0), user_id INTEGER NOT NULL REFERENCES users(id),
 request_key TEXT NOT NULL UNIQUE, created_at TEXT NOT NULL DEFAULT(datetime('now')), UNIQUE(enrollment_id,month));
CREATE TABLE IF NOT EXISTS checkout_items(
 id INTEGER PRIMARY KEY, checkout_id INTEGER NOT NULL REFERENCES checkouts(id), product_id INTEGER NOT NULL REFERENCES products(id),
 price_id INTEGER NOT NULL REFERENCES prices(id), quantity INTEGER NOT NULL CHECK(quantity>0), unit_credits INTEGER NOT NULL CHECK(unit_credits>0),
 UNIQUE(checkout_id,product_id));
CREATE TABLE IF NOT EXISTS allocations(
 id INTEGER PRIMARY KEY, checkout_item_id INTEGER NOT NULL REFERENCES checkout_items(id), lot_id INTEGER NOT NULL REFERENCES lots(id),
 quantity INTEGER NOT NULL CHECK(quantity>0));
CREATE TABLE IF NOT EXISTS stock_movements(
 id INTEGER PRIMARY KEY, lot_id INTEGER NOT NULL REFERENCES lots(id), checkout_item_id INTEGER REFERENCES checkout_items(id),
 kind TEXT NOT NULL CHECK(kind IN ('donation','checkout','discard')), quantity INTEGER NOT NULL CHECK(quantity<>0),
 reason TEXT NOT NULL, user_id INTEGER NOT NULL REFERENCES users(id), created_at TEXT NOT NULL DEFAULT(datetime('now')),
 CHECK((kind='donation' AND quantity>0) OR (kind IN ('checkout','discard') AND quantity<0)));
CREATE TABLE IF NOT EXISTS audit_log(
 id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), action TEXT NOT NULL, entity TEXT NOT NULL,
 entity_id INTEGER NOT NULL, detail TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT(datetime('now')));
CREATE INDEX IF NOT EXISTS idx_lots_fefo ON lots(product_id,expires_on,remaining);
CREATE INDEX IF NOT EXISTS idx_checkouts_month ON checkouts(month);
CREATE INDEX IF NOT EXISTS idx_movements_date ON stock_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_prices_date ON prices(product_id,effective_date);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON sessions(expires_at);
