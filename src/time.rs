// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Timer primitives.

use crate::arch::time;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
use core::time::Duration;
pub use time::time_manager;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Time management functions.
pub trait SimpleTimer {
    /// The timer's resolution.
    fn resolution(&self) -> Duration;

    /// The uptime since power-on of the device.
    ///
    /// This includes time consumed by firmware and bootloaders.
    fn uptime(&self) -> Duration;

    /// Spin for a given duration.
    fn spin_for(&self, duration: Duration);
}
