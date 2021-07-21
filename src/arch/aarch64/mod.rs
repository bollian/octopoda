/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod time;

pub mod asm {
    use cortex_a::asm::*;

    pub use cortex_a::asm::nop;

    #[inline(always)]
    pub fn wait_forever() -> ! {
        loop {
            wfe()
        }
    }
}

pub mod cpu {
    #[inline(always)]
    pub fn core_id() -> u64 {
        use cortex_a::registers::*;
        use tock_registers::interfaces::Readable;
        MPIDR_EL1.get() & 0b11
    }

    #[inline(always)]
    pub fn exception_level() -> u64 {
        todo!()
    }
}
