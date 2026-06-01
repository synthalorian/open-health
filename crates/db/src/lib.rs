//! open_health_db — Encrypted SQLite database layer.
//!
//! Uses [open_health_crypto] to transparently encrypt/decrypt individual
//! record fields at rest. Schema managed via `schema.rs`.

#![forbid(unsafe_code)]

pub mod schema;

use open_health_crypto::MasterKey;
use open_health_shared::*;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Crypto error")]
    Crypto,
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

/// Encrypted health database with thread-safe interior mutability.
pub struct HealthDatabase {
    conn: Mutex<Connection>,
    key: MasterKey,
    salt: [u8; 32],
}

impl HealthDatabase {
    /// Open (or create) a database at `path`, deriving encryption from `passphrase`.
    pub fn open(path: impl AsRef<Path>, passphrase: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        let salt = match Self::load_salt(&conn) {
            Ok(salt) => salt,
            Err(_) => {
                let salt = MasterKey::generate_salt();
                schema::initialize(&conn)?;
                conn.execute(
                    "INSERT INTO meta (key, value) VALUES ('crypto_salt', ?1)",
                    params![hex::encode(salt)],
                )?;
                salt
            }
        };

        let key = MasterKey::derive(passphrase, &salt);
        Ok(Self { conn: Mutex::new(conn), key, salt })
    }

    fn load_salt(conn: &Connection) -> Result<[u8; 32]> {
        let hex_str: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'crypto_salt'", [], |row| row.get(0))
            .map_err(|_| DbError::NotFound("crypto_salt".into()))?;
        let mut salt = [0u8; 32];
        hex::decode_to_slice(hex_str, &mut salt).map_err(|_| DbError::Crypto)?;
        Ok(salt)
    }

    /// Verify the passphrase matches the stored key.
    pub fn verify_passphrase(&self, passphrase: &str) -> bool {
        MasterKey::verify(passphrase, &self.salt, self.key.as_ref())
    }

    // ─── CRUD: Health Records ──────────────────────────────────────────

    pub fn insert_record(&self, record: &HealthRecord) -> Result<()> {
        let json = serde_json::to_string(record)?;
        let (nonce, ciphertext) = self.key.encrypt(json.as_bytes());
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO health_records (id, record_type, timestamp, value, unit, encrypted_data, nonce, source, import_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.id.to_string(),
                serde_json::to_string(&record.record_type)?,
                record.timestamp.to_string(),
                record.value,
                record.unit,
                hex::encode(&ciphertext),
                hex::encode(nonce),
                record.source,
                record.import_id.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn get_records(
        &self,
        record_type: &RecordType,
        from: chrono::NaiveDateTime,
        to: chrono::NaiveDateTime,
    ) -> Result<Vec<HealthRecord>> {
        let type_str = serde_json::to_string(record_type)?;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT encrypted_data, nonce FROM health_records
             WHERE record_type = ?1 AND timestamp >= ?2 AND timestamp <= ?3
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(
            params![type_str, from.to_string(), to.to_string()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;

        let mut records = Vec::new();
        for row in rows {
            let (hex_cipher, hex_nonce) = row?;
            let ciphertext = hex::decode(&hex_cipher).map_err(|_| DbError::Crypto)?;
            let mut nonce = [0u8; 12];
            hex::decode_to_slice(&hex_nonce, &mut nonce).map_err(|_| DbError::Crypto)?;
            let plaintext = self.key.decrypt(&nonce, &ciphertext).map_err(|_| DbError::Crypto)?;
            records.push(serde_json::from_slice(&plaintext)?);
        }
        Ok(records)
    }

    // ─── Sleep Records ─────────────────────────────────────────────────

    pub fn insert_sleep_record(&self, record: &SleepRecord) -> Result<()> {
        let json = serde_json::to_string(record)?;
        let (nonce, ciphertext) = self.key.encrypt(json.as_bytes());
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO sleep_records (id, date, duration_minutes, quality_score, encrypted_data, nonce, source, import_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id.to_string(),
                record.date.to_string(),
                record.duration_minutes,
                record.quality_score,
                hex::encode(&ciphertext),
                hex::encode(nonce),
                record.source,
                record.import_id.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn get_sleep_records(&self, from: chrono::NaiveDate, to: chrono::NaiveDate) -> Result<Vec<SleepRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT encrypted_data, nonce FROM sleep_records WHERE date >= ?1 AND date <= ?2 ORDER BY date ASC",
        )?;

        let rows = stmt.query_map(params![from.to_string(), to.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (hex_cipher, hex_nonce) = row?;
            let ciphertext = hex::decode(&hex_cipher).map_err(|_| DbError::Crypto)?;
            let mut nonce = [0u8; 12];
            hex::decode_to_slice(&hex_nonce, &mut nonce).map_err(|_| DbError::Crypto)?;
            let plaintext = self.key.decrypt(&nonce, &ciphertext).map_err(|_| DbError::Crypto)?;
            records.push(serde_json::from_slice(&plaintext)?);
        }
        Ok(records)
    }

    // ─── Import Sessions ───────────────────────────────────────────────

    pub fn insert_import_session(&self, session: &ImportSession) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO import_sessions (id, source_name, file_name, record_count, imported_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id.to_string(),
                session.source_name,
                session.file_name,
                session.record_count,
                session.imported_at.to_string(),
                serde_json::to_string(&session.status)?,
            ],
        )?;
        Ok(())
    }

    pub fn list_import_sessions(&self) -> Result<Vec<ImportSession>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_name, file_name, record_count, imported_at, status FROM import_sessions ORDER BY imported_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ImportSession {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                source_name: row.get(1)?,
                file_name: row.get(2)?,
                record_count: row.get(3)?,
                imported_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                status: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or(ImportStatus::Failed("unknown".into())),
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn delete_import_session(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM import_sessions WHERE id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM health_records WHERE import_id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM sleep_records WHERE import_id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn store_passphrase_hash(&self, _passphrase: &str) -> Result<()> {
        let hash = self.key.as_ref().to_vec();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('passphrase_hash', ?1)",
            params![hex::encode(&hash)],
        )?;
        Ok(())
    }

    pub fn change_passphrase(&mut self, new_passphrase: &str) -> Result<()> {
        let new_salt = MasterKey::generate_salt();
        let new_key = MasterKey::derive(new_passphrase, &new_salt);
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE meta SET value = ?1 WHERE key = 'crypto_salt'", params![hex::encode(new_salt)])?;
        self.key = new_key;
        self.salt = new_salt;
        Ok(())
    }

    pub fn verify_stored_passphrase(&self, passphrase: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        let hash_hex: Result<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'passphrase_hash'", [], |row| row.get(0))
            .map_err(|e| e.into());
        drop(conn);
        match hash_hex {
            Ok(hex_str) => {
                let hash = hex::decode(&hex_str).unwrap_or_default();
                MasterKey::verify(passphrase, &self.salt, &hash)
            }
            Err(_) => false,
        }
    }
}

impl std::fmt::Debug for HealthDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthDatabase").finish_non_exhaustive()
    }
}
