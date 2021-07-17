/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Timer primitives.

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
use core::time::Duration;
pub use crate::arch::time::simple_timer as arch_timer;

/// Number of nanoseconds in a second
pub const NS_PER_SEC: u64 = 1_000_000_000;

/// Number of seconds in a minute
pub const SECS_PER_MINUTE: u64 = 60;

/// Number of seconds in an hour
pub const SECS_PER_HOUR: u64 = 60 * SECS_PER_MINUTE;

/// Number of seconds in a day, ignoring leap seconds.
pub const SECS_PER_DAY: u64 = 24 * SECS_PER_HOUR;

/// Number of seconds in a week, ignoring leap seconds.
pub const SECS_PER_WEEK: u64 = 7 * SECS_PER_DAY;

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

/// Extension methods for [`core::time::Duration`]
pub trait DurationExt {
    fn display_human(&self) -> DisplayDuration<'_>;
}

impl DurationExt for Duration {
    fn display_human(&self) -> DisplayDuration<'_> {
        DisplayDuration(self)
    }
}

pub struct DisplayDuration<'d>(&'d Duration);

impl ufmt::uDisplay for DisplayDuration<'_> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized
    {
        use ufmt::uwrite;

        let secs = self.0.as_secs();
        let nanos = self.0.as_nanos();
        let mut secs_remainder = secs;

        // we always compare against the original number of seconds, but do our divisions using the
        // remainder after subtracting the larger units. This means we sometimes get 0-numbered
        // units (e.g. 5 minutes, 0 seconds). This could be good or bad depending on who you ask.
        // It also means we don't need to keep track of whether or not to print the comma.
        if secs >= SECS_PER_WEEK {
            uwrite!(f, "{} weeks, ", secs / SECS_PER_WEEK)?;
            secs_remainder %= SECS_PER_WEEK;
        }
        if secs >= SECS_PER_DAY {
            uwrite!(f, "{} days, ", secs_remainder / SECS_PER_DAY)?;
            secs_remainder %= SECS_PER_DAY;
        }
        if secs >= SECS_PER_HOUR {
            uwrite!(f, "{} hours, ", secs_remainder / SECS_PER_HOUR)?;
            secs_remainder %= SECS_PER_HOUR;
        }
        if secs >= SECS_PER_MINUTE {
            uwrite!(f, "{} minutes, ", secs_remainder / SECS_PER_MINUTE)?;
            secs_remainder %= SECS_PER_MINUTE;
        }
        uwrite!(f, "{} seconds, {} ns", secs_remainder, nanos)
    }
}
