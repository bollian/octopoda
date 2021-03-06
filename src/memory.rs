// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Various memory manipulation helpers.

/// Zero out a memory region.
///
/// The provided range must have valid, T-aligned memory addresses.
pub unsafe fn set_volatile<T: Copy>(range: core::ops::RangeInclusive<*mut T>, value: T) {
    let mut ptr = *range.start();

    while ptr <= *range.end() {
        core::ptr::write_volatile(ptr, value);
        ptr = ptr.offset(1);
    }
}
