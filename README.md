# open_health

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Flutter-3.0-blue?style=for-the-badge&logo=flutter" alt="Flutter">
  <img src="https://img.shields.io/badge/License-Apache_2.0-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Status-MVP-red?style=for-the-badge" alt="Status">
</p>

> **Privacy-first health data aggregator. Your data stays local. Always.**

---

## Table of Contents

- [The Problem](#the-problem)
- [The Solution](#the-solution)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Architecture](#architecture)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## The Problem

Every health app wants your data. Apple Health, Google Fit, Fitbit, Oura, Whoop, Garmin — they all track you, store your data in the cloud, and monetize insights. As a health enthusiast who values privacy, you have no local-first alternative that puts you in full control.

## The Solution

**open_health** is a local-only health data engine. Import CSV exports from any major tracker, view beautiful charts and insights, and keep everything encrypted on-device. No cloud. No accounts. No telemetry. Just your health data, owned by you.

```
┌───────────────────────────────────────────────────┐
│              open_health architecture              │
│                                                    │
│  ┌──────────────┐      ┌──────────────────┐       │
│  │  Flutter UI   │◄────►│  Rust Backend    │       │
│  │  (Charts,    │  IPC │  SQLite (AES-GCM)│       │
│  │   Dashboard) │      │  Data pipeline   │       │
│  └──────────────┘      └──────────────────┘       │
│                                                    │
│  ┌──────────────────────────────────────────┐      │
│  │  Import Pipeline                          │      │
│  │  Fitbit • Oura • Apple Health • Garmin    │      │
│  │  • Whoop • Custom CSV • Manual entry      │      │
│  └──────────────────────────────────────────┘      │
└───────────────────────────────────────────────────┘
```

---

## Features

### MVP (Current)
- ✅ CSV import from Fitbit, Oura, Apple Health, Garmin, Whoop, and generic sources
- ✅ Encrypted local SQLite database (AES-GCM-256 via `ring`)
- ✅ Auto-detection of CSV columns for generic imports
- ✅ Dashboard with sleep trends, HRV, heart rate, steps, calories, and body composition
- ✅ Weekly / monthly report generation (PDF)
- ✅ Synthwave dark theme + clean light theme
- ✅ Unix domain socket IPC between Flutter frontend and Rust backend

### Roadmap
- 🔜 Bluetooth LE scanning for real-time heart rate
- 🔜 Apple HealthKit / Google Fit native integration
- 🔜 Anomaly detection (Rust ML via `burn`)
- 🔜 WearOS / WatchOS companion app
- 🔜 Health data export (PDF, CSV, FHIR)

---

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Rust Backend** | Tokio, `rusqlite`, `ring`, `serde` | Async, safe, encrypted, zero-copy IPC |
| **Flutter Frontend** | Riverpod, `fl_chart`, `go_router`, Material 3 | Clean state, smooth UI, adaptive themes |
| **Database** | SQLite + AES-GCM-256 | Embedded, encrypted at rest, fast queries |
| **Communication** | Unix domain socket (NDJSON) | No network exposure, minimal overhead |
| **Crypto** | `ring` (PBKDF2 + AES-GCM-256) | Battle-tested, audited, constant-time |

---

## Getting Started

### Prerequisites
- **Rust** 1.75+ (`rustup` recommended)
- **Flutter** 3.24.0+ with Dart 3.11.5+
- **macOS** 13+ / **Android** API 26+ / **Linux** (Windows via WSL)

### Build & Run

```bash
# Clone the repo
git clone https://github.com/synth/open_health.git
cd open_health

# Build the Rust workspace
cargo build --release

# Run the IPC server
cargo run --bin open_health_server

# In another terminal, build and run the Flutter app
cd flutter
flutter pub get
flutter run
```

### Import Data

```bash
# Place your CSV exports in the data directory
mkdir -p data/imports
cp ~/Downloads/fitbit-export.csv data/imports/
cp ~/Downloads/oura-export.json data/imports/

# Import via the app UI, or use the generic CSV import API
```

---

## Architecture

### Rust Workspace (`crates/`)

The backend is organised as a Cargo workspace with five crates:

```
crates/
├── crypto/          # AES-GCM-256 encryption + PBKDF2 key derivation
├── shared/          # Core data types (HealthRecord, SleepRecord, IPC messages)
├── db/              # Encrypted SQLite layer + schema management
├── import/          # Generic CSV import pipeline with column auto-detection
└── server/          # Tokio-based Unix socket IPC server
```

| Crate | Responsibility |
|-------|----------------|
| `open_health_crypto` | `MasterKey` derivation, `encrypt()` / `decrypt()`, salt & nonce generation |
| `open_health_shared` | Serde-compatible structs: `HealthRecord`, `SleepRecord`, `IpcRequest`, `IpcResponse` |
| `open_health_db` | `HealthDatabase` with CRUD, transparent field-level encryption, schema init |
| `open_health_import` | `ColumnMapping::auto_detect()`, `import_csv()`, timestamp & record-type parsing |
| `open_health_server` | Async NDJSON request/response over Unix socket, request routing |

### Flutter Frontend (`flutter/`)

```
flutter/
├── lib/
│   ├── main.dart
│   ├── app.dart                 # App shell, routing, bottom nav
│   ├── core/
│   │   ├── theme/app_theme.dart  # Synthwave / light themes
│   │   └── services/ipc_client.dart  # Unix socket NDJSON client
│   ├── providers/
│   │   └── health_provider.dart  # Riverpod async data providers
│   └── features/
│       ├── dashboard/           # Metric cards, line & bar charts
│       ├── imports/             # File import UI
│       ├── reports/             # Report generation dialog
│       └── settings/            # Theme toggle, DB status, about
└── test/
    └── widget_test.dart         # Navigation & rendering tests
```

---

## Development

### Running Tests

```bash
# Rust — all crates, all targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Flutter
cd flutter
flutter test
flutter analyze
```

### Code Style

- **Rust:** `cargo fmt` + `cargo clippy -- -D warnings`
- **Flutter:** `dart format .` + `flutter analyze`

### Security Model

- Master key is derived from a user passphrase via **PBKDF2-HMAC-SHA256** (100k iterations).
- Each record is encrypted with a **unique 96-bit nonce** using AES-GCM-256.
- Key material is **zeroed on drop** (`MasterKey` implements `Drop`).
- The IPC server listens on a **Unix domain socket** — no TCP, no network exposure.

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repo
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- `cargo test --workspace` passes
- `flutter analyze` passes (if Flutter code is modified)

---

## License

This project is licensed under the **Apache License, Version 2.0**. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

Built with love by **synth** 🎹🦞 — [synthclaw](https://github.com/synth)

Part of **The Neon Stack** — three open-source apps, one ecosystem.

---

*Privacy is a right. Health data shouldn't be an exception.*
