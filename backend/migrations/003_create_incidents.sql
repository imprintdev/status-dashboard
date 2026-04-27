CREATE TABLE IF NOT EXISTS incidents (
    id             TEXT PRIMARY KEY,
    service_id     TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    started_at     TIMESTAMPTZ NOT NULL,
    resolved_at    TIMESTAMPTZ,
    status         TEXT NOT NULL,
    trigger_status TEXT NOT NULL,
    notes          TEXT
);

CREATE INDEX IF NOT EXISTS idx_incidents_service
    ON incidents(service_id, started_at DESC);
