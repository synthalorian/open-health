//! open_health — Privacy-first health data engine.
//!
//! Local-only SQLite database with AES-GCM encryption.
//! Import CSV from any major health tracker.

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::doc_missing_intra_doc_links)]

pub mod crypto;
pub mod db;
pub mod import;
pub mod server;
pub mod stats;

pub use crypto::*;
pub use db::*;
pub use import::*;
pub use server::*;
pub use stats::*;

/// Application version, injected at compile time via Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name.
pub const APP_NAME: &str = "open_health";

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
