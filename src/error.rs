//! # Error Handling Module
//!
//! This module provides custom error types and implementations used throughout the library for handling
//! various errors that may occur during time and memory complexity measurements.
//!
//! ## Overview
//! - Defines comprehensive error types to standardize error handling across different components of the library.
//! - Implements conversion from common error types to `MetricError` for seamless integration with other Rust libraries.
//!
//! ## Error Types
//! - `MetricError`: The main error type that encapsulates different categories of errors, including those related to
//!   system locks, async and sync task failures, and unexpected conditions.
//! - `TimeError`: Specific errors related to time complexity measurements, such as missing CPU information or arithmetic overflow.
//! - `MemoryError`: Specific errors related to memory complexity measurements, such as failures in memory collection.
//!
//! ## Key Features
//! - Implements the `thiserror::Error` trait for easy-to-read error descriptions.
//! - Provides conversions from `anyhow::Error` and `tokio::task::JoinError` to `MetricError`.
//!
//! ## Example Usage
//! ```rust
//! use kolbold::error::MetricError;
//! use anyhow::Result;
//!
//! fn perform_measurement() -> Result<(), MetricError> {
//!     // Simulate a measurement operation that might fail
//!     Err(MetricError::LockError("Failed to acquire lock".into()))
//! }
//!
//! match perform_measurement() {
//!     Ok(_) => println!("Measurement successful"),
//!     Err(e) => eprintln!("Error occurred: {}", e),
//! }
//! ```
//!
//! ## Integration
//! The module's error types are designed to integrate seamlessly with `anyhow::Result` for flexible error handling
//! in applications that use the `anyhow` crate.

use anyhow::Error as AnyhowError;
use smol_str::{SmolStr, ToSmolStr};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum MetricError {
    /// Error occurring when a system lock fails, typically due to inability to get a lock on a `MutexGuard`.
    #[error("System Lock failed. Could not get a lock on MutexGuard")]
    LockError(SmolStr),

    /// Error occurring when joining an async task fails.
    #[error("Failed to join async threads")]
    AsyncJoinError(JoinError),

    /// Error occurring when joining a synchronous thread fails, with the provided error message.
    #[error("Could not join threads, err: {0:?}")]
    SyncJoinError(SmolStr),

    /// Wrapper for errors related to time-specific metric collection.
    #[error("Time-specific error: {0}")]
    TimeError(#[from] TimeError),

    /// Wrapper for errors related to memory-specific metric collection.
    #[error("Memory-specific error: {0}")]
    MemoryError(#[from] MemoryError),

    /// General catch-all error for unexpected conditions, carrying a string representation of the error.
    #[error("Unexpected error {0}")]
    Other(SmolStr),
}

impl From<AnyhowError> for MetricError {
    /// Converts an `anyhow::Error` into a `MetricError`, encapsulating it as an `Other` error type.
    fn from(error: AnyhowError) -> Self {
        MetricError::Other(error.to_smolstr())
    }
}

impl From<JoinError> for MetricError {
    /// Converts a `tokio::task::JoinError` into a `MetricError`, encapsulating it as an `AsyncJoinError`.
    fn from(error: JoinError) -> Self {
        MetricError::AsyncJoinError(error)
    }
}

#[derive(Debug, Error)]
pub enum TimeError {
    /// Error indicating that no CPUs were found in the system.
    #[error("No CPUs found")]
    MissingCpuError,

    /// Error indicating an unexpected arithmetic overflow during metric collection.
    #[error("Unexpected overflow occurred")]
    OverflowError,
}

#[derive(Debug, Error)]
pub enum MemoryError {
    /// Error indicating failure in collecting memory metrics.
    #[error("Memory collection failed")]
    CollectionError,
}
