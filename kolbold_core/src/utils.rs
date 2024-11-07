use super::error::{MetricError, TimeError};
use anyhow::Result;

/// Helper function to prevent integer overflow
pub fn check_sub(val_1: u64, val_2: u64) -> Result<u64> {
    match val_1.checked_sub(val_2) {
        Some(result) => {
            if val_1 < val_2 {
                return Ok(0);
            }
            Ok(result)
        }
        None => Err(MetricError::TimeError(TimeError::OverflowError).into()),
    }
}
