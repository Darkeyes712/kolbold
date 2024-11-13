//! # System Metrics Collection Module
//!
//! This module provides traits for collecting and locking system metrics related to time and memory usage.
//! The traits are designed for use in both synchronous and asynchronous contexts, supporting flexible usage
//! in performance measurement and resource monitoring scenarios within concurrent applications.
//!
//! ## Overview
//! - Defines core traits for gathering and managing CPU and memory usage metrics in Rust applications.
//! - Provides locking traits to ensure safe, thread-safe access to shared metrics in both sync and async contexts.
//! - Uses `MetricError` for error handling, ensuring robust error propagation in metric collection operations.
//!
//! ## Traits
//! ### Synchronous Metric Collection
//! - **`TimeSysMetricsCollectorSync`**: Defines methods for collecting CPU metrics in synchronous contexts.
//! - **`MemorySysMetricsCollectorSync`**: Defines methods for collecting memory metrics in synchronous contexts.
//!
//! ### Asynchronous Metric Collection
//! - **`TimeSysMetricsCollectorAsync`**: Defines methods for collecting CPU metrics in asynchronous contexts.
//! - **`MemorySysMetricsCollectorAsync`**: Defines methods for collecting memory metrics in asynchronous contexts.
//!
//! ### Locking Mechanisms
//! - **`TimeCollectorLock`**: Provides a locking mechanism for synchronously accessing time metrics collectors.
//! - **`TimeCollectorLockAsync`**: Provides a locking mechanism for asynchronously accessing time metrics collectors.
//! - **`MemoryCollectorLock`**: Provides a locking mechanism for synchronously accessing memory metrics collectors.
//! - **`MemoryCollectorLockAsync`**: Provides a locking mechanism for asynchronously accessing memory metrics collectors.
//!
//! ## Example Usage
//! ```rust
//! use kolbold_core::system_metrics::TimeSysMetricsCollectorSync;
//! use kolbold_core::error::MetricError;
//! use anyhow::Result;
//!
//! struct ExampleCollector;
//!
//! impl TimeSysMetricsCollectorSync for ExampleCollector {
//!     fn refresh_cpu(&mut self) {
//!         // Implementation for refreshing CPU metrics
//!     }
//!
//!     fn cpu_usage(&self) -> Result<f32, MetricError> {
//!         // Implementation for retrieving CPU usage
//!         Ok(42.0) // Example CPU usage
//!     }
//! }
//!
//! let mut collector = ExampleCollector;
//! collector.refresh_cpu();
//! match collector.cpu_usage() {
//!     Ok(usage) => println!("CPU usage: {:.2}%", usage),
//!     Err(e) => eprintln!("Error retrieving CPU usage: {}", e),
//! }
//! ```
//!
//! ## Error Handling
//! - The `MetricError` type is used for consistent error handling across all metrics collection functions that return a `Result`.
//! - `MetricError` encapsulates various error cases, such as lock acquisition failures and metric retrieval errors, making it easy to handle failures gracefully.
//!
//! ## Integration Notes
//! - This module’s traits are intended for use in performance monitoring and resource management in both synchronous and asynchronous Rust applications.
//! - The async versions of the traits (`TimeSysMetricsCollectorAsync` and `MemorySysMetricsCollectorAsync`) are designed to work with asynchronous runtimes like Tokio, making them suitable for high-performance, non-blocking applications.

use super::error::MetricError;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::MutexGuard;
use tokio::sync::MutexGuard as TokioMutexGuard;

pub trait TimeSysMetricsCollectorSync {
    /// Refreshes the current CPU metrics.
    fn refresh_cpu(&mut self);

    /// Retrieves the current CPU usage as a percentage.
    ///
    /// # Returns
    /// - `Ok(f32)` representing the CPU usage.
    /// - `Err(MetricError)` if the CPU usage cannot be retrieved.
    fn cpu_usage(&self) -> Result<f32, MetricError>;
}

#[async_trait]
pub trait TimeSysMetricsCollectorAsync {
    /// Asynchronously refreshes the current CPU metrics.
    async fn refresh_cpu(&mut self);

    /// Asynchronously retrieves the current CPU usage as a percentage.
    ///
    /// # Returns
    /// - A `f32` representing the CPU usage.
    async fn cpu_usage(&self) -> f32;
}

pub trait MemorySysMetricsCollectorSync {
    /// Refreshes the current memory metrics.
    fn refresh_memory(&mut self);

    /// Retrieves the current memory usage in bytes.
    ///
    /// # Returns
    /// - `Ok(u64)` representing memory usage in bytes.
    /// - `Err(MetricError)` if the memory usage cannot be retrieved.
    fn memory_usage(&self) -> Result<u64, MetricError>;

    /// Retrieves the current swap usage in bytes.
    ///
    /// # Returns
    /// - `Ok(u64)` representing swap usage in bytes.
    /// - `Err(MetricError)` if the swap usage cannot be retrieved.
    fn swap_usage(&self) -> Result<u64, MetricError>;
}

#[async_trait]
pub trait MemorySysMetricsCollectorAsync {
    /// Asynchronously refreshes the current memory metrics.
    async fn refresh_memory(&mut self);

    /// Asynchronously retrieves the current memory usage in bytes.
    ///
    /// # Returns
    /// - A `u64` representing memory usage in bytes.
    async fn memory_usage(&self) -> u64;

    /// Asynchronously retrieves the current swap usage in bytes.
    ///
    /// # Returns
    /// - A `u64` representing swap usage in bytes.
    async fn swap_usage(&self) -> u64;
}

pub trait TimeCollectorLock {
    /// The type of the collector being locked.
    type Collector: TimeSysMetricsCollectorSync;

    /// Locks the time metrics collector, ensuring safe concurrent access.
    ///
    /// # Returns
    /// - `Ok(MutexGuard<Self::Collector>)` for safe access to the collector.
    /// - `Err(MetricError)` if the lock cannot be acquired.
    fn lock_collector(&self) -> Result<MutexGuard<Self::Collector>, MetricError>;
}

#[async_trait]
pub trait TimeCollectorLockAsync {
    /// The type of the collector being locked.
    type Collector: TimeSysMetricsCollectorAsync;

    /// Asynchronously locks the time metrics collector, ensuring safe concurrent access.
    ///
    /// # Returns
    /// - `Ok(TokioMutexGuard<Self::Collector>)` for safe access to the collector.
    /// - `Err(MetricError)` if the lock cannot be acquired.
    async fn lock_collector(&self) -> Result<TokioMutexGuard<Self::Collector>, MetricError>;
}

pub trait MemoryCollectorLock {
    /// The type of the collector being locked.
    type Collector: MemorySysMetricsCollectorSync;

    /// Locks the memory metrics collector, ensuring safe concurrent access.
    ///
    /// # Returns
    /// - `Ok(MutexGuard<Self::Collector>)` for safe access to the collector.
    /// - `Err(MetricError)` if the lock cannot be acquired.
    fn lock_collector(&self) -> Result<MutexGuard<Self::Collector>, MetricError>;
}

#[async_trait]
pub trait MemoryCollectorLockAsync {
    /// The type of the collector being locked.
    type Collector: MemorySysMetricsCollectorAsync;

    /// Asynchronously locks the memory metrics collector, ensuring safe concurrent access.
    ///
    /// # Returns
    /// - `Ok(TokioMutexGuard<Self::Collector>)` for safe access to the collector.
    /// - `Err(MetricError)` if the lock cannot be acquired.
    async fn lock_collector(&self) -> Result<TokioMutexGuard<Self::Collector>, MetricError>;
}
