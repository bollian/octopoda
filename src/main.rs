/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An experimental kernel I'm developing as a thought experiment.

#![allow(clippy::enum_variant_names)] // enum variants are often based on hardware names
#![deny(const_err, illegal_floating_point_literal_pattern, trivial_numeric_casts)]
#![forbid(bare_trait_objects, improper_ctypes, no_mangle_generic_items, patterns_in_fns_without_body)]

#![no_std]
#![no_main]

#![feature(asm, naked_functions, maybe_uninit_extra)]

#[cfg(target_arch = "x86_64")]
extern crate bootloader;

mod arch;
mod bsp;
mod defer;
mod driver;
mod log;
mod memory;
mod panic_wait;
mod runtime_init;
mod sync;
mod time;

use core::time::Duration;
use time::{DurationExt, SimpleTimer};
use ufmt::uwriteln;
use bsp::DriverManager;
use driver::WriteError;

pub static DRIVERS: sync::Lazy<DriverManager> = sync::Lazy::new(|| unsafe { DriverManager::new() });

/// The "main" entrypoint of the kernel. Called after stopping other cores
/// and initializing the bss section.
fn main() -> ! {
    stdout().with_lock(|w| {
        let _ = uwriteln!(w, "Hello, World!");
        for driver in DRIVERS.get().iter() {
            let _ = uwriteln!(w, "Loaded driver '{}'", driver.compatible());
        }
    });

    // let mut count = 0;
    // loop {
    //     trace!("Hello {}", count);
    //     count += 1;
    // }
    loop {
        time::arch_timer().spin_for(Duration::from_secs(5));
        trace!("Current uptime: {}", time::arch_timer().uptime().display_human());
    }
}

pub fn stdout() -> sync::SpinMutexMut<'static, dyn ufmt::uWrite<Error=WriteError>> {
    DRIVERS.get().stdout()
}
