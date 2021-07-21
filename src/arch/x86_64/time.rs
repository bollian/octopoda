/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::time::Duration;
use crate::time::SimpleTimer;

pub struct GenericTimer;

fn read_instruction_count() -> u64 {
    // SAFETY: it's alright that this acts as an instruction barrier
    unsafe { ::x86::time::rdtscp() }
}

pub fn simple_timer() -> impl SimpleTimer {
    GenericTimer
}

impl SimpleTimer for GenericTimer {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(10)
    }

    fn uptime(&self) -> Duration {
        Duration::from_nanos(read_instruction_count())
    }

    fn spin_for(&self, duration: Duration) {
        for _ in 0..duration.as_nanos() {
            crate::arch::asm::nop()
        }
    }
}
