//! # Time Measurement Module
//!
//! This module provides tools and implementations for collecting and measuring time-related metrics
//! in both synchronous and asynchronous contexts. It includes support for single-threaded and multi-threaded
//! execution, allowing comprehensive CPU usage tracking and time complexity analysis.
//!
//! ## Overview
//! - `TimeMeasurementData`: Represents collected data including start and end timestamps, elapsed time,
//!   CPU usage, and optional thread-specific metrics.
//! - `SingleSystemMetricsCollector`, `SystemMetricsCollectorSync`, and `SystemMetricsCollectorAsync`:
//!   Collectors used for gathering system-level metrics in different execution environments.
//! - Traits such as `TimeSysMetricsCollectorSync` and `TimeSysMetricsCollectorAsync` for defining
//!   synchronous and asynchronous operations related to time metrics collection.
//!
//! ## Key Features
//! - Measure time and CPU usage for single-threaded and multi-threaded code execution.
//! - Supports both synchronous and asynchronous metric collection.
//! - Collects detailed thread-specific metrics when running in multi-threaded mode.
//! - `SystemMetricsCollectorSync` and `SystemMetricsCollectorAsync` now implement the `Default` trait, allowing for easy instantiation using `::default()`.
//!
//! ## Instantiation
//! Collectors can be instantiated using the `new()` method or the `Default` trait:
//! ```rust
//! use kolbold::memory::memory_metrics_collector::SystemMetricsCollectorSync;
//! use kolbold::memory::memory_metrics_collector::SystemMetricsCollectorAsync;
//!
//! let sync_collector = SystemMetricsCollectorSync::new();
//! let async_collector = SystemMetricsCollectorAsync::default(); // Equivalent to `SystemMetricsCollectorAsync::new()`
//! ```
//!
//! ## Example Usage
//! ```rust
//! use kolbold::time::time_measurement::{TimeComplexity, TimeMeasurement};
//! use anyhow::Result;
//!
//! fn example_sync_measurement() -> Result<()> {
//!     let data = TimeMeasurement::measure_single_thread_sync(|| {
//!         // Simulated computation
//!         let mut v = vec![0; 1_000_000];
//!         v.iter_mut().for_each(|x| *x += 1);
//!     })?;
//!
//!     println!("Time measurement data: {:?}", data);
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn example_async_measurement() -> Result<()> {
//!     let data = TimeMeasurement::measure_single_thread_async(|| {
//!         // Simulated asynchronous computation
//!         let mut v = vec![0; 1_000_000];
//!         v.iter_mut().for_each(|x| *x += 1);
//!     }).await?;
//!
//!     println!("Async time measurement data: {:?}", data);
//!     Ok(())
//! }
//! ```
//!
//! ## Usage Notes
//! - The module uses `sysinfo` to collect CPU metrics, which may have specific configuration and dependency requirements.
//! - Thread-specific metrics provide granular insight when using multi-threaded execution for performance analysis.
//!
//! ## Test Suite
//! This module includes a test suite that verifies the functionality of both synchronous and asynchronous
//! measurements for single-threaded and multi-threaded code. The tests ensure that the metric collection
//! is accurate and functions as expected in different scenarios.

use super::super::{
    error::{MetricError, TimeError},
    system_metrics::{
        TimeCollectorLock, TimeCollectorLockAsync, TimeSysMetricsCollectorAsync,
        TimeSysMetricsCollectorSync,
    },
    thread_metrics::TimeThreadMetrics,
};

use anyhow::Result;
use futures::future;
use smol_str::{SmolStr, ToSmolStr};
use std::{
    fmt,
    time::Instant as SyncInstant,
    {
        sync::{Arc, Mutex, MutexGuard},
        thread,
    },
};
use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL,
};
use tokio::{sync::Mutex as AsyncMutex, time::Instant as AsyncInstant};
use uuid::Uuid;

#[derive(Debug)]
pub struct TimeMeasurementData {
    /// Timestamp in milliseconds when the measurement started.
    start_time: u64,
    /// Timestamp in milliseconds when the measurement ended.
    end_time: u64,
    /// Duration in milliseconds between start and end times, representing the total time taken.
    elapsed_time: u64,
    /// CPU usage percentage during the measurement, indicating the load on the CPU.
    cpu_usage: f32,
    /// Optional thread-specific metrics, present if the measurement is multi-threaded.
    thread_metrics: Option<Vec<TimeThreadMetrics>>,
}

impl fmt::Display for TimeMeasurementData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Start time: {:?} | End time: {:?} | Elapsed time: {:?} | CPU usage: {:.2}%",
            self.start_time, self.end_time, self.elapsed_time, self.cpu_usage,
        )?;

        // If thread metrics are available, print each thread's details.
        if let Some(ref threads) = self.thread_metrics {
            write!(f, "\nThread Metrics:\n")?;
            for thread in threads {
                writeln!(f, "{:?}", thread)?;
            }
        }

        Ok(())
    }
}

impl TimeMeasurementData {
    /// Constructs a new `TimeMeasurementData` instance.
    ///
    /// # Parameters
    /// - `start_time`: The timestamp in milliseconds when the measurement started.
    /// - `end_time`: The timestamp in milliseconds when the measurement ended.
    /// - `elapsed_time`: The duration of the measurement in milliseconds.
    /// - `cpu_usage`: The CPU usage percentage during the measurement.
    /// - `thread_metrics`: Optional metrics for individual threads.
    ///
    /// # Returns
    /// A new `TimeMeasurementData` instance.
    pub fn new(
        start_time: u64,
        end_time: u64,
        elapsed_time: u64,
        cpu_usage: f32,
        thread_metrics: Option<Vec<TimeThreadMetrics>>,
    ) -> Self {
        Self {
            start_time,
            end_time,
            elapsed_time,
            cpu_usage,
            thread_metrics,
        }
    }
}

#[derive(Debug)]
pub struct SingleSystemMetricsCollector {
    /// A System instance to refresh and retrieve metrics
    system: System,
}

impl Default for SingleSystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSysMetricsCollectorSync for SingleSystemMetricsCollector {
    /// Refreshes CPU information within the `System` instance.
    fn refresh_cpu(&mut self) {
        self.system.refresh_cpu_all();
    }

    /// Calculates and returns the average CPU usage across all cores.
    ///
    /// # Returns
    /// - `Ok(f32)`: The average CPU usage as a percentage.
    /// - `Err(MetricError)`: If no CPUs are found or another error occurs.
    fn cpu_usage(&self) -> Result<f32, MetricError> {
        // Calculate average CPU usage
        let total_cpu_usage: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();

        let cpu_count = self.system.cpus().len() as f32;
        if cpu_count == 0.0 {
            return Err(MetricError::TimeError(TimeError::MissingCpuError));
        }

        Ok(total_cpu_usage / cpu_count)
    }
}

impl SingleSystemMetricsCollector {
    /// Creates a new `SingleSystemMetricsCollector` with specific refresh settings for CPU and memory.
    ///
    /// # Returns
    /// A new `SingleSystemMetricsCollector` instance.
    pub fn new() -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
        }
    }

    /// Refreshes initial CPU metrics and waits to ensure accurate initial readings.
    ///
    /// # Returns
    /// - `Ok(f32)`: The initial CPU usage percentage.
    /// - `Err(MetricError)`: If an error occurs during metric collection.
    pub fn refresh_initial_metrics(&mut self) -> Result<f32, MetricError> {
        self.refresh_cpu();

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

        self.refresh_cpu();

        let initial_cpu_usage: f32 = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.system.cpus().len() as f32;

        Ok(initial_cpu_usage)
    }

    /// Refreshes final CPU metrics and calculates the average CPU usage between the initial and final readings.
    ///
    /// # Parameters
    /// - `initial_cpu_usage`: The initial CPU usage percentage.
    ///
    /// # Returns
    /// - `Ok(f32)`: The average CPU usage percentage.
    /// - `Err(MetricError)`: If an error occurs during metric collection.
    pub fn refresh_final_metrics(&mut self, initial_cpu_usage: f32) -> Result<f32, MetricError> {
        self.refresh_cpu();

        let final_cpu_usage: f32 = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.system.cpus().len() as f32;
        let avg_cpu_usage = (initial_cpu_usage + final_cpu_usage) / 2.0;

        Ok(avg_cpu_usage)
    }
}

#[derive(Debug, Clone)]
pub struct SystemMetricsCollectorSync {
    /// A System instance to refresh and retrieve metrics wrapped in sync Arc and Mutex for safe thread sharing
    system: Arc<Mutex<System>>,
}

impl Default for SystemMetricsCollectorSync {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSysMetricsCollectorSync for SystemMetricsCollectorSync {
    /// Refreshes CPU information within the `System` instance.
    fn refresh_cpu(&mut self) {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_cpu_all();
        }
    }

    /// Calculates and returns the average CPU usage across all cores.
    ///
    /// # Returns
    /// - `Ok(f32)`: The average CPU usage as a percentage.
    /// - `Err(MetricError)`: If the system lock fails or no CPUs are found.
    fn cpu_usage(&self) -> Result<f32, MetricError> {
        let system = self.system.lock().map_err(|_| {
            MetricError::LockError(
                anyhow::anyhow!("Failed to lock system mutex while getting CPU usage").to_smolstr(),
            )
        })?;

        let total_cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();

        let cpu_count = system.cpus().len() as f32;
        if cpu_count == 0.0 {
            return Err(MetricError::TimeError(TimeError::MissingCpuError));
        }

        Ok(total_cpu_usage / cpu_count)
    }
}

impl TimeCollectorLock for Arc<Mutex<SystemMetricsCollectorSync>> {
    type Collector = SystemMetricsCollectorSync;

    /// Locks the collector for safe access and returns a `MutexGuard`.
    ///
    /// # Returns
    /// - `Ok(MutexGuard<Self::Collector>)`: The locked collector.
    /// - `Err(MetricError)`: If the lock cannot be acquired.
    fn lock_collector(&self) -> Result<MutexGuard<Self::Collector>, MetricError> {
        self.lock().map_err(|_| {
            MetricError::LockError(anyhow::anyhow!("Failed to lock collector mutex").to_smolstr())
        })
    }
}

impl SystemMetricsCollectorSync {
    /// Creates a new `SystemMetricsCollectorSync` with specific refresh settings for CPU and memory.
    ///
    /// # Returns
    /// A new `SystemMetricsCollectorSync` instance.
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ))),
        }
    }

    /// Collects thread-specific CPU metrics by executing a process across multiple threads.
    ///
    /// # Parameters
    /// - `process`: A shared process to execute across threads.
    ///
    /// # Returns
    /// - `Ok(Vec<TimeThreadMetrics>)`: A vector of metrics for each thread.
    /// - `Err(MetricError)`: If an error occurs during collection or thread execution.
    pub fn collect_thread_sys_metrics<F, R>(
        &self,
        process: Arc<Mutex<F>>,
    ) -> Result<Vec<TimeThreadMetrics>, MetricError>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let num_of_cores = num_cpus::get();
        let collector = Arc::new(Mutex::new(self.clone()));
        let mut handles = Vec::with_capacity(num_of_cores);

        for _ in 0..num_of_cores {
            let process = Arc::clone(&process);
            let collector = Arc::clone(&collector);

            let handle = thread::spawn(move || -> Result<TimeThreadMetrics, MetricError> {
                Self::gather_metrics_generic_sync(&collector, process)
            });
            handles.push(handle);
        }

        let thread_metrics: Vec<TimeThreadMetrics> = handles
            .into_iter()
            .map(|handle| {
                handle.join().map_err(|e| {
                    MetricError::SyncJoinError(
                        anyhow::anyhow!("Thread join failed with error: {e:?}").to_smolstr(),
                    )
                })?
            })
            .collect::<Result<_, _>>()?;

        Ok(thread_metrics)
    }

    /// Collects metrics for a single thread synchronously.
    ///
    /// # Parameters
    /// - `collector`: The collector to gather metrics from.
    /// - `process`: The process to execute and measure.
    ///
    /// # Returns
    /// - `Ok(TimeThreadMetrics)`: The metrics collected for the thread.
    /// - `Err(MetricError)`: If an error occurs during the process or metric collection.
    fn gather_metrics_generic_sync<L, F, R>(
        collector: &L,
        process: Arc<Mutex<F>>,
    ) -> Result<TimeThreadMetrics, MetricError>
    where
        L: TimeCollectorLock,
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let mut collector = collector.lock_collector()?;
        collector.refresh_cpu();

        let thread_id = SmolStr::new(format!("{:?}", thread::current().id()));
        let start_time = SyncInstant::now();
        let initial_cpu = collector.cpu_usage()?;

        let _ = process.lock().map_err(|_| {
            MetricError::LockError(
                anyhow::anyhow!("Failed to lock process mutex for execution").to_smolstr(),
            )
        })?();

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        collector.refresh_cpu();
        let final_cpu = collector.cpu_usage()?;

        Ok(TimeThreadMetrics::new(
            thread_id,
            (initial_cpu + final_cpu) / 2.0,
            elapsed_time,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SystemMetricsCollectorAsync {
    /// The `System` instance wrapped in `Arc<AsyncMutex>` for safe concurrent access.
    system: Arc<AsyncMutex<System>>,
}

impl Default for SystemMetricsCollectorAsync {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TimeSysMetricsCollectorAsync for SystemMetricsCollectorAsync {
    /// Refreshes CPU information asynchronously within the `System` instance.
    async fn refresh_cpu(&mut self) {
        let mut system = self.system.lock().await;
        system.refresh_cpu_all();
    }

    /// Calculates and returns the average CPU usage across all cores asynchronously.
    ///
    /// # Returns
    /// A `f32` representing the average CPU usage as a percentage.
    async fn cpu_usage(&self) -> f32 {
        let system = self.system.lock().await;
        system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32
    }
}

#[async_trait::async_trait]
impl TimeCollectorLockAsync for Arc<AsyncMutex<SystemMetricsCollectorAsync>> {
    type Collector = SystemMetricsCollectorAsync;

    /// Asynchronously locks the time metrics collector, ensuring safe concurrent access.
    ///
    /// # Returns
    /// - `Ok(TokioMutexGuard<Self::Collector>)`: The locked collector.
    /// - `Err(MetricError)`: If the lock cannot be acquired.
    async fn lock_collector(
        &self,
    ) -> Result<tokio::sync::MutexGuard<Self::Collector>, MetricError> {
        Ok(self.lock().await)
    }
}

impl SystemMetricsCollectorAsync {
    /// Creates a new `SystemMetricsCollectorAsync` with specific refresh settings for CPU and memory.
    ///
    /// # Returns
    /// A new `SystemMetricsCollectorAsync` instance.
    pub fn new() -> Self {
        Self {
            system: Arc::new(AsyncMutex::new(System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ))),
        }
    }

    /// Collects thread-specific CPU metrics asynchronously by executing a process across multiple threads.
    ///
    /// # Parameters
    /// - `process`: A shared process to execute across threads.
    ///
    /// # Returns
    /// - `Ok(Vec<TimeThreadMetrics>)`: A vector of metrics for each thread.
    /// - `Err(MetricError)`: If an error occurs during collection or thread execution.
    pub async fn collect_thread_sys_metrics_async<F, R>(
        &self,
        process: Arc<AsyncMutex<F>>,
    ) -> Result<Vec<TimeThreadMetrics>, MetricError>
    where
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let num_of_cores = num_cpus::get();

        let collector = Arc::new(AsyncMutex::new(self.clone()));
        let mut handles = Vec::with_capacity(num_of_cores);

        for _ in 0..num_of_cores {
            let process = Arc::clone(&process);
            let collector = Arc::clone(&collector);

            let handle = tokio::spawn(async move {
                Self::gather_metrics_generic_async(&collector, process).await
            });

            handles.push(handle);
        }

        let thread_metrics: Vec<TimeThreadMetrics> = future::try_join_all(handles)
            .await?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(thread_metrics)
    }

    /// Collects metrics for a single thread asynchronously.
    ///
    /// # Parameters
    /// - `collector`: The collector to gather metrics from.
    /// - `process`: The process to execute and measure.
    ///
    /// # Returns
    /// - `Ok(TimeThreadMetrics)`: The metrics collected for the thread.
    /// - `Err(MetricError)`: If an error occurs during the process or metric collection.
    async fn gather_metrics_generic_async<L, F, R>(
        collector: &L,
        process: Arc<AsyncMutex<F>>,
    ) -> Result<TimeThreadMetrics, MetricError>
    where
        L: TimeCollectorLockAsync,
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let mut collector = collector.lock_collector().await?;
        collector.refresh_cpu().await;

        let thread_id = Uuid::new_v4().to_smolstr();
        let start_time = AsyncInstant::now();
        let initial_cpu = collector.cpu_usage().await;

        let _ = process.lock().await;

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        collector.refresh_cpu().await;
        let final_cpu = collector.cpu_usage().await;

        Ok(TimeThreadMetrics::new(
            thread_id,
            (initial_cpu + final_cpu) / 2.0,
            elapsed_time,
        ))
    }
}
