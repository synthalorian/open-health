//! open_health_import — Generic CSV import pipeline.
//!
//! Maps user-defined columns to [HealthRecord] fields and ingests into the DB.

#![forbid(unsafe_code)]

use chrono::NaiveDateTime;
use open_health_db::HealthDatabase;
use open_health_shared::{HealthRecord, ImportSession, ImportStatus, RecordType};
use std::collections::HashMap;
use std::io::Read;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("DB error: {0}")]
    Db(String),
    #[error("Missing required column: {0}")]
    MissingColumn(String),
}

pub type Result<T> = std::result::Result<T, ImportError>;

/// Column mapping configuration for generic CSV import.
#[derive(Debug, Clone, Default)]
pub struct ColumnMapping {
    pub timestamp: String,
    pub value: String,
    pub record_type: String,
    pub unit: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

impl ColumnMapping {
    /// Auto-detect mapping from CSV headers using common names.
    pub fn auto_detect(headers: &[String]) -> Option<Self> {
        let header_map: HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.to_lowercase().replace(' ', "_"), i))
            .collect();

        let timestamp = detect_column(
            &header_map,
            &["timestamp", "date", "datetime", "time", "date_time", "created_at"],
        )?;
        let value = detect_column(
            &header_map,
            &["value", "amount", "measurement", "val", "reading"],
        )?;
        let record_type = detect_column(
            &header_map,
            &[
                "type",
                "record_type",
                "metric",
                "measurement_type",
                "category",
            ],
        )?;

        Some(Self {
            timestamp: headers[timestamp].clone(),
            value: headers[value].clone(),
            record_type: headers[record_type].clone(),
            unit: detect_column(&header_map, &["unit", "units"]).map(|i| headers[i].clone()),
            source: detect_column(&header_map, &["source", "device", "tracker"])
                .map(|i| headers[i].clone()),
            notes: detect_column(&header_map, &["notes", "note", "comment", "comments"])
                .map(|i| headers[i].clone()),
        })
    }
}

fn detect_column(map: &HashMap<String, usize>, candidates: &[&str]) -> Option<usize> {
    for c in candidates {
        if let Some(&idx) = map.get(*c) {
            return Some(idx);
        }
    }
    None
}

/// Import a generic CSV into the database.
pub fn import_csv<R: Read>(
    db: &HealthDatabase,
    reader: R,
    mapping: &ColumnMapping,
    source_name: &str,
    file_name: &str,
) -> Result<ImportSession> {
    let mut csv_reader = csv::Reader::from_reader(reader);
    let headers: Vec<String> = csv_reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let header_idx: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), i))
        .collect();

    let ts_idx = *header_idx
        .get(&mapping.timestamp)
        .ok_or_else(|| ImportError::MissingColumn(mapping.timestamp.clone()))?;
    let val_idx = *header_idx
        .get(&mapping.value)
        .ok_or_else(|| ImportError::MissingColumn(mapping.value.clone()))?;
    let type_idx = *header_idx
        .get(&mapping.record_type)
        .ok_or_else(|| ImportError::MissingColumn(mapping.record_type.clone()))?;

    let unit_idx = mapping.unit.as_ref().and_then(|u| header_idx.get(u).copied());
    let source_idx = mapping.source.as_ref().and_then(|s| header_idx.get(s).copied());
    let notes_idx = mapping.notes.as_ref().and_then(|n| header_idx.get(n).copied());

    let import_id = Uuid::new_v4();
    let mut record_count = 0u32;

    for result in csv_reader.records() {
        let record = result?;
        let timestamp_str = record
            .get(ts_idx)
            .ok_or_else(|| ImportError::Parse("missing timestamp".into()))?;
        let timestamp = parse_timestamp(timestamp_str)
            .map_err(|e| ImportError::Parse(format!("timestamp '{timestamp_str}': {e}")))?;

        let value_str = record
            .get(val_idx)
            .ok_or_else(|| ImportError::Parse("missing value".into()))?;
        let value: f64 = value_str
            .parse()
            .map_err(|e| ImportError::Parse(format!("value '{value_str}': {e}")))?;

        let type_str = record
            .get(type_idx)
            .ok_or_else(|| ImportError::Parse("missing type".into()))?;
        let record_type = parse_record_type(type_str);

        let unit = unit_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .to_string();
        let source = source_idx.and_then(|i| record.get(i)).map(|s| s.to_string());
        let notes = notes_idx.and_then(|i| record.get(i)).map(|s| s.to_string());

        let health_record = HealthRecord {
            id: Uuid::new_v4(),
            record_type,
            timestamp,
            value,
            unit,
            source: source.clone(),
            notes,
            import_id: Some(import_id),
        };

        db.insert_record(&health_record)
            .map_err(|e| ImportError::Db(e.to_string()))?;
        record_count += 1;
    }

    let session = ImportSession {
        id: import_id,
        source_name: source_name.to_string(),
        file_name: file_name.to_string(),
        record_count,
        imported_at: chrono::Utc::now().naive_utc(),
        status: ImportStatus::Success,
    };

    db.insert_import_session(&session)
        .map_err(|e| ImportError::Db(e.to_string()))?;

    Ok(session)
}

fn parse_timestamp(s: &str) -> Result<NaiveDateTime> {
    // Try ISO 8601 first
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt);
    }
    // Try common date formats
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d.and_hms_opt(0, 0, 0).unwrap());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Ok(d.and_hms_opt(0, 0, 0).unwrap());
    }
    Err(ImportError::Parse(format!("unrecognized timestamp: {s}")))
}

fn parse_record_type(s: &str) -> RecordType {
    let lower = s.to_lowercase().replace(' ', "_");
    match lower.as_str() {
        "heart_rate" | "heartrate" | "hr" => RecordType::HeartRate,
        "resting_heart_rate" | "resting_hr" | "resting" => RecordType::RestingHeartRate,
        "hrv" | "heart_rate_variability" => RecordType::Hrv,
        "sleep_duration" | "sleep" | "sleep_time" => RecordType::SleepDuration,
        "sleep_quality" | "quality" => RecordType::SleepQuality,
        "deep_sleep" | "deep" => RecordType::DeepSleep,
        "rem_sleep" | "rem" => RecordType::RemSleep,
        "light_sleep" | "light" => RecordType::LightSleep,
        "awake_time" | "awake" => RecordType::AwakeTime,
        "steps" | "step_count" => RecordType::Steps,
        "calories" | "cal" | "cals" => RecordType::Calories,
        "active_calories" | "active_cals" => RecordType::ActiveCalories,
        "distance" | "dist" => RecordType::Distance,
        "weight" | "body_mass" | "mass" => RecordType::BodyMass,
        "body_fat" | "fat" | "body_fat_percentage" => RecordType::BodyFat,
        "bmi" => RecordType::Bmi,
        "glucose" | "blood_glucose" | "bg" => RecordType::Glucose,
        "spo2" | "blood_oxygen" | "oxygen" => RecordType::Spo2,
        "stress" => RecordType::Stress,
        "temperature" | "temp" => RecordType::Temperature,
        "respiratory_rate" | "resp_rate" | "breathing_rate" => RecordType::RespiratoryRate,
        "blood_pressure_systolic" | "bp_sys" | "systolic" => RecordType::BloodPressureSystolic,
        "blood_pressure_diastolic" | "bp_dia" | "diastolic" => {
            RecordType::BloodPressureDiastolic
        }
        _ => RecordType::Custom(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_record_type() {
        assert!(matches!(parse_record_type("heart_rate"), RecordType::HeartRate));
        assert!(matches!(parse_record_type("Sleep Duration"), RecordType::SleepDuration));
        assert!(matches!(
            parse_record_type("custom_metric"),
            RecordType::Custom(_)
        ));
    }

    #[test]
    fn test_parse_timestamp_iso() {
        let ts = parse_timestamp("2026-05-14T08:30:00").unwrap();
        assert_eq!(ts.date().year(), 2026);
        assert_eq!(ts.date().month(), 5);
    }

    #[test]
    fn test_auto_detect_headers() {
        let headers = vec![
            "timestamp".to_string(),
            "value".to_string(),
            "type".to_string(),
            "unit".to_string(),
        ];
        let mapping = ColumnMapping::auto_detect(&headers).unwrap();
        assert_eq!(mapping.timestamp, "timestamp");
        assert_eq!(mapping.value, "value");
        assert_eq!(mapping.record_type, "type");
        assert_eq!(mapping.unit, Some("unit".to_string()));
    }
}
