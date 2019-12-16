// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![feature(global_asm)]

#[allow(unused_imports)]
use panic_abort;

/// Type-check the user-supplied entry function and provide the necessary
/// linker name.
#[macro_export]
macro_rules! entry {
    ($main_fn:expr) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            const _: fn () -> ! = $main_fn;
            $main_fn()
        }
    }
}

/// Initialize the bss section before calling into main()
/// This handler is called by the ARM architecture
#[no_mangle]
pub unsafe fn reset() -> ! {
    // boundaries of the .bss section, provided by the linker script
    extern "C" {
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    extern "Rust" {
        fn main() -> !;
    }
    main()
}

global_asm!(include_str!("boot_cores_armv8.S"));
