//! # Time Measurement Module for `kolbold`
//!
//! This module is designed to measure the time complexity of code execution by collecting CPU usage and time metrics
//! in both synchronous and asynchronous contexts. The module supports single-threaded and multi-threaded operations
//! to provide comprehensive performance insights.
//!
//! ## Overview
//! - `TimeComplexity` Trait: Defines methods for measuring code execution time and CPU usage in different environments.
//! - `TimeMeasurement` Struct: Implements `TimeComplexity` to provide concrete methods for time measurement.
//! - `TimeMeasurementData`: Represents the data collected during the time measurement process, including timestamps,
//!   CPU usage, and optional thread-specific metrics.
//!
//! ## Key Features
//! - Measure CPU usage and execution time for both single-threaded and multi-threaded code.
//! - Support for both synchronous and asynchronous execution environments.
//! - Collects detailed metrics to help analyze performance in various runtime scenarios.
//!
//! ## Example Usage
//! ```rust
//! use kolbold::time::time_measurement::{TimeComplexity, TimeMeasurement};
//! use anyhow::Result;
//!
//! fn sync_example() -> Result<()> {
//!     let data = TimeMeasurement::measure_single_thread_sync(|| {
//!         // Simulated workload
//!         let mut vec = vec![0; 1_000_000];
//!         vec.iter_mut().for_each(|x| *x += 1);
//!     })?;
//!
//!     println!("Synchronous time measurement data: {:?}", data);
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn async_example() -> Result<()> {
//!     let data = TimeMeasurement::measure_single_thread_async(|| {
//!         // Simulated asynchronous workload
//!         let mut vec = vec![0; 1_000_000];
//!         vec.iter_mut().for_each(|x| *x += 1);
//!     }).await?;
//!
//!     println!("Asynchronous time measurement data: {:?}", data);
//!     Ok(())
//! }
//! ```
//!
//! ## Usage Notes
//! - The module relies on the `sysinfo` crate to gather system-level metrics.
//! - The multi-threaded measurement methods provide detailed thread-specific data, which can help identify performance bottlenecks.
//! - Timestamps are measured in milliseconds relative to the UNIX epoch.
//!
//! ## Test Suite
//! The module includes a comprehensive test suite with cases for both single-threaded and multi-threaded performance
//! measurements, verifying the functionality of both synchronous and asynchronous code paths.

use super::time_metrics_collector::{
    SingleSystemMetricsCollector, SystemMetricsCollectorAsync, SystemMetricsCollectorSync,
    TimeMeasurementData,
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
pub trait TimeComplexity {
    /// Measures synchronous, single-threaded code.
    fn measure_single_thread_sync<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug;

    /// Measures synchronous, multi-threaded code.
    fn measure_multi_thread_sync<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static;

    /// Measures asynchronous, single-threaded code.
    async fn measure_single_thread_async<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send;

    /// Measures asynchronous, multi-threaded code.
    async fn measure_multi_thread_async<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static;
}

pub struct TimeMeasurement;

#[async_trait]
impl TimeComplexity for TimeMeasurement {
    fn measure_single_thread_sync<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R,
        R: fmt::Debug,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_cpu_usage = system.refresh_initial_metrics()?;
        let start_time = SystemTime::now();
        let start_instant = SyncInstant::now();

        // Execute the process
        process();

        let avg_cpu_usage = system.refresh_final_metrics(initial_cpu_usage)?;

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

        Ok(TimeMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_cpu_usage,
            None,
        ))
    }

    fn measure_multi_thread_sync<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_cpu_usage = system.refresh_initial_metrics()?;
        let start_time = SystemTime::now();
        let start_instant = SyncInstant::now();

        let collector = SystemMetricsCollectorSync::new();
        let process = Arc::new(Mutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorSync::collect_thread_sys_metrics(&collector, process)?;

        let avg_cpu_usage = system.refresh_final_metrics(initial_cpu_usage)?;

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

        Ok(TimeMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_cpu_usage,
            Some(thread_data),
        ))
    }

    async fn measure_single_thread_async<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send,
        R: fmt::Debug + Send,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_cpu_usage = system.refresh_initial_metrics()?;

        let start_time = SystemTime::now();
        let start_instant = AsyncInstant::now();

        // Execute the process
        process();

        let avg_cpu_usage = system.refresh_final_metrics(initial_cpu_usage)?;

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

        Ok(TimeMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_cpu_usage,
            None,
        ))
    }

    async fn measure_multi_thread_async<F, R>(process: F) -> Result<TimeMeasurementData>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: fmt::Debug + Send + 'static,
    {
        let mut system = SingleSystemMetricsCollector::new();
        let initial_cpu_usage = system.refresh_initial_metrics()?;

        let start_time = SystemTime::now();
        let start_instant = AsyncInstant::now();

        let collector = SystemMetricsCollectorAsync::new();
        let process = Arc::new(AsyncMutex::new(process));

        // Execute the process
        let thread_data =
            SystemMetricsCollectorAsync::collect_thread_sys_metrics_async(&collector, process)
                .await?;

        let avg_cpu_usage = system.refresh_final_metrics(initial_cpu_usage)?;

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

        Ok(TimeMeasurementData::new(
            start_time_millis,
            end_time_millis,
            elapsed_time,
            avg_cpu_usage,
            Some(thread_data),
        ))
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::{TimeComplexity, TimeMeasurement, *};

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
            TimeMeasurement::measure_single_thread_sync(create_single_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    #[test]
    fn test_sync_multi_thread_code_print() -> Result<()> {
        let data = TimeMeasurement::measure_multi_thread_sync(create_multi_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_async_single_thread_code_print() -> Result<()> {
        let data =
            TimeMeasurement::measure_single_thread_sync(create_single_thread_test_conditions);

        assert!(data.is_ok());

        Ok(())
    }

    /// Change number of workers based on architecture of running machine
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn test_async_multi_thread_code_print() -> Result<()> {
        let data =
            TimeMeasurement::measure_multi_thread_async(create_multi_thread_test_conditions).await;

        assert!(data.is_ok());

        Ok(())
    }
}
