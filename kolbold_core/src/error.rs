//! # Error Handling Module
//!
//! This module defines custom error types for handling various failures and exceptions that may occur
//! during time and memory complexity measurements within the library. By encapsulating errors in
//! specific types, it provides clear and consistent error messages across the library's components.
//!
//! ## Overview
//! - Defines structured error types for handling common issues in time and memory metrics collection.
//! - Integrates with `anyhow::Result` and `thiserror::Error` to provide seamless error handling in user applications.
//!
//! ## Error Types
//! - `MetricError`: The primary error type that wraps other specific errors (e.g., `TimeError`, `MemoryError`) and
//!   also includes general errors related to system locks and thread management.
//! - `TimeError`: Specific errors encountered during time metrics collection, such as missing CPU information or arithmetic overflow.
//! - `MemoryError`: Specific errors encountered during memory metrics collection, such as failure to collect metrics.
//!
//! ## Key Features
//! - Implements the `thiserror::Error` trait, which provides detailed and readable error descriptions for each error type.
//! - Implements conversions from standard Rust error types (`anyhow::Error`, `tokio::task::JoinError`) into `MetricError`,
//!   allowing for streamlined error propagation in asynchronous and synchronous contexts.
//!
//! ## Example Usage
//! ```rust
//! use kolbold_core::error::MetricError;
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
//! - `MetricError` and its related error types integrate seamlessly with `anyhow::Result`, making it easy to handle errors in applications that already use the `anyhow` crate.
//! - The module’s error types are compatible with async tasks managed by Tokio, handling errors from async task joins and other async operations through `JoinError` conversion.
//!
//! ## Detailed Error Types
//! - `MetricError::LockError`: Used when a system lock fails, often due to issues with a `MutexGuard`.
//! - `MetricError::AsyncJoinError`: Represents errors that occur when joining an asynchronous task fails, e.g., during concurrent operations with Tokio.
//! - `MetricError::SyncJoinError`: Represents errors when joining synchronous threads fails.
//! - `TimeError`: Specific to time metrics and includes errors for missing CPU information or arithmetic overflow.
//! - `MemoryError`: Specific to memory metrics and includes errors for memory collection failures.
//!

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
