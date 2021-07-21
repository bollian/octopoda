/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Memory mappings and other things for specific boards

cfg_if::cfg_if! {
    if #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))] {
        pub mod bsp_rpi;
        pub use bsp_rpi::*;
    } else if #[cfg(target_arch = "x86_64")] {
        pub mod bsp_x86_64;
        pub use bsp_x86_64::*;
    }
}
