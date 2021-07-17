/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module is responsible for making sure we're running in a sane
//! environment. It doesn't contain any "functional" code, and currently just
//! zeroes out the bss section and stops all but the first core.

use crate::arch::asm;
use crate::memory;

#[cfg(target_arch = "aarch64")]
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

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    // only continue with running the kernel if we're core 0,
    // otherwise wait_forever (i.e. stop the core)
    if crate::arch::cpu::smp::core_id() == 0 {
            asm!(
                "ldr x1, =_start",
                "mov sp, x1",
            );
        runtime_init()
    } else {
        asm::wait_forever()
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // TODO: make sure x86 initializes properly
    asm::wait_forever()
}

/// Zero the bss section before calling into main.
/// In the future, this function should include any setup code that isn't
/// architecture specific and required for a normal rust runtime.
#[no_mangle]
pub unsafe fn runtime_init() -> ! {
    memory::set_volatile(crate::bsp::bss_range(), 0);
    crate::main()
}
