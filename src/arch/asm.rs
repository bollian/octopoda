/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The purpose of this module is to abstract over the different supported cpu architectures. The
//! desired effect is that no other modules need contain assembly code.

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(target_arch = "aarch64")]
use aarch64::*;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use self::x86::*;

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
