// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        mod aarch64;
        pub use aarch64::*;
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        mod x86;
        pub use self::x86::*;
    }
}
