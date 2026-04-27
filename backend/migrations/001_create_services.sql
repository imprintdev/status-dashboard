CREATE TABLE IF NOT EXISTS services (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    service_type  TEXT NOT NULL,
    config        TEXT NOT NULL,
    interval_secs BIGINT NOT NULL DEFAULT 60,
    enabled       BIGINT NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
