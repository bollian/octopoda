/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::driver::{text_vga::TextVga, traits::Compatible, WriteError};
use crate::sync::{SpinMutex, SpinMutexMut};

pub mod mmap {
    pub const TEXT_VGA: usize = 0xb8000;
}

pub struct DriverManager {
    text_vga: SpinMutex<TextVga>,
}

impl DriverManager {
    /// Initialize all available drivers
    ///
    /// # Safety
    ///
    /// Must be called only once to avoid double-initializing peripherals.
    pub unsafe fn new() -> Self {
        Self {
            text_vga: SpinMutex::new(TextVga::new(mmap::TEXT_VGA)),
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &dyn Compatible> {
        core::array::IntoIter::new([&self.text_vga as &dyn Compatible])
    }

    pub fn stdout(&self) -> SpinMutexMut<dyn ufmt::uWrite<Error = WriteError>> {
        self.text_vga.borrow()
    }
}
