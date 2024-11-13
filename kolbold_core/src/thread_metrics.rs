//! # Thread Metrics Module
//!
//! This module provides structures for collecting and displaying thread-level metrics,
//! including CPU and memory usage along with execution time. These metrics are useful
//! for analyzing the performance of multi-threaded applications by capturing data
//! for each individual thread.
//!
//! ## Overview
//! - `TimeThreadMetrics`: Represents time-based metrics for a specific thread, capturing CPU usage
//!   and execution time.
//! - `MemoryThreadMetrics`: Represents memory-based metrics for a specific thread, capturing memory
//!   usage and execution time.
//!
//! ## Example Usage
//! ```rust
//! use kolbold_core::thread_metrics::{TimeThreadMetrics, MemoryThreadMetrics};
//! use smol_str::SmolStr;
//!
//! // Example for creating a TimeThreadMetrics instance
//! let time_metrics = TimeThreadMetrics::new(
//!     SmolStr::new("thread-1"),
//!     45.0, // CPU usage percentage
//!     1500, // Execution time in milliseconds
//! );
//! println!("{}", time_metrics); // Displays: "Thread ID: thread-1 | CPU usage: 45.00% | Execution Time: 1500 ms"
//!
//! // Example for creating a MemoryThreadMetrics instance
//! let memory_metrics = MemoryThreadMetrics::new(
//!     SmolStr::new("thread-1"),
//!     204800, // Memory usage in bytes
//!     100520 // Swap usage in bytes
//!     1500,   // Execution time in milliseconds
//! );
//! println!("{}", memory_metrics); // Displays: "Thread ID: thread-1 | Memory usage: 200 KB | Execution Time: 1500 ms"
//! ```
//!
//! ## Structs
//! - `TimeThreadMetrics`: Captures metrics for CPU usage and execution time specific to a thread.
//! - `MemoryThreadMetrics`: Captures metrics for memory usage and execution time specific to a thread.
//!
//! ## Usage Notes
//! These structs are designed to facilitate thread-level performance tracking, enabling
//! benchmarking and tuning in multi-threaded environments. They are especially useful
//! for applications requiring granular insights into resource consumption per thread.

use smol_str::SmolStr;
use std::fmt;

#[derive(Debug)]
pub struct TimeThreadMetrics {
    /// Unique identifier for the thread.
    thread_id: SmolStr,
    /// CPU usage percentage during the measurement period.
    cpu_usage: f32,
    /// Total execution time in milliseconds for the thread.
    execution_time: u64,
}

impl fmt::Display for TimeThreadMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Thread ID: {} | CPU usage: {:.2}% | Execution Time: {} ms",
            self.thread_id, self.cpu_usage, self.execution_time,
        )
    }
}

impl TimeThreadMetrics {
    /// Creates a new instance of `TimeThreadMetrics`.
    ///
    /// # Parameters
    /// - `thread_id`: Unique identifier for the thread.
    /// - `cpu_usage`: CPU usage percentage during the measurement period.
    /// - `execution_time`: Total execution time in milliseconds for the thread.
    ///
    /// # Returns
    /// A new `TimeThreadMetrics` instance.
    pub fn new(thread_id: SmolStr, cpu_usage: f32, execution_time: u64) -> Self {
        Self {
            thread_id,
            cpu_usage,
            execution_time,
        }
    }
}

#[derive(Debug)]
pub struct MemoryThreadMetrics {
    /// Unique identifier for the thread.
    thread_id: SmolStr,
    /// Memory usage in bytes during the measurement period.
    memory_usage: u64,
    /// Swap usage in bytes during measurement period.
    swap_usage: u64,
    /// Total execution time in milliseconds for the thread.
    execution_time: u64,
}

impl fmt::Display for MemoryThreadMetrics {
    /// Formats the `MemoryThreadMetrics` for display, showing thread ID, memory usage, swap usage and execution time.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Thread ID: {} | Memory usage: {} KB | Swap usage: {} KB | Execution Time: {} ms",
            self.thread_id, self.memory_usage, self.swap_usage, self.execution_time,
        )
    }
}

impl MemoryThreadMetrics {
    /// Creates a new instance of `MemoryThreadMetrics`.
    ///
    /// # Parameters
    /// - `thread_id`: Unique identifier for the thread.
    /// - `memory_usage`: Memory usage in bytes during the measurement period.
    /// - `spaw_usage`: Usage of swap memory in bytes during measurement period.
    /// - `execution_time`: Total execution time in milliseconds for the thread.
    ///
    /// # Returns
    /// A new `MemoryThreadMetrics` instance.
    pub fn new(
        thread_id: SmolStr,
        memory_usage: u64,
        swap_usage: u64,
        execution_time: u64,
    ) -> Self {
        Self {
            thread_id,
            memory_usage,
            swap_usage,
            execution_time,
        }
    }
}
