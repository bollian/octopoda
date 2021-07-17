// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

use cortex_a::asm::*;

pub use cortex_a::asm::nop as _nop;

#[inline(always)]
pub fn _wait_forever() -> ! {
    loop {
        wfe()
    }
}
