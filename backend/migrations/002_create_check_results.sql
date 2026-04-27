CREATE TABLE IF NOT EXISTS check_results (
    id            TEXT PRIMARY KEY,
    service_id    TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    checked_at    TEXT NOT NULL,
    status        TEXT NOT NULL,
    response_ms   BIGINT,
    detail        TEXT,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_check_results_service_time
    ON check_results(service_id, checked_at DESC);
