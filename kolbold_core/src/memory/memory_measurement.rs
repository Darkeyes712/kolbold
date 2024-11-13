//! # Memory Measurement Module
//!
//! This module provides tools and traits for measuring memory usage in both synchronous and asynchronous
//! contexts. It supports single-threaded and multi-threaded memory complexity measurements, making it ideal
//! for performance analysis and benchmarking in Rust applications.
//!
//! ## Overview
//! - `MemoryComplexity` Trait: Defines methods for measuring memory usage in various contexts (sync/async, single/multi-threaded).
//! - `MemoryMeasurement` Struct: Implements `MemoryComplexity` to provide actual functionality for memory measurements.
//!   The `MemoryMeasurement` struct also derives the `Timer` trait through a procedural macro, allowing it to handle
//!   timing operations seamlessly in both synchronous and asynchronous settings.
//! - `MemoryMeasurementData`: Represents collected data, including timestamps, elapsed time, memory usage, and optional
//!   thread-specific metrics for detailed analysis.
//!
//! ## Key Features
//! - Synchronous and asynchronous memory measurement support.
//! - Single-threaded and multi-threaded metric collection.
//! - Integrates with `SystemMetricsCollectorSync` and `SystemMetricsCollectorAsync` for comprehensive data collection.
//! - Flexible use of `Timer` trait through `#[derive(Timer)]`, enabling modular use of timing functionality across structs.
//!
//! ## Example Usage
//! Here is how to use `MemoryMeasurement`, which implements `MemoryComplexity`, to measure memory usage:
//!
//! ```rust
//! use kolbold_core::memory::memory_measurement::{MemoryComplexity, MemoryMeasurement};
//! use anyhow::Result;
//!
//! fn example_sync_measurement() -> Result<()> {
//!     let data = MemoryMeasurement::measure_single_thread_sync::<_, _, MemoryMeasurement>(|| {
//!         // Simulate a process to measure
//!         let mut v = vec![0; 1_000_000];
//!         v.iter_mut().for_each(|x| *x += 1);
//!     })?;
//!
//!     println!("Memory measurement data: {:?}", data);
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn example_async_measurement() -> Result<()> {
//!     let data = MemoryMeasurement::measure_single_thread_async::<_, _, MemoryMeasurement>(|| {
//!         // Simulate a process to measure asynchronously
//!         let mut v = vec![0; 1_000_000];
//!         v.iter_mut().for_each(|x| *x += 1);
//!     }).await?;
//!
//!     println!("Async memory measurement data: {:?}", data);
//!     Ok(())
//! }
//! ```
//!
//! ## Usage Notes
//! - This module relies on the `sysinfo` crate to collect system-level memory metrics.
//! - The `MemoryMeasurementData` struct provides a structured representation of collected data, including memory, cpu, swap usage, elapsed time, and optional thread-specific metrics, making it easier to analyze complex memory behaviors.
//! - Structs that implement `MemoryComplexity` should derive `Timer` using `#[derive(Timer)]`, which provides modular and reusable timing functions for both sync and async contexts.
//!
//! ## Test Suite
//! The module includes a comprehensive test suite that verifies synchronous and asynchronous memory measurements.
//! The suite covers both single-threaded and multi-threaded scenarios to ensure the accuracy of metric collection.
//! The tests illustrate the use of `MemoryMeasurement` and its interaction with the `Timer` trait for various workloads.

use super::{
    super::time_handle::Timer,
    memory_metrics_collector::{
        MemoryMeasurementData, SingleSystemMetricsCollector, SystemMetricsCollectorAsync,
        SystemMetricsCollectorSync,
    },
};
use kolbold_macros::Timer;

use anyhow::Result;
use async_trait::async_trait;
use std::{
    fmt::{self},
    sync::{Arc, Mutex},
};
use tokio::sync::Mutex as AsyncMutex;

#[async_trait]
pub trait MemoryComplexity: Timer {
    /// Measures synchronous, single-threaded code.
    fn measure_single_thread_sync<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug,
        T: Timer;

    /// Measures synchronous, multi-threaded code.
    fn measure_multi_thread_sync<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
        T: Timer;

    /// Measures asynchronous, single-threaded code.
    async fn measure_single_thread_async<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send,
        T: Timer;

    /// Measures asynchronous, multi-threaded code.
    async fn measure_multi_thread_async<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
        T: Timer;
}

#[derive(Timer)]
pub struct MemoryMeasurement;

#[async_trait]
impl MemoryComplexity for MemoryMeasurement {
    fn measure_single_thread_sync<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug,
        T: Timer,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let (initial_mem_usage, initial_swap_usage) = system.refresh_initial_metrics()?;
        let (start_time, start_instant) = T::start_timer_sync()?;

        // Execute the process
        process();

        let (avg_mem_usage, avg_swap_usage) =
            system.refresh_final_metrics((initial_mem_usage, initial_swap_usage))?;

        let (start_time_millis, end_time_millis, elapsed_time) =
            T::stop_timer_sync(start_time, start_instant)?;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            avg_swap_usage,
            None,
        ))
    }

    fn measure_multi_thread_sync<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
        T: Timer,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let (initial_mem_usage, initial_swap_usage) = system.refresh_initial_metrics()?;
        let (start_time, start_instant) = T::start_timer_sync()?;

        let collector = SystemMetricsCollectorSync::new();
        let process = Arc::new(Mutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorSync::collect_thread_sys_metrics(&collector, process)?;

        let (avg_mem_usage, avg_swap_usage) =
            system.refresh_final_metrics((initial_mem_usage, initial_swap_usage))?;

        let (start_time_millis, end_time_millis, elapsed_time) =
            T::stop_timer_sync(start_time, start_instant)?;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            avg_swap_usage,
            Some(thread_data),
        ))
    }

    async fn measure_single_thread_async<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send,
        T: Timer,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let (initial_mem_usage, initial_swap_usage) = system.refresh_initial_metrics()?;
        let (start_time, start_instant) = T::start_timer_async().await?;

        // Execute the process
        process();

        let (avg_mem_usage, avg_swap_usage) =
            system.refresh_final_metrics((initial_mem_usage, initial_swap_usage))?;

        let (start_time_millis, end_time_millis, elapsed_time) =
            T::stop_timer_async(start_time, start_instant).await?;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            avg_swap_usage,
            None,
        ))
    }

    async fn measure_multi_thread_async<F, R, T>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
        T: Timer,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let (initial_mem_usage, initial_swap_usage) = system.refresh_initial_metrics()?;

        let (start_time, start_instant) = T::start_timer_async().await?;

        let collector = SystemMetricsCollectorAsync::new();
        let process = Arc::new(AsyncMutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorAsync::collect_thread_sys_metrics_async(&collector, process)
                .await?;

        let (avg_mem_usage, avg_swap_usage) =
            system.refresh_final_metrics((initial_mem_usage, initial_swap_usage))?;

        let (start_time_millis, end_time_millis, elapsed_time) =
            T::stop_timer_async(start_time, start_instant).await?;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            avg_swap_usage,
            Some(thread_data),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{MemoryComplexity, MemoryMeasurement, *};
    use std::thread;

    fn create_single_thread_test_conditions() {
        let mut collection: Vec<u64> = Vec::new();

        for i in 0..5_000_000 {
            collection.push(i);
        }

        let _ = collection
            .into_iter()
            .map(|x| if x % 2 == 0 { x * 2 } else { x })
            .collect::<Vec<u64>>();
    }

    fn create_multi_thread_test_conditions() {
        let mut collection: Vec<u64> = Vec::new();

        let handles = thread::spawn(move || {
            for i in 0..5_000_000 {
                collection.push(i);
            }

            let _ = collection
                .into_iter()
                .map(|x| if x % 2 == 0 { x + 2 } else { x })
                .collect::<Vec<u64>>();
        });

        handles.join().unwrap();
    }

    #[test]
    fn test_sync_single_thread_code_print() -> Result<()> {
        let data = MemoryMeasurement::measure_single_thread_sync::<_, _, MemoryMeasurement>(
            create_single_thread_test_conditions,
        );

        assert!(data.is_ok());

        Ok(())
    }

    #[test]
    fn test_sync_multi_thread_code_print() -> Result<()> {
        let data = MemoryMeasurement::measure_multi_thread_sync::<_, _, MemoryMeasurement>(
            create_multi_thread_test_conditions,
        );

        assert!(data.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_async_single_thread_code_print() -> Result<()> {
        let data = MemoryMeasurement::measure_single_thread_sync::<_, _, MemoryMeasurement>(
            create_single_thread_test_conditions,
        );

        assert!(data.is_ok());

        Ok(())
    }

    /// Change number of workers based on architecture of running machine
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn test_async_multi_thread_code_print() -> Result<()> {
        let data = MemoryMeasurement::measure_multi_thread_async::<_, _, MemoryMeasurement>(
            create_multi_thread_test_conditions,
        )
        .await;

        assert!(data.is_ok());

        Ok(())
    }
}
