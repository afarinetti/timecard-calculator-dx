CREATE TABLE IF NOT EXISTS labor_codes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    wbs_number  TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hour_types (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS timecard_entries (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    labor_code_id  INTEGER NOT NULL REFERENCES labor_codes(id) ON DELETE RESTRICT,
    hour_type_id   INTEGER NOT NULL REFERENCES hour_types(id)  ON DELETE RESTRICT,
    telework       INTEGER NOT NULL DEFAULT 0,
    date           TEXT NOT NULL,
    start_time     TEXT NOT NULL,
    end_time       TEXT,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f','now'))
);

CREATE TABLE IF NOT EXISTS pay_period_anchors (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    start_date TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_timecard_date       ON timecard_entries(date);
CREATE INDEX IF NOT EXISTS idx_timecard_labor_code ON timecard_entries(labor_code_id);
CREATE INDEX IF NOT EXISTS idx_timecard_hour_type  ON timecard_entries(hour_type_id);
