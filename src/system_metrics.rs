//! # System Metrics Collection Module
//!
//! This module provides traits for collecting and locking system metrics related to time and memory usage.
//! The traits are implemented for both synchronous and asynchronous contexts, allowing for flexible usage
//! in different performance measurement scenarios.
//!
//! ## Overview
//! - The module defines traits for gathering CPU and memory usage metrics and locking mechanisms to ensure
//!   thread-safe access to these metrics.
//! - Traits include methods for refreshing metrics and retrieving usage data, with error handling built-in
//!   through the `MetricError` type.
//!
//! ## Traits
//! - `TimeSysMetricsCollectorSync`: For collecting CPU metrics synchronously.
//! - `TimeSysMetricsCollectorAsync`: For collecting CPU metrics asynchronously.
//! - `MemorySysMetricsCollectorSync`: For collecting memory metrics synchronously.
//! - `MemorySysMetricsCollectorAsync`: For collecting memory metrics asynchronously.
//! - `TimeCollectorLock`: Provides a locking mechanism for synchronous time metrics collectors.
//! - `TimeCollectorLockAsync`: Provides a locking mechanism for asynchronous time metrics collectors.
//! - `MemoryCollectorLock`: Provides a locking mechanism for synchronous memory metrics collectors.
//! - `MemoryCollectorLockAsync`: Provides a locking mechanism for asynchronous memory metrics collectors.
//!
//! ## Example Usage
//! ```rust
//! use kolbold::system_metrics::TimeSysMetricsCollectorSync;
//! use anyhow::Result;
//!
//! struct ExampleCollector;
//!
//! impl TimeSysMetricsCollectorSync for ExampleCollector {
//!     fn refresh_cpu(&mut self) {
//!         // Implementation for refreshing CPU metrics
//!     }
//!
//!     fn cpu_usage(&self) -> Result<f32, kolbold::error::MetricError> {
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
//! - All functions that return a `Result` type handle errors through `MetricError`, making it easy to handle
//!   failures during metric collection or locking operations.

use super::error::MetricError;
use anyhow::Result;
use std::sync::MutexGuard;
use tokio::sync::MutexGuard as TokioMutexGuard;

use async_trait::async_trait;

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
}

#[async_trait]
#[async_trait]
pub trait MemorySysMetricsCollectorAsync {
    /// Asynchronously refreshes the current memory metrics.
    async fn refresh_memory(&mut self);

    /// Asynchronously retrieves the current memory usage in bytes.
    ///
    /// # Returns
    /// - A `u64` representing memory usage in bytes.
    async fn memory_usage(&self) -> u64;
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

#[async_trait::async_trait]
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

#[async_trait::async_trait]
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
