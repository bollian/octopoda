// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An experimental kernel I'm developing as a thought experiment.

#![deny(const_err, illegal_floating_point_literal_pattern, trivial_numeric_casts)]
#![forbid(bare_trait_objects, improper_ctypes, no_mangle_generic_items, patterns_in_fns_without_body)]

#![no_std]
#![no_main]

#![feature(asm)]

mod ext;
mod gpio;
mod uart;

use ext::Puts;

const MMIO_BASE: u32 = 0x3F00_0000;

/// The "main" entrypoint of the kernel. Called after stopping other cores
/// and initializing the bss section.
pub fn main() -> ! {
    let miniuart = uart::MiniUart::new();
    let puts_task = miniuart.puts("Hello, World!");
    task_loop(puts_task);

    loop {
        // do nothing
    }
}

fn task_loop<F: core::future::Future + core::marker::Unpin>(mut fut: F) {
    use core::task::Context;

    let waker = futures::task::noop_waker();
    let context = &mut Context::from_waker(&waker);

    let mut fut = core::pin::Pin::new(&mut fut);
    while fut.as_mut().poll(context).is_pending() {
        unsafe { asm!("nop" :::: "volatile"); }
    }
}

raspi3_boot::entry!(main);
