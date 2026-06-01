//! open_health_shared — Data types for the health data engine.
//!
//! All structs shared between the backend, IPC layer, and frontend.

#![forbid(unsafe_code)]
#![allow(clippy::upper_case_acronyms, clippy::similar_names, clippy::struct_field_names)]

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Core Record Types ─────────────────────────────────────────────────

/// A single health data point — the universal record format.
/// Every import source normalises into this.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthRecord {
    pub id: Uuid,
    pub record_type: RecordType,
    /// ISO-8601 timestamp when this measurement was taken
    pub timestamp: NaiveDateTime,
    /// The numeric value
    pub value: f64,
    /// Unit string (e.g. "bpm", "kg", "mg/dL", "min")
    pub unit: String,
    /// Optional source device or tracker name
    pub source: Option<String>,
    /// Optional notes / tags
    pub notes: Option<String>,
    /// FK to import_sessions
    pub import_id: Option<Uuid>,
}

/// Categorised health metric types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecordType {
    HeartRate,
    Hrv,                     // Heart Rate Variability (rMSSD ms)
    RestingHeartRate,
    SleepDuration,
    SleepQuality,            // 0–100 score
    SleepOnset,              // Time fell asleep (epoch seconds)
    SleepOffset,             // Time woke up (epoch seconds)
    DeepSleep,               // Minutes
    RemSleep,                // Minutes
    LightSleep,              // Minutes
    AwakeTime,               // Minutes awake during night
    BodyMass,
    BodyFat,
    BodyWater,
    MuscleMass,
    BoneMass,
    Bmi,
    Glucose,
    Steps,
    Calories,
    ActiveCalories,
    Distance,
    Floors,
    ActivityMinutes,
    ModerateActivity,
    VigorousActivity,
    Stress,
    Spo2,                    // Blood oxygen saturation %
    RespiratoryRate,
    Temperature,
    SkinTemperature,
    BloodPressureSystolic,
    BloodPressureDiastolic,
    Custom(String),
}

// ─── Sleep Record (aggregated) ─────────────────────────────────────────

/// A single night's sleep, aggregated from raw data points.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SleepRecord {
    pub id: Uuid,
    pub date: NaiveDate,
    pub duration_minutes: u32,
    pub quality_score: Option<u8>,     // 0–100
    pub deep_sleep_min: Option<u32>,
    pub rem_sleep_min: Option<u32>,
    pub light_sleep_min: Option<u32>,
    pub awake_min: Option<u32>,
    pub onset: Option<NaiveDateTime>,
    pub offset: Option<NaiveDateTime>,
    pub source: Option<String>,
    pub import_id: Option<Uuid>,
}

// ─── Heart Rate Snapshot ───────────────────────────────────────────────

/// A snapshot of heart-rate stats for a given period.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeartRateSummary {
    pub date: NaiveDate,
    pub avg_bpm: Option<f64>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub resting_bpm: Option<f64>,
    pub hrv_rmssd: Option<f64>,
    pub source: Option<String>,
}

// ─── Body Composition ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodyComposition {
    pub date: NaiveDate,
    pub weight_kg: Option<f64>,
    pub body_fat_pct: Option<f64>,
    pub muscle_kg: Option<f64>,
    pub bone_kg: Option<f64>,
    pub water_pct: Option<f64>,
    pub bmi: Option<f64>,
    pub source: Option<String>,
}

// ─── Glucose ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlucoseRecord {
    pub id: Uuid,
    pub timestamp: NaiveDateTime,
    pub value_mg_dl: f64,
    pub source: Option<String>,
    pub notes: Option<String>,
}

// ─── Activity ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivitySummary {
    pub date: NaiveDate,
    pub steps: Option<u32>,
    pub calories: Option<f64>,
    pub active_minutes: Option<u32>,
    pub distance_km: Option<f64>,
    pub floors: Option<u32>,
    pub source: Option<String>,
}

// ─── Import Tracking ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportSession {
    pub id: Uuid,
    pub source_name: String,         // "fitbit", "oura", "apple_health", etc.
    pub file_name: String,
    pub record_count: u32,
    pub imported_at: NaiveDateTime,
    pub status: ImportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportStatus {
    Success,
    Partial,
    Failed(String),
}

// ─── Device ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: Uuid,
    pub name: String,
    pub device_type: String,         // "fitbit_charge", "oura_ring", "garmin", etc.
    pub last_synced: Option<NaiveDateTime>,
}

// ─── IPC Message Types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    Ping,
    InsertRecord(HealthRecord),
    GetRecords {
        record_type: RecordType,
        from: NaiveDateTime,
        to: NaiveDateTime,
    },
    GetSleepRecords {
        from: NaiveDate,
        to: NaiveDate,
    },
    GetHeartRateSummary {
        from: NaiveDate,
        to: NaiveDate,
    },
    GetBodyComposition {
        from: NaiveDate,
        to: NaiveDate,
    },
    GetActivitySummary {
        from: NaiveDate,
        to: NaiveDate,
    },
    ListImportSessions,
    DeleteImportSession(Uuid),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Pong,
    Ok,
    Records(Vec<HealthRecord>),
    SleepRecords(Vec<SleepRecord>),
    HeartRateSummaries(Vec<HeartRateSummary>),
    BodyCompositions(Vec<BodyComposition>),
    ActivitySummaries(Vec<ActivitySummary>),
    ImportSessions(Vec<ImportSession>),
    Error(String),
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_record_serde_roundtrip() {
        let record = HealthRecord {
            id: Uuid::new_v4(),
            record_type: RecordType::HeartRate,
            timestamp: chrono::Utc::now().naive_utc(),
            value: 72.0,
            unit: "bpm".into(),
            source: Some("test".into()),
            notes: None,
            import_id: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        let decoded: HealthRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, decoded);
    }

    #[test]
    fn test_sleep_record_serde() {
        let sleep = SleepRecord {
            id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            duration_minutes: 420,
            quality_score: Some(85),
            deep_sleep_min: Some(90),
            rem_sleep_min: Some(100),
            light_sleep_min: Some(200),
            awake_min: Some(30),
            onset: None,
            offset: None,
            source: Some("oura".into()),
            import_id: None,
        };
        let json = serde_json::to_string(&sleep).unwrap();
        let decoded: SleepRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(sleep, decoded);
    }

    #[test]
    fn test_ipc_message_roundtrip() {
        let req = IpcRequest::GetRecords {
            record_type: RecordType::HeartRate,
            from: chrono::Utc::now().naive_utc() - chrono::Duration::days(7),
            to: chrono::Utc::now().naive_utc(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: IpcRequest = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, IpcRequest::GetRecords { .. }));
    }
}
