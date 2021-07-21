/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module is responsible for making sure we're running in a sane
//! environment. It doesn't contain any "functional" code, and currently just
//! zeroes out the bss section and stops all but the first core.
//!
//! On arm, we boot into the [`_start`] function. On x86, we rely on the `bootloader` crate.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        use core::cell::UnsafeCell;
        use core::ops::RangeInclusive;
        use crate::arch::asm;
        use crate::memory;

        #[naked]
        #[no_mangle]
        pub unsafe extern fn _start() -> ! {
            asm!(
                // set the stack pointer
                "adrp x0, __boot_core_stack_end_exclusive",
                "add x0, x0, #:lo12:__boot_core_stack_end_exclusive",
                "mov sp, x0",

                // call into rust code
                "b _start_rust",
                options(noreturn)
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn _start_rust() -> ! {
            // only continue with running the kernel if we're core 0,
            // otherwise wait_forever (i.e. stop the core)
            if crate::arch::cpu::core_id() == 0 {
                asm!(
                    "ldr x1, =_start",
                    "mov sp, x1",
                );
                runtime_init()
            } else {
                asm::wait_forever()
            }
        }

        /// Zero the bss section before calling into main.
        /// In the future, this function should include any setup code that isn't
        /// architecture specific and required for a normal rust runtime.
        #[no_mangle]
        pub unsafe fn runtime_init() -> ! {
            memory::set_volatile(bss_range(), 0);
            crate::main()
        }

        extern "Rust" {
            // these are named to match the linker script, and screaming snake case is usually
            // reserved for linker commands in those scripts
            #[allow(non_upper_case_globals)]
            static __bss_start: UnsafeCell<usize>;

            #[allow(non_upper_case_globals)]
            static __bss_end_inclusive: UnsafeCell<usize>;
        }

        /// Returns the start and end addresses for the bss section in memory.
        ///
        /// The returned addresses must be valid and usize-aligned.
        #[inline(always)]
        pub fn bss_range() -> RangeInclusive<*mut usize> {
            let bss_range = unsafe {
                RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get())
            };
            assert!(!bss_range.is_empty());
            bss_range
        }
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        #[no_mangle]
        pub unsafe extern "C" fn _start() -> ! {
            crate::main()
        }
    }
}
