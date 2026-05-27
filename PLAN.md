# open_health — Development Plan

> **Privacy-first health data aggregator. Your data stays local. Always.**

---

## Vision

A local-first, encrypted health data engine that turns CSV exports from any tracker into actionable insights — without a single byte leaving the device.

---

## Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Rust Backend** | `tokio`, `rusqlite`, `ring` | Async, safe, AES-GCM encryption |
| **Flutter Frontend** | Riverpod, `fl_chart`, `go_router` | Clean state, smooth charts |
| **Database** | SQLite + AES-GCM via `ring` | Embedded, encrypted, fast |
| **IPC** | Unix domain socket (Linux/macOS), named pipe (Windows) | Zero-copy, no network exposure |
| **Import Pipeline** | `csv` crate, `serde_json` | Flexible parsing |
| **Crypto** | `ring` (AES-GCM-256) | Battle-tested, audited |

---

## Development Phases

### Phase 1 — Foundation (Core Engine + Data Model)

**Goal:** Rust backend that can encrypt/decrypt a SQLite database and store/retrieve health records.

#### 1.1 Database Schema

- [ ] Design schema: `health_records`, `import_sessions`, `devices`
- [ ] Write schema migration in `rust/src/db/schema.rs`
- [ ] Implement typed queries in `rust/src/db/queries.rs`
- [ ] Write schema integration tests

#### 1.2 Encryption Layer

- [ ] Implement key derivation (PBKDF2 + user passphrase) in `rust/src/crypto.rs`
- [ ] Implement AES-GCM encryption for each SQLite table
- [ ] Implement secure key storage (OS keychain via `keyring`)
- [ ] Write crypto unit tests (encrypt ↔ decrypt roundtrip)

#### 1.3 Data Model

- [ ] Define Rust structs: `SleepRecord`, `HeartRate`, `Glucose`, `BodyComposition`, `Activity`
- [ ] Implement serialization/deserialization (`serde`)
- [ ] Write validation (ranges, types, required fields)
- [ ] Write serde unit tests

#### 1.4 Local IPC Server

- [ ] Implement Unix domain socket server in `rust/src/server/`
- [ ] Define message types (`Request` / `Response`)
- [ ] Implement basic CRUD endpoints
- [ ] Write server integration tests

**Deliverable:** `cargo test` passes 100%. `rust/src/lib.rs` exposes a stable public API.

---

### Phase 2 — Import Pipeline

**Goal:** Parse CSV/JSON exports from Fitbit, Oura, Apple Health, Garmin, Whoop → normalized records.

#### 2.1 Generic CSV Parser

- [ ] Build flexible CSV parser in `rust/src/import/generic_csv.rs`
- [ ] Support column mapping via config
- [ ] Handle timezone normalization (ISO 8601)
- [ ] Write CSV parsing tests with sample files

#### 2.2 Fitbit Import

- [ ] Parse Fitbit daily summary CSV
- [ ] Parse Fitbit hourly data CSV
- [ ] Map Fitbit fields → `health_records` schema
- [ ] Write import tests with real Fitbit CSV structure

#### 2.3 Oura Import

- [ ] Parse Oura JSON export (sleep, readiness, activity, HRV)
- [ ] Flatten nested JSON → flat records
- [ ] Handle date range queries (Oura paginated API structure)
- [ ] Write import tests with Oura sample JSON

#### 2.4 Apple Health Import

- [ ] Parse Apple Health XML export
- [ ] Parse Apple Health CSV export
- [ ] Handle Apple Health's `type`/`startDate`/`endDate`/`value` format
- [ ] Map Apple Health types → normalized schema

#### 2.5 Garmin Import

- [ ] Parse Garmin CSV (daily summary, activities, sleep)
- [ ] Map Garmin fields → normalized schema
- [ ] Handle Garmin's proprietary date/time format

#### 2.6 Whoop Import

- [ ] Parse Whoop CSV export (strain, recovery, sleep, HR)
- [ ] Handle Whoop's unique metric names
- [ ] Map → normalized schema

#### 2.7 Generic CSV Importer

- [ ] Build a universal CSV mapper (user defines columns)
- [ ] Config file: `importers/<name>.yaml` with column mappings
- [ ] Auto-detect format from header names

**Deliverable:** `cargo run --bin import -- data/imports/` ingests all five sources.

---

### Phase 3 — Analytics & Dashboard Backend

**Goal:** Rust computes health trends, correlations, anomaly detection.

#### 3.1 Aggregation Engine

- [ ] Daily/weekly/monthly rollup queries
- [ ] Moving averages (7-day, 30-day)
- [ ] Trend direction (↑ ↓ →)
- [ ] Write aggregation SQL + Rust tests

#### 3.2 Sleep Analysis

- [ ] Sleep duration trends
- [ ] Sleep onset/offset patterns
- [ ] Sleep quality score (composite from Oura/fitbit)
- [ ] Correlate sleep with activity/HRV

#### 3.3 Heart Rate / HRV

- [ ] Resting heart rate trends
- [ ] HRV (rMSSD, LF/HF) time series
- [ ] Anomaly detection (sudden HR spikes)
- [ ] Zone-based analysis (if heart rate zones available)

#### 3.4 Body Composition

- [ ] Weight trend over time
- [ ] Body fat % trends
- [ ] Correlate body comp with activity/sleep

#### 3.5 Glucose (if available)

- [ ] Glucose time series
- [ ] Correlate with activity/food (if data available)
- [ ] Time-in-range analysis

#### 3.6 PDF Reports

- [ ] Generate weekly/monthly summary PDFs via `pdf` crate
- [ ] Include charts (sleep, HR, activity, weight)
- [ ] Export to file
- [ ] Write PDF generation tests

**Deliverable:** Rust API exposes computed analytics. `cargo run --bin report -- week` generates a PDF.

---

### Phase 4 — Flutter Frontend

**Goal:** Synthwave-themed UI that displays health data, manages imports, generates reports.

#### 4.1 App Shell & Routing

- [ ] Flutter project scaffold
- [ ] Riverpod state management
- [ ] `go_router` setup (dashboard, imports, reports tabs)
- [ ] Synthwave neon theme (dark mode, gradient accents)

#### 4.2 IPC Bridge

- [ ] Unix socket client in Flutter (`dart:io` `Socket`)
- [ ] Serialize/deserialize messages to/from Rust
- [ ] Error handling (connection lost, timeout)
- [ ] Background service (keep Rust process alive)

#### 4.3 Dashboard Screen

- [ ] Health overview cards (sleep, HR, activity, weight)
- [ ] `fl_chart` line charts for trends
- [ ] Trend arrows + percentage change
- [ ] Date range selector (7d / 30d / 90d / 1y)
- [ ] Pull-to-refresh

#### 4.4 Import Screen

- [ ] File picker for CSV/JSON/XML
- [ ] Format detector (auto-detect from file)
- [ ] Column mapping UI (for generic CSV)
- [ ] Import progress indicator
- [ ] Import history list

#### 4.5 Reports Screen

- [ ] List generated reports (weekly/monthly)
- [ ] PDF preview
- [ ] Generate new report button
- [ ] Export/share PDF

#### 4.6 Settings Screen

- [ ] Passphrase setup / change
- [ ] Import format preferences
- [ ] Data export (encrypted DB backup)
- [ ] About / feedback

#### 4.7 Polish

- [ ] Lottie animations for loading states
- [ ] Haptic feedback on interactions
- [ ] Offline-first (all state local)
- [ ] Accessibility (semantic labels, contrast)

**Deliverable:** `flutter build apk --release` produces a working Android APK.

---

### Phase 5 — Hardening & Release

#### 5.1 Security Audit

- [ ] Pen-test IPC layer
- [ ] Verify encryption at rest (dump SQLite, confirm unreadable)
- [ ] Key rotation flow
- [ ] Secure deletion (overwrite keys on uninstall)

#### 5.2 CI/CD

- [ ] GitHub Actions: Rust fmt, clippy, test (PR)
- [ ] GitHub Actions: Flutter analyze, test (PR)
- [ ] GitHub Actions: Cross-platform release (Android APK, iOS IPA, Linux AppImage, macOS DMG)
- [ ] Release artifacts attached to GitHub releases

#### 5.3 Performance

- [ ] Benchmark import pipeline (10k+ records)
- [ ] Optimize SQLite queries with EXPLAIN ANALYZE
- [ ] Lazy load chart data (don't fetch all records)
- [ ] Memory usage profiling

#### 5.4 Documentation

- [ ] Update README with screenshots
- [ ] Add import guide (how to export from each tracker)
- [ ] Developer guide (how to add new importers)
- [ ] Security model doc

#### 5.5 Publishing

- [ ] Publish Android APK to GitHub Releases
- [ ] Submit to F-Diff (if applicable)
- [ ] Submit to Amazon Appstore
- [ ] Write blog post / HN / r/selfhosted

**Deliverable:** v1.0 release on GitHub with binaries for Android, iOS, Linux, macOS.

---

## Dependencies Between Projects

| Feature | Depends on |
|---------|-----------|
| Wearable notifications (heart rate alerts) | open_grid (BLE discovery + messaging) |
| Group challenges (friends' health) | open_grid (P2P messaging) |
| Health → Habit correlation | open_habit (cross-app data sharing) |

These are future integrations. Phase 1–4 are fully self-contained.

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Import time (10k records) | < 30 seconds |
| DB size (1 year data) | < 10 MB |
| Encryption strength | AES-GCM-256, PBKDF2 100k iterations |
| Crash-free sessions | > 99% |
| Import success rate (known formats) | > 95% |

---

## Open Questions

1. **Apple Health export format** — XML or CSV? Both need support.
2. **Oura export** — Does Oura still provide direct JSON exports, or is it via API only? Need to verify.
3. **Key recovery** — If user forgets passphrase, should we offer a recovery key option?
4. **Cross-platform IPC** — Windows uses named pipes. How to unify with Unix sockets?
5. **Background sync** — If user imports data on desktop, should it sync to mobile? (Out of scope — stays local-only.)

---

*Privacy is a right. Health data shouldn't be an exception.*
