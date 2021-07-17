/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod gpio;
pub mod uart;

#[derive(Debug)]
pub enum Error {
}

pub mod traits {
    use crate::sync::*;

    /// Trait that's implemented by all drivers
    pub trait Driver {
        const COMPATIBLE: &'static str;
    }

    /// Object-safe trait that can be implemented for Mutex-protected drivers
    pub trait Compatible {
        fn compatible(&self) -> &'static str;
    }

    impl<T: Driver> Compatible for T {
        fn compatible(&self) -> &'static str {
            T::COMPATIBLE
        }
    }

    impl<R, T> Compatible for Mutex<R, T>
    where
        R: RawMutex,
        T: Driver,
    {
        fn compatible(&self) -> &'static str {
            T::COMPATIBLE
        }
    }
}
