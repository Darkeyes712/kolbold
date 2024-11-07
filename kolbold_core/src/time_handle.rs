//! # Timer Trait Module
//!
//! This module provides the `Timer` trait, which standardizes time measurement functionality for both
//! synchronous and asynchronous contexts. The trait includes methods to start and stop a timer, returning
//! comprehensive timing data, such as start and end timestamps and elapsed time. This is particularly useful
//! for tracking the performance and time complexity of code segments in various runtime environments.
//!
//! ## Overview
//! - `Timer` Trait: Defines methods for starting and stopping a timer, capturing timing data in both synchronous
//!   and asynchronous contexts.
//!
//! ## Example Usage
//! ```rust
//! use kolbold_core::{TimeMeasurement, MemoryMeasurement};
//! use kolbold_core::time_handle::Timer;
//! use anyhow::Result;
//!
//! fn sync_timer_example() -> Result<()> {
//!     let (start_time, start_instant) = <TimeMeasurement as Timer>::start_timer_sync()?;
//!     // Simulate work
//!     let (start_millis, end_millis, elapsed) = <MemoryMeasurement as Timer>::stop_timer_sync(start_time, start_instant)?;
//!     println!("Elapsed time: {} ms, from {} to {}", elapsed, start_millis, end_millis);
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn async_timer_example() -> Result<()> {
//!     let (start_time, start_instant) = <TimeMeasurement as Timer>::start_timer_async().await?;
//!     // Simulate async work
//!     let (start_millis, end_millis, elapsed) = <MemoryMeasurement as Timer>::stop_timer_async(start_time, start_instant).await?;
//!     println!("Async Elapsed time: {} ms, from {} to {}", elapsed, start_millis, end_millis);
//!     Ok(())
//! }
//! ```
//!
//! ## Usage Notes
//! The `Timer` trait methods provide a unified approach to measure execution time accurately in different
//! contexts. It uses `SystemTime` for timestamping and `Instant` (or `AsyncInstant`) for calculating
//! elapsed time, ensuring high-resolution time tracking.

use super::error::MetricError;
use async_trait::async_trait;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::time::Instant as AsyncInstant;

#[async_trait]
pub trait Timer {
    /// Starts a synchronous timer, returning the current `SystemTime` and `Instant`.
    ///
    /// # Returns
    /// - `Ok((SystemTime, Instant))`: The start time in system time and an instant for elapsed calculation.
    /// - `Err(MetricError)`: If an error occurs during time measurement.
    fn start_timer_sync() -> Result<(SystemTime, Instant), MetricError> {
        Ok((SystemTime::now(), Instant::now()))
    }

    /// Stops a synchronous timer, calculating start and end timestamps along with elapsed time.
    ///
    /// # Parameters
    /// - `start_time`: The system start time recorded when the timer started.
    /// - `start_instant`: The instant start time for calculating elapsed duration.
    ///
    /// # Returns
    /// - `Ok((u64, u64, u64))`: A tuple containing start time in milliseconds, end time in milliseconds, and elapsed time in milliseconds.
    /// - `Err(MetricError)`: If an error occurs during time calculation.
    fn stop_timer_sync(
        start_time: SystemTime,
        start_instant: Instant,
    ) -> Result<(u64, u64, u64), MetricError> {
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

        Ok((start_time_millis, end_time_millis, elapsed_time))
    }

    /// Starts an asynchronous timer, returning the current `SystemTime` and `AsyncInstant`.
    ///
    /// # Returns
    /// - `Ok((SystemTime, AsyncInstant))`: The system start time and an async instant for elapsed calculation.
    /// - `Err(MetricError)`: If an error occurs during time measurement.
    async fn start_timer_async() -> Result<(SystemTime, AsyncInstant), MetricError> {
        Ok((SystemTime::now(), AsyncInstant::now()))
    }

    /// Stops an asynchronous timer, calculating start and end timestamps along with elapsed time.
    ///
    /// # Parameters
    /// - `start_time`: The system start time recorded when the timer started.
    /// - `start_instant`: The async instant start time for calculating elapsed duration.
    ///
    /// # Returns
    /// - `Ok((u64, u64, u64))`: A tuple containing start time in milliseconds, end time in milliseconds, and elapsed time in milliseconds.
    /// - `Err(MetricError)`: If an error occurs during time calculation.
    async fn stop_timer_async(
        start_time: SystemTime,
        start_instant: AsyncInstant,
    ) -> Result<(u64, u64, u64), MetricError> {
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

        Ok((start_time_millis, end_time_millis, elapsed_time))
    }
}
