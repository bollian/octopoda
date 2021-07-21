/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod time;

pub mod asm {
    #[inline(always)]
    pub fn nop() {
        unsafe {
            asm!("nop");
        }
    }

    #[inline(always)]
    pub fn wait_forever() -> ! {
        unsafe {
            loop {
                x86::halt()
            }
        }
    }
}

pub mod cpu {
    #[inline(always)]
    pub fn core_id() -> u64 {
        todo!()
    }
}
