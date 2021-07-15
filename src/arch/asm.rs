//! The purpose of this module is to abstract over the different supported cpu architectures. The
//! desired effect is that no other modules need contain assembly code.

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(target_arch = "aarch64")]
use aarch64::*;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use x86::*;

/// Do nothing for a cycle or two. Shouldn't modify any registers aside from the instruction
/// pointer.
#[inline(always)]
pub fn nop() {
    _nop()
}

/// Wait forever in a power-efficient manner
#[inline(always)]
pub fn wait_forever() -> ! {
    _wait_forever()
}

/// Run a given number of `nop` instructions. Since many CPUs execute a number of instructions
/// != the number of clock cycles, this does not guarantee that the CPU will wait for any specific
/// number of _cycles_, just `nop`s. Additionally, this is a busy loop, so only use it for small
/// delays, if possible.
#[inline(always)]
pub fn spin_for_nops(count: u32) {
    for _ in 0..count {
        nop()
    }
}
