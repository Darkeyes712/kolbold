//! # Kolbold - Performance Measurement Library
//!
//! **Kolbold** is a comprehensive Rust library designed to measure the time and memory complexity of code execution,
//! offering tools for tracking CPU and memory usage in both single-threaded and multi-threaded contexts.
//! It supports synchronous and asynchronous code execution, making it versatile for various performance analysis needs.
//!
//! This library is particularly suited for developers who need to profile and benchmark Rust applications, with detailed metrics available for
//! time and memory usage across different runtime contexts.
//!
//! ## Core Modules
//!
//! - [`kolbold_core`](crate::kolbold_core): Contains the core functionality for time and memory measurements, including the `TimeComplexity` and `MemoryComplexity` traits,
//!   and structs for capturing these metrics (`TimeMeasurement`, `MemoryMeasurement`).
//! - [`kolbold_macros`](crate::kolbold_macros): Provides procedural macros used in simplifying metric implementations.
//!
//! ## Primary Features
//!
//! - **Time Complexity Measurement**: Track CPU usage and execution time in both synchronous and asynchronous contexts.
//! - **Memory Complexity Measurement**: Capture memory usage during function execution, with support for synchronous and asynchronous measurement.
//! - **Thread-Specific Metrics**: Detailed metrics for individual threads in multi-threaded environments, aiding in granular performance analysis.
//! - **Re-Exports for Convenience**: Common traits and structs, such as `TimeComplexity`, `MemoryComplexity`, `TimeMeasurement`, and `MemoryMeasurement`, are re-exported for ease of use.
//!
//! ## Example Usage
//!
//! ```rust
//! use kolbold_core::{TimeComplexity, MemoryComplexity, TimeMeasurement, MemoryMeasurement};
//! use anyhow::Result;
//!
//! // Measure time complexity of a single-threaded synchronous function
//! fn measure_sync_time() -> Result<()> {
//!     TimeMeasurement::measure_single_thread_sync::<_, _, TimeMeasurement>(|| {
//!         // Example process to measure
//!         let sum: u64 = (0..1_000_000).sum();
//!         println!("Sum: {}", sum);
//!     })
//!     .map(|result| println!("Time measurement successful: {:?}", result))
//!     .unwrap_or_else(|e| eprintln!("Failed to measure time complexity: {}", e));
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn measure_async_memory() -> Result<()> {
//!     // Measure memory complexity of an async function
//!     MemoryMeasurement::measure_single_thread_async::<_, _, MemoryMeasurement>(|| {
//!         // Async process to measure
//!         let vec: Vec<u64> = (0..1_000_000).collect();
//!         println!("Vector length: {}", vec.len());
//!     })
//!     .await
//!     .map(|result| println!("Memory measurement successful: {:?}", result))
//!     .unwrap_or_else(|e| eprintln!("Failed to measure memory complexity: {}", e));
//!     Ok(())
//! }
//! ```
//!
//! ## Getting Started
//!
//! 1. Add `kolbold` as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! kolbold = "1.1.0"
//! ```
//!
//! 2. Import the measurement structs (`TimeMeasurement`, `MemoryMeasurement`) for access to time and memory measurement methods.
//!
//! 3. If needed, extend functionality by implementing `TimeComplexity` and `MemoryComplexity` traits in your custom types.
//!
//! ## More Information
//!
//! For details on each component, refer to the documentation for:
//! - [`kolbold_core`](crate::kolbold_core): Core measurement functionalities
//! - [`kolbold_macros`](crate::kolbold_macros): Macros that simplify trait implementations

pub use kolbold_core;
pub use kolbold_macros;
