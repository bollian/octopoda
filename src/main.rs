//! An experimental kernel I'm developing as a thought experiment.

#![deny(const_err, illegal_floating_point_literal_pattern, trivial_numeric_casts)]
#![forbid(bare_trait_objects, improper_ctypes, no_mangle_generic_items, patterns_in_fns_without_body)]

#![no_std]
#![no_main]

#![feature(asm, naked_functions, maybe_uninit_extra, maybe_uninit_ref)]

mod arch;
mod bsp;
mod defer;
mod driver;
mod memory;
mod panic_wait;
mod runtime_init;
mod sync;
// mod time;
// mod warn;

use core::convert::Infallible;
use ufmt::uwriteln;
use bsp::DriverManager;

/// The "main" entrypoint of the kernel. Called after stopping other cores
/// and initializing the bss section.
fn main() -> ! {
    stdout().with_lock(|w| {
        let _ = uwriteln!(w, "Hello, World!");
    });
    arch::asm::wait_forever()
}

pub fn drivers() -> &'static DriverManager {
    static DRIVER_MANAGER: sync::OnceCell<DriverManager> = sync::OnceCell::new();
    // SAFETY: guaranteed to only run once thanks to OnceCell
    DRIVER_MANAGER.get_or_init(|| {
        unsafe { DriverManager::new() }
    })
}

pub fn stdout() -> sync::SpinMutexMut<'static, dyn ufmt::uWrite<Error=Infallible>> {
    drivers().stdout()
}
