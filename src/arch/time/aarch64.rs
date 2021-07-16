// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural timer primitives.
//!
//! There are a couple registers used for the Aarch64 architectural timer:
//! - CNTPCT_EL0: the system "counter-timer". Basically it's a number that counts up with time.
//! - CNTFRQ_EL0: the frequency of the above timer, in Hz
//!
//! Together these can be used to measure relative time in real units.
//!
//! Additionally, there exists:
//! - CNTP_TVAL_EL0: set a hardware countdown using CNTPCT_EL0,
//! - CNTP_CTL_EL0: control the behavior of the countdown timer (interrupts, enable/disable, etc)
//!
//! Which are used for creating countdown timers.

use crate::{time, warn};
use core::convert::TryInto;
use core::time::Duration;
use cortex_a::{asm::barrier, registers::*};
use tock_registers::interfaces::{Readable, ReadWriteable, Writeable};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// TODO: this constant probably shouldn't live here
const NS_PER_S: u64 = 1_000_000_000;

/// ARMv8 Generic Timer.
struct GenericTimer;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static SIMPLE_TIMER: GenericTimer = GenericTimer;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl GenericTimer {
    #[inline(always)]
    fn read_cntpct(&self) -> u64 {
        // Prevent that the counter is read ahead of time due to out-of-order execution.
        unsafe { barrier::isb(barrier::SY) };
        CNTPCT_EL0.get()
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the time manager.
pub fn simple_timer() -> &'static impl time::SimpleTimer {
    &SIMPLE_TIMER
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl time::SimpleTimer for GenericTimer {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(NS_PER_S / CNTFRQ_EL0.get())
    }

    fn uptime(&self) -> Duration {
        let current_count: u64 = self.read_cntpct() * NS_PER_S;
        let frq = CNTFRQ_EL0.get();

        Duration::from_nanos(current_count / frq)
    }

    fn spin_for(&self, duration: Duration) {
        // Instantly return on zero.
        if duration.as_nanos() == 0 {
            return
        }

        // Calculate the register compare value.
        let frq = CNTFRQ_EL0.get();
        let nanos = duration.as_nanos().try_into().ok();
        let x = match nanos.and_then(|nanos| frq.checked_mul(nanos)) {
            Some(val) => val,
            None => {
                warn!("Spin duration of {}ns too long, skipping", duration.as_nanos());
                return
            }
        };
        let tval = x / NS_PER_S;

        // Check if it is within supported bounds.
        if tval == 0 {
            warn!("Spin duration smaller than architecturally supported, skipping");
            return
        } else if tval > u32::max_value().into() {
            warn!("Spin duration bigger than architecturally supported, skipping");
            return
        }

        // Set the compare value register.
        CNTP_TVAL_EL0.set(tval);

        // Kick off the counting.                       // Disable timer interrupt.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

        // ISTATUS will be '1' when cval ticks have passed. Busy-check it.
        while !CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {}

        // Disable counting again.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
    }
}
