//! open_health_server — Unix domain socket IPC server.
//!
//! Listens on a Unix socket for JSON-encoded [IpcRequest] messages,
//! processes them against the encrypted database, and returns [IpcResponse].

#![forbid(unsafe_code)]

use open_health_db::HealthDatabase;
use open_health_shared::*;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info};

/// Default socket path (Linux/macOS).
const DEFAULT_SOCKET_PATH: &str = "/tmp/open_health.sock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("open_health_server=info".parse()?),
        )
        .init();

    // Database path and passphrase (in production, passphrase comes from Flutter via socket handshake)
    let db_path = std::env::var("OPEN_HEALTH_DB").unwrap_or_else(|_| "data/open_health.db".into());
    let passphrase = std::env::var("OPEN_HEALTH_PASSPHRASE")
        .unwrap_or_else(|_| "default-dev-passphrase".into());

    let db = std::sync::Arc::new(
        HealthDatabase::open(&db_path, &passphrase)
            .map_err(|e| anyhow::anyhow!("Failed to open DB: {e}"))?,
    );

    // Remove stale socket if present
    let socket_path = PathBuf::from(
        std::env::var("OPEN_HEALTH_SOCKET").unwrap_or_else(|_| DEFAULT_SOCKET_PATH.into()),
    );
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!(
        "🎹 open_health server listening on {}",
        socket_path.display()
    );

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let db_ref = std::sync::Arc::clone(&db);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, db_ref).await {
                        error!("Client error: {e}");
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {e}");
            }
        }
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    db: std::sync::Arc<HealthDatabase>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    while buf_reader.read_line(&mut line).await? > 0 {
        if line.trim().is_empty() {
            line.clear();
            continue;
        }

        let request: IpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let resp = IpcResponse::Error(format!("Invalid JSON: {e}"));
                let resp_json = serde_json::to_string(&resp)?;
                writer.write_all(resp_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                line.clear();
                continue;
            }
        };

        let response = process_request(request, &db);
        let resp_json = serde_json::to_string(&response)?;
        writer.write_all(resp_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

fn process_request(request: IpcRequest, db: &HealthDatabase) -> IpcResponse {
    match request {
        IpcRequest::Ping => IpcResponse::Pong,

        IpcRequest::InsertRecord(record) => match db.insert_record(&record) {
            Ok(_) => IpcResponse::Ok,
            Err(e) => IpcResponse::Error(e.to_string()),
        },

        IpcRequest::GetRecords {
            record_type,
            from,
            to,
        } => match db.get_records(&record_type, from, to) {
            Ok(records) => IpcResponse::Records(records),
            Err(e) => IpcResponse::Error(e.to_string()),
        },

        IpcRequest::GetSleepRecords { from, to } => match db.get_sleep_records(from, to) {
            Ok(records) => IpcResponse::SleepRecords(records),
            Err(e) => IpcResponse::Error(e.to_string()),
        },

        IpcRequest::GetHeartRateSummary { from, to } => {
            // Derive heart rate summary from health records
            match db.get_records(&RecordType::HeartRate, from.and_hms_opt(0, 0, 0).unwrap(), to.and_hms_opt(23, 59, 59).unwrap()) {
                Ok(records) => {
                    let mut summaries = Vec::new();
                    if !records.is_empty() {
                        let avg = records.iter().map(|r| r.value).sum::<f64>() / records.len() as f64;
                        let min = records.iter().map(|r| r.value).fold(f64::MAX, f64::min);
                        let max = records.iter().map(|r| r.value).fold(f64::MIN, f64::max);
                        summaries.push(HeartRateSummary {
                            date: from,
                            avg_bpm: Some(avg),
                            min_bpm: Some(min),
                            max_bpm: Some(max),
                            resting_bpm: None,
                            hrv_rmssd: None,
                            source: None,
                        });
                    }
                    IpcResponse::HeartRateSummaries(summaries)
                }
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::GetBodyComposition { from: _, to: _ } => {
            IpcResponse::BodyCompositions(vec![]) // TODO: implement
        }

        IpcRequest::GetActivitySummary { from: _, to: _ } => {
            IpcResponse::ActivitySummaries(vec![]) // TODO: implement
        }

        IpcRequest::ListImportSessions => match db.list_import_sessions() {
            Ok(sessions) => IpcResponse::ImportSessions(sessions),
            Err(e) => IpcResponse::Error(e.to_string()),
        },

        IpcRequest::DeleteImportSession(id) => match db.delete_import_session(id) {
            Ok(_) => IpcResponse::Ok,
            Err(e) => IpcResponse::Error(e.to_string()),
        },

        IpcRequest::Shutdown => {
            std::process::exit(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_ping() {
        let db = HealthDatabase::open(":memory:", "test").unwrap();
        let resp = process_request(IpcRequest::Ping, &db);
        assert!(matches!(resp, IpcResponse::Pong));
    }

    #[test]
    fn test_process_insert_and_get() {
        let db = HealthDatabase::open(":memory:", "test").unwrap();
        let now = chrono::Utc::now().naive_utc();
        let record = HealthRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::HeartRate,
            timestamp: now,
            value: 72.0,
            unit: "bpm".into(),
            source: Some("test".into()),
            notes: None,
            import_id: None,
        };

        let resp = process_request(IpcRequest::InsertRecord(record.clone()), &db);
        assert!(matches!(resp, IpcResponse::Ok));

        let from = now - chrono::Duration::hours(1);
        let to = now + chrono::Duration::hours(1);
        let resp = process_request(
            IpcRequest::GetRecords {
                record_type: RecordType::HeartRate,
                from,
                to,
            },
            &db,
        );
        match resp {
            IpcResponse::Records(records) => {
                assert_eq!(records.len(), 1);
                assert!((records[0].value - 72.0).abs() < 0.01);
            }
            _ => panic!("Expected Records response"),
        }
    }
}
