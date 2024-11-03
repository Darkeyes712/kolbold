//! # Performance Measurement Library
//!
//! This library provides tools for measuring the time and memory complexity of code, supporting both
//! synchronous and asynchronous operations. It is designed for developers who need to benchmark
//! their code in multi-threaded or single-threaded environments.
//!
//! ## Modules
//! - `error`: Custom error handling types used across the library.
//! - `memory`: Functions and traits for memory complexity measurement.
//! - `system_metrics`: Structures and traits for collecting system-level metrics.
//! - `thread_metrics`: Data structures for holding thread-specific metric data.
//! - `time`: Functions and traits for time complexity measurement.
//! - `utils`: Utility functions used throughout the library.
//!
//! ## Re-exports
//! The library re-exports the most commonly used traits and structs for easier access:
//! - `MemoryComplexity` and `MemoryMeasurement` for memory complexity measurement.
//! - `TimeComplexity` and `TimeMeasurement` for time complexity measurement.
//!
//! ## Example Usage
//! ```rust
//! use your_crate::{TimeComplexity, MemoryComplexity};
//!
//! // Measure time complexity of a single-threaded synchronous process
//! TimeMeasurement::measure_single_thread_sync(|| {
//!     // Your process to benchmark
//! })
//! .map(|result| println!("Time measurement successful: {:?}", result))
//! .unwrap_or_else(|e| eprintln!("Failed to measure time complexity: {}", e));
//!
//! // Measure memory complexity of an async process
//! # async {
//! MemoryMeasurement::measure_single_thread_async(|| {
//!     // Your async process to benchmark
//! })
//! .await
//! .map(|result| println!("Memory measurement successful: {:?}", result))
//! .unwrap_or_else(|e| eprintln!("Failed to measure memory complexity: {}", e));
//! # };
//! ```
//!
//! ## Getting Started
//! - Import the `TimeMeasurement` or `MemoryMeasurement` structs for access to measurement methods.
//! - Use the traits `TimeComplexity` and `MemoryComplexity` to implement custom measurement strategies if needed.

pub mod error;
pub mod memory;
pub mod system_metrics;
pub mod thread_metrics;
pub mod time;
pub mod utils;

// Re-export primary traits and structs for convenience
pub use memory::memory_measurement::MemoryComplexity;
pub use memory::memory_measurement::MemoryMeasurement;
pub use time::time_measurement::TimeComplexity;
pub use time::time_measurement::TimeMeasurement;
