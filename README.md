# open_health

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Flutter-3.0-blue?style=for-the-badge&logo=flutter" alt="Flutter">
  <img src="https://img.shields.io/badge/License-Apache_2.0-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Status-MVP-red?style=for-the-badge" alt="Status">
</p>

> **Privacy-first health data aggregator. Your data stays local. Always.**

---

## The Problem

Every health app wants your data. Apple Health, Google Fit, Fitbit, Oura, Whoop, Garmin — they all track you, store your data, and sell insights. You're a health enthusiast who cares about privacy.

## The Solution

**open_health** is a local-only health data engine. Import CSV exports from any tracker. Get beautiful charts and insights. No cloud. No accounts. No telemetry. Just raw health data, encrypted on-device.

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

## Features

### MVP (Current)
- ✅ CSV import from Fitbit, Oura, Apple Health, Garmin, Whoop
- ✅ Encrypted local SQLite database (`ring` + AES-GCM)
- ✅ Dashboard with sleep trends, HRV, glucose, body composition
- ✅ Weekly/monthly PDF reports
- ✅ Synthwave dark theme

### Roadmap
- 🔜 Bluetooth LE scanning for real-time heart rate
- 🔜 Apple Health / Google Fit native integration
- 🔜 Anomaly detection (Rust ML via `burn`)
- 🔜 WearOS / WatchOS companion app
- 🔜 HealthKit / HealthData exports

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Rust Backend** | Tokio, `rusqlite`, `ring` | Async, safe, encrypted |
| **Flutter Frontend** | Riverpod, `fl_chart`, `go_router` | Clean state, smooth UI |
| **Database** | SQLite + AES-GCM | Embedded, encrypted, fast |
| **Communication** | Local IPC (Unix socket) | Zero-copy, no network exposure |
| **Crypto** | `ring` | Battle-tested, audited |

## Getting Started

### Prerequisites
- Rust 1.75+
- Flutter 3.0+
- macOS 13+ / Android API 26+ / Linux

### Build

```bash
# Clone the repo
git clone https://github.com/synth/open_health.git
cd open_health

# Build the Rust backend
cd rust
cargo build --release

# Build the Flutter app
cd ../flutter
flutter build apk --release  # Android
flutter build ios --release  # iOS
```

### Import Data

```bash
# Place your CSV exports in the data directory
mkdir -p data/imports
cp ~/Downloads/fitbit-export.csv data/imports/
cp ~/Downloads/oura-export.json data/imports/

# Import via the app UI or CLI
cargo run --bin import -- data/imports/
```

## Architecture

### Rust Backend (`rust/`)

```
rust/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Library root
│   ├── crypto.rs       # Encryption utilities
│   ├── db/             # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs   # SQLite schema
│   │   └── queries.rs  # Typed SQL queries
│   ├── import/         # Data import pipeline
│   │   ├── mod.rs
│   │   ├── fitbit.rs
│   │   ├── oura.rs
│   │   ├── apple_health.rs
│   │   └── generic_csv.rs
│   ├── server/         # Local IPC server
│   │   ├── mod.rs
│   │   └── handler.rs
│   └── stats/          # Aggregation & analytics
│       ├── mod.rs
│       └── trends.rs
```

### Flutter Frontend (`flutter/`)

```
flutter/
├── pubspec.yaml
├── lib/
│   ├── main.dart
│   ├── app.dart              # App shell & routing
│   ├── themes/               # Synthwave neon themes
│   ├── screens/              # Dashboard, imports, reports
│   ├── widgets/              # Chart components, cards
│   ├── providers/            # Riverpod state
│   ├── services/             # IPC bridge to Rust
│   └── utils/                # Helpers
└── test/
    ├── unit/
    ├── integration/
    └── widgets/
```

## Development

### Running Tests

```bash
# Rust tests
cd rust && cargo test

# Flutter tests
cd flutter && flutter test
```

### Code Style

- **Rust:** `cargo fmt` + `cargo clippy -- -D warnings`
- **Flutter:** `dart format .` + `flutter analyze`

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repo
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with love by **synth** 🎹🦞 — [synthclaw](https://github.com/synth)

Part of **The Neon Stack** — three open-source apps, one ecosystem.

---

*Privacy is a right. Health data shouldn't be an exception.*
