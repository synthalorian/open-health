//! Database schema initialization and migrations.

use rusqlite::Connection;

/// Create all tables if they don't exist.
pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        -- Key-value store for metadata (salt, passphrase hash, schema version)
        CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Encrypted health data points
        CREATE TABLE IF NOT EXISTS health_records (
            id             TEXT PRIMARY KEY,
            record_type    TEXT NOT NULL,
            timestamp      TEXT NOT NULL,
            value          REAL NOT NULL,
            unit           TEXT NOT NULL,
            encrypted_data TEXT NOT NULL,
            nonce          TEXT NOT NULL,
            source         TEXT,
            import_id      TEXT,
            created_at     TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_health_records_type_ts
            ON health_records(record_type, timestamp);

        -- Encrypted sleep records
        CREATE TABLE IF NOT EXISTS sleep_records (
            id              TEXT PRIMARY KEY,
            date            TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL,
            quality_score   INTEGER,
            encrypted_data  TEXT NOT NULL,
            nonce           TEXT NOT NULL,
            source          TEXT,
            import_id       TEXT,
            created_at      TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sleep_records_date
            ON sleep_records(date);

        -- Import session tracking
        CREATE TABLE IF NOT EXISTS import_sessions (
            id          TEXT PRIMARY KEY,
            source_name TEXT NOT NULL,
            file_name   TEXT NOT NULL,
            record_count INTEGER NOT NULL,
            imported_at TEXT NOT NULL,
            status      TEXT NOT NULL,
            created_at  TEXT DEFAULT (datetime('now'))
        );

        -- Registered devices
        CREATE TABLE IF NOT EXISTS devices (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            device_type TEXT NOT NULL,
            last_synced TEXT
        );

        -- Schema version tracking
        INSERT OR IGNORE INTO meta (key, value) VALUES ('schema_version', '1');
        ",
    )?;
    Ok(())
}
