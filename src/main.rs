//! An experimental kernel I'm developing as a thought experiment.

#![allow(clippy::enum_variant_names)] // enum variants are often based on hardware names
#![deny(const_err, illegal_floating_point_literal_pattern, trivial_numeric_casts)]
#![forbid(bare_trait_objects, improper_ctypes, no_mangle_generic_items, patterns_in_fns_without_body)]

#![no_std]
#![no_main]

#![feature(asm, naked_functions, maybe_uninit_extra)]

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

use time::{DurationExt, SimpleTimer};
use core::convert::Infallible;
use core::time::Duration;
use ufmt::uwriteln;
use bsp::DriverManager;

pub static DRIVERS: sync::Lazy<DriverManager> = sync::Lazy::new(|| unsafe { DriverManager::new() });

/// The "main" entrypoint of the kernel. Called after stopping other cores
/// and initializing the bss section.
fn main() -> ! {
    stdout().with_lock(|w| {
        let _ = uwriteln!(w, "Hello, World!");
    });

    loop {
        time::arch_timer().spin_for(Duration::from_secs(5));
        trace!("Current uptime: {}", time::arch_timer().uptime().display_human());
    }
}

pub fn stdout() -> sync::SpinMutexMut<'static, dyn ufmt::uWrite<Error=Infallible>> {
    DRIVERS.get().stdout()
}
