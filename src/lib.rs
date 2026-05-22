//! SFLOW API — Public embedding interface for structured.world products.
//!
//! **License:** MIT/Apache-2.0 (dual-licensed).
//!
//! This crate defines ONLY traits and types — zero implementation logic.
//! Products depend on this crate for a clean license boundary:
//! - AGPL products (SID CE) import this MIT crate — no contamination
//! - sflow-engine (proprietary) implements these traits
//!
//! Usage in product Cargo.toml:
//! ```toml
//! [dependencies]
//! sflow-api = { git = "https://github.com/structured-world/sflow-api.git", tag = "v0.1.0" }
//! # OR via crates.io once published:
//! # sflow-api = "0.1"
//! ```
//!
//! Products use `WorkflowBackend` trait for a pluggable interface
//! (can swap for Temporal/Camunda), or `WorkflowEngine` for full SFLOW API.

pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used items at crate root
pub use error::SflowError;
pub use traits::*;
pub use types::*;
