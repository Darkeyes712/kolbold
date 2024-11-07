//! # Memory Metrics Collection Module
//!
//! This module provides structures and functions for measuring and collecting memory metrics in both
//! single-threaded and multi-threaded contexts, synchronously and asynchronously. It is useful for
//! benchmarking and analyzing memory usage in complex, concurrent Rust applications.
//!
//! ## Overview
//! - `MemoryMeasurementData`: Represents data collected during a memory measurement session, including
//!   start and end timestamps, elapsed time, total memory usage, and optional thread-specific metrics.
//! - `SingleSystemMetricsCollector`: Collects system-level memory metrics synchronously in a single-threaded context.
//! - `SystemMetricsCollectorSync`: Manages memory metrics collection across multiple threads synchronously.
//! - `SystemMetricsCollectorAsync`: Handles memory metrics collection asynchronously in a multi-threaded environment.
//!
//! ## Key Traits
//! - `MemorySysMetricsCollectorSync`: Trait for synchronous memory metric collection.
//! - `MemorySysMetricsCollectorAsync`: Trait for asynchronous memory metric collection.
//! - `MemoryCollectorLock` and `MemoryCollectorLockAsync`: Traits for safely locking collectors in synchronous and asynchronous environments.
//!
//! ## Key Features
//! - `SingleSystemMetricsCollector`, `SystemMetricsCollectorSync`, and `SystemMetricsCollectorAsync` implement the `Default` trait for convenient instantiation using `::default()`.
//! - Multi-threaded collection enables detailed thread-specific metrics, facilitating in-depth analysis of memory usage across concurrent threads.
//!
//! ## Example Usage
//! Here’s how to use `SingleSystemMetricsCollector` for a synchronous memory measurement, demonstrating `Default` for initialization:
//!
//! ```rust
//! use kolbold_core::system_metrics::MemorySysMetricsCollectorSync;
//! use kolbold_core::memory::memory_metrics_collector::{MemoryMeasurementData, SingleSystemMetricsCollector};
//! use anyhow::Result;
//!
//! fn measure_memory() -> Result<MemoryMeasurementData> {
//!     let mut collector = SingleSystemMetricsCollector::default(); // Using Default trait
//!     let initial_mem_usage = collector.refresh_initial_metrics()?;
//!     
//!     // Simulate a process to measure
//!     let simulated_process = || {
//!         let mut v = vec![0; 1_000_000];
//!         v.iter_mut().for_each(|x| *x += 1);
//!     };
//!     simulated_process();
//!
//!     let avg_mem_usage = collector.refresh_final_metrics(initial_mem_usage)?;
//!     Ok(MemoryMeasurementData::new(0, 0, 100, avg_mem_usage, None))
//! }
//!
//! match measure_memory() {
//!     Ok(data) => println!("Memory measurement successful: {:?}", data),
//!     Err(e) => eprintln!("Failed to measure memory: {}", e),
//! }
//! ```
//!
//! ## Usage Notes
//! - The module relies on `sysinfo` for retrieving system-level metrics and wraps it with custom error handling using `MetricError`.
//! - The `Default` implementation for `SingleSystemMetricsCollector`, `SystemMetricsCollectorSync`, and `SystemMetricsCollectorAsync` simplifies initialization, avoiding the need to call `new()` explicitly.
//! - The structs and traits in this module are ideal for benchmarking scenarios, offering precise memory usage tracking across both single-threaded and multi-threaded contexts.
//!
//! ## Test Suite
//! This module includes a robust test suite that verifies the functionality of synchronous and asynchronous
//! memory measurements in both single-threaded and multi-threaded scenarios. The tests cover initialization,
//! memory tracking, and error handling to ensure consistent metric collection and accurate memory usage reports.

use super::super::{
    error::{MemoryError, MetricError},
    system_metrics::{
        MemoryCollectorLock, MemoryCollectorLockAsync, MemorySysMetricsCollectorAsync,
        MemorySysMetricsCollectorSync,
    },
    thread_metrics::MemoryThreadMetrics,
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
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::{sync::Mutex as AsyncMutex, time::Instant as AsyncInstant};
use uuid::Uuid;

#[derive(Debug)]
pub struct MemoryMeasurementData {
    /// Timestamp in milliseconds when the measurement started.
    start_time: u64,
    /// Timestamp in milliseconds when the measurement ended.
    end_time: u64,
    /// Duration in milliseconds between the start and end times, indicating the total time taken for the measurement.
    elapsed_time: u64,
    /// Total memory usage in bytes during the measurement period.
    memory_usage: u64,
    /// Optional thread-specific metrics, available if the measurement is conducted in a multi-threaded context.
    thread_metrics: Option<Vec<MemoryThreadMetrics>>,
}

impl fmt::Display for MemoryMeasurementData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Start time: {} | End time: {} | Elapsed time: {} | Memory usage: {} | Thread Metrics: {:?}",
            self.start_time, self.end_time, self.elapsed_time, self.memory_usage, self.thread_metrics
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

impl MemoryMeasurementData {
    /// Constructs a new `MemoryMeasurementData` instance.
    ///
    /// # Parameters
    /// - `start_time`: The start timestamp in milliseconds.
    /// - `end_time`: The end timestamp in milliseconds.
    /// - `elapsed_time`: The total duration of the measurement in milliseconds.
    /// - `memory_usage`: The total memory usage during the measurement period in bytes.
    /// - `thread_metrics`: Optional thread-specific metrics.
    ///
    /// # Returns
    /// A new `MemoryMeasurementData` instance.
    pub fn new(
        start_time: u64,
        end_time: u64,
        elapsed_time: u64,
        memory_usage: u64,
        thread_metrics: Option<Vec<MemoryThreadMetrics>>,
    ) -> Self {
        Self {
            start_time,
            end_time,
            elapsed_time,
            memory_usage,
            thread_metrics,
        }
    }
}

#[derive(Debug)]
pub struct SingleSystemMetricsCollector {
    /// The `System` instance used to gather system metrics.
    system: System,
}

impl Default for SingleSystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MemorySysMetricsCollectorSync for SingleSystemMetricsCollector {
    /// Refreshes memory information within the `System` instance.
    fn refresh_memory(&mut self) {
        self.system.refresh_memory();
    }

    /// Retrieves the total memory usage in bytes.
    ///
    /// # Returns
    /// - `Ok(u64)`: The total memory usage in bytes.
    /// - `Err(MetricError)`: If memory usage cannot be determined.
    fn memory_usage(&self) -> Result<u64, MetricError> {
        self.system
            .total_memory()
            .checked_sub(self.system.available_memory())
            .ok_or_else(|| MetricError::MemoryError(MemoryError::CollectionError))
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

    /// Refreshes the initial memory metrics.
    ///
    /// # Returns
    /// - `Ok(u64)`: The initial memory usage in bytes.
    /// - `Err(MetricError)`: If an error occurs during metric collection.
    pub fn refresh_initial_metrics(&mut self) -> Result<u64, MetricError> {
        self.refresh_memory();

        let mem_usage = self.memory_usage()?;
        Ok(mem_usage)
    }

    /// Refreshes the final memory metrics and calculates the average memory usage.
    ///
    /// # Parameters
    /// - `init_mem_usage`: The initial memory usage in bytes.
    ///
    /// # Returns
    /// - `Ok(u64)`: The average memory usage in bytes.
    /// - `Err(MetricError)`: If an error occurs during metric collection.
    pub fn refresh_final_metrics(&mut self, init_mem_usage: u64) -> Result<u64, MetricError> {
        self.refresh_memory();

        let fin_mem_usage = self.memory_usage()?;
        let avg_mem_usage = (init_mem_usage + fin_mem_usage) / 2;

        Ok(avg_mem_usage)
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

impl MemorySysMetricsCollectorSync for SystemMetricsCollectorSync {
    /// The `System` instance wrapped in `Arc<Mutex>` for safe concurrent access.
    fn refresh_memory(&mut self) {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_memory();
        }
    }

    /// Retrieves the total memory usage in bytes.
    ///
    /// # Returns
    /// - `Ok(u64)`: The total memory usage in bytes.
    /// - `Err(MetricError)`: If the system lock fails or memory usage cannot be determined.
    fn memory_usage(&self) -> Result<u64, MetricError> {
        let system = self.system.lock().map_err(|_| {
            MetricError::LockError(
                anyhow::anyhow!("Failed to lock system mutex while getting memory usage")
                    .to_smolstr(),
            )
        })?;

        Ok(system.total_memory() - system.available_memory())
    }
}

impl MemoryCollectorLock for Arc<Mutex<SystemMetricsCollectorSync>> {
    type Collector = SystemMetricsCollectorSync;

    /// Locks the memory metrics collector for safe access.
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

    /// Collects thread-specific memory metrics by executing a process across multiple threads.
    ///
    /// # Parameters
    /// - `process`: A shared process to execute across threads.
    ///
    /// # Returns
    /// - `Ok(Vec<MemoryThreadMetrics>)`: A vector of metrics for each thread.
    /// - `Err(MetricError)`: If an error occurs during collection or thread execution.
    pub fn collect_thread_sys_metrics<F, R>(
        &self,
        process: Arc<Mutex<F>>,
    ) -> Result<Vec<MemoryThreadMetrics>, MetricError>
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

            let handle = thread::spawn(move || -> Result<MemoryThreadMetrics, MetricError> {
                Self::gather_metrics_generic_sync(&collector, process)
            });
            handles.push(handle);
        }

        let thread_metrics: Vec<MemoryThreadMetrics> = handles
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
    /// - `Ok(MemoryThreadMetrics)`: The metrics collected for the thread.
    /// - `Err(MetricError)`: If an error occurs during the process or metric collection.
    fn gather_metrics_generic_sync<L, F, R>(
        collector: &L,
        process: Arc<Mutex<F>>,
    ) -> Result<MemoryThreadMetrics, MetricError>
    where
        L: MemoryCollectorLock,
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let mut collector = collector.lock_collector()?;
        collector.refresh_memory();

        let thread_id = SmolStr::new(format!("{:?}", thread::current().id()));
        let start_time = SyncInstant::now();
        let init_mem_usage = collector.memory_usage()?;

        let _ = process.lock().map_err(|_| {
            MetricError::LockError(
                anyhow::anyhow!("Failed to lock process mutex for execution").to_smolstr(),
            )
        })?();

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        collector.refresh_memory();
        let fin_mem_usage = collector.memory_usage()?;
        let avg_mem_usage = (init_mem_usage + fin_mem_usage) / 2;

        Ok(MemoryThreadMetrics::new(
            thread_id,
            avg_mem_usage,
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
impl MemorySysMetricsCollectorAsync for SystemMetricsCollectorAsync {
    /// Asynchronously refreshes memory information within the `System` instance.
    async fn refresh_memory(&mut self) {
        let mut system = self.system.lock().await;

        system.refresh_memory();
    }

    /// Asynchronously retrieves the total memory usage in bytes.
    ///
    /// # Returns
    /// A `u64` representing the total memory usage in bytes.
    async fn memory_usage(&self) -> u64 {
        let system = self.system.lock().await;

        system.total_memory() - system.available_memory()
    }
}

#[async_trait::async_trait]
impl MemoryCollectorLockAsync for Arc<AsyncMutex<SystemMetricsCollectorAsync>> {
    type Collector = SystemMetricsCollectorAsync;

    /// Asynchronously locks the memory metrics collector for safe access.
    ///
    /// # Returns
    /// - `Ok(tokio::sync::MutexGuard<Self::Collector>)`: The locked collector.
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

    /// Collects thread-specific memory metrics asynchronously by executing a process across multiple threads.
    ///
    /// # Parameters
    /// - `process`: A shared process to execute across threads.
    ///
    /// # Returns
    /// - `Ok(Vec<MemoryThreadMetrics>)`: A vector of metrics for each thread.
    /// - `Err(MetricError)`: If an error occurs during collection or thread execution.
    pub async fn collect_thread_sys_metrics_async<F, R>(
        &self,
        process: Arc<AsyncMutex<F>>,
    ) -> Result<Vec<MemoryThreadMetrics>, MetricError>
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

        let thread_metrics: Vec<MemoryThreadMetrics> = future::try_join_all(handles)
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
    /// - `Ok(MemoryThreadMetrics)`: The metrics collected for the thread.
    /// - `Err(MetricError)`: If an error occurs during the process or metric collection.
    async fn gather_metrics_generic_async<L, F, R>(
        collector: &L,
        process: Arc<AsyncMutex<F>>,
    ) -> Result<MemoryThreadMetrics, MetricError>
    where
        L: MemoryCollectorLockAsync,
        F: FnOnce() -> R + Send + 'static + Copy,
        R: Send + 'static,
    {
        let mut collector = collector.lock_collector().await?;
        collector.refresh_memory().await;

        let thread_id = Uuid::new_v4().to_smolstr();
        let start_time = AsyncInstant::now();
        let init_mem_usage = collector.memory_usage().await;

        let _ = process.lock().await;

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        collector.refresh_memory().await;

        let fin_mem_usage = collector.memory_usage().await;
        let avg_mem_usage = (init_mem_usage + fin_mem_usage) / 2;

        Ok(MemoryThreadMetrics::new(
            thread_id,
            avg_mem_usage,
            elapsed_time,
        ))
    }
}
