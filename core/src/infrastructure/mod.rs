//! # Infrastructure Layer
//!
//! Concrete implementations of port interfaces.
//! Provides adapters for different platforms.

pub mod memory_storage;
pub mod simple_diff;
pub mod sha256_hasher;

pub use memory_storage::MemoryStorage;
pub use simple_diff::SimpleDiff;
pub use sha256_hasher::Sha256Hasher;
