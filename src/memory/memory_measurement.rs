//! # Memory Measurement Module
//!
//! This module provides tools and traits for measuring memory usage in both synchronous and asynchronous
//! contexts. It includes capabilities for single-threaded and multi-threaded memory complexity measurements,
//! making it useful for performance analysis and benchmarking in Rust applications.
//!
//! ## Overview
//! - `MemoryComplexity` trait: Defines methods for measuring memory usage in different scenarios (sync/async, single/multi-threaded).
//! - `MemoryMeasurement`: Struct that implements `MemoryComplexity` to provide actual functionality for memory measurements.
//!
//! ## Features
//! - Synchronous and asynchronous memory measurement support.
//! - Single-threaded and multi-threaded metric collection.
//! - Integration with `SystemMetricsCollectorSync` and `SystemMetricsCollectorAsync` for comprehensive data collection.
//!
//! ## Example Usage
//! ```rust
//! use kolbold::memory::memory_measurement::{MemoryComplexity, MemoryMeasurement};
//! use anyhow::Result;
//!
//! fn example_sync_measurement() -> Result<()> {
//!     let data = MemoryMeasurement::measure_single_thread_sync(|| {
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
//!     let data = MemoryMeasurement::measure_single_thread_async(|| {
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
//! - The module relies on `sysinfo` for system-level memory metrics collection.
//! - The `MemoryMeasurementData` struct provides a structured representation of collected data, including memory usage and timestamps.
//!
//! ## Test Suite
//! The module includes a test suite that verifies the functionality of both synchronous and asynchronous memory measurements,
//! ensuring that the implementations work as expected for single-threaded and multi-threaded scenarios.

use super::memory_metrics_collector::{
    MemoryMeasurementData, SingleSystemMetricsCollector, SystemMetricsCollectorAsync,
    SystemMetricsCollectorSync,
};

use anyhow::Result;
use async_trait::async_trait;
use std::{
    fmt::{self},
    sync::{Arc, Mutex},
    time::{Instant as SyncInstant, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::Mutex as AsyncMutex, time::Instant as AsyncInstant};

#[async_trait]
pub trait MemoryComplexity {
    /// Measures synchronous, single-threaded code.
    fn measure_single_thread_sync<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug;

    /// Measures synchronous, multi-threaded code.
    fn measure_multi_thread_sync<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static;

    /// Measures asynchronous, single-threaded code.
    async fn measure_single_thread_async<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send;

    /// Measures asynchronous, multi-threaded code.
    async fn measure_multi_thread_async<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static;
}

pub struct MemoryMeasurement;

#[async_trait]
impl MemoryComplexity for MemoryMeasurement {
    fn measure_single_thread_sync<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_mem_usage = system.refresh_initial_metrics()?;
        let start_time = SystemTime::now();
        let start_instant = SyncInstant::now();

        // Execute the process
        process();

        let avg_mem_usage = system.refresh_final_metrics(initial_mem_usage)?;

        let end_time = SystemTime::now();
        let elapsed_time = start_instant.elapsed().as_millis() as u64;

        let start_time_millis = start_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let end_time_millis = end_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            None,
        ))
    }

    fn measure_multi_thread_sync<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_mem_usage = system.refresh_initial_metrics()?;
        let start_time = SystemTime::now();
        let start_instant = SyncInstant::now();

        let collector = SystemMetricsCollectorSync::new();
        let process = Arc::new(Mutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorSync::collect_thread_sys_metrics(&collector, process)?;

        let avg_mem_usage = system.refresh_final_metrics(initial_mem_usage)?;

        let end_time = SystemTime::now();
        let elapsed_time = start_instant.elapsed().as_millis() as u64;
        println!("elapsed time: {}", elapsed_time);

        let start_time_millis = start_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let end_time_millis = end_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            Some(thread_data),
        ))
    }

    async fn measure_single_thread_async<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_mem_usage = system.refresh_initial_metrics()?;
        let start_time = SystemTime::now();
        let start_instant = SyncInstant::now();

        // Execute the process
        process();

        let avg_mem_usage = system.refresh_final_metrics(initial_mem_usage)?;

        let end_time = SystemTime::now();
        let elapsed_time = start_instant.elapsed().as_millis() as u64;

        let start_time_millis = start_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let end_time_millis = end_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            None,
        ))
    }

    async fn measure_multi_thread_async<F, R>(process: F) -> Result<MemoryMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_mem_usage = system.refresh_initial_metrics()?;

        let start_time = SystemTime::now();
        let start_instant = AsyncInstant::now();

        let collector = SystemMetricsCollectorAsync::new();
        let process = Arc::new(AsyncMutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorAsync::collect_thread_sys_metrics_async(&collector, process)
                .await?;

        let avg_mem_usage = system.refresh_final_metrics(initial_mem_usage)?;

        let end_time = SystemTime::now();
        let elapsed_time = start_instant.elapsed().as_millis() as u64;

        let start_time_millis = start_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let end_time_millis = end_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Ok(MemoryMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_mem_usage,
            Some(thread_data),
        ))
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::{MemoryComplexity, MemoryMeasurement, *};

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
        let data =
            MemoryMeasurement::measure_single_thread_sync(create_single_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    #[test]
    fn test_sync_multi_thread_code_print() -> Result<()> {
        let data =
            MemoryMeasurement::measure_multi_thread_sync(create_multi_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_async_single_thread_code_print() -> Result<()> {
        let data =
            MemoryMeasurement::measure_single_thread_sync(create_single_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    /// Change number of workers based on architecture of running machine
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn test_async_multi_thread_code_print() -> Result<()> {
        let data =
            MemoryMeasurement::measure_multi_thread_async(create_multi_thread_test_conditions)
                .await;

        assert!(data.is_ok());

        Ok(())
    }
}
