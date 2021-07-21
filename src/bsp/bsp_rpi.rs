/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::driver::{gpio::Gpio, uart::PL011Uart, traits::Compatible, WriteError};
use crate::sync::{SpinMutex, SpinMutexMut};

pub mod mmap {
    #[cfg(feature = "bsp_rpi3")]
    pub const MMIO_BASE: usize = 0x3f00_0000;
    #[cfg(feature = "bsp_rpi4")]
    pub const MMIO_BASE: usize = 0xfe00_0000;

    pub const GPIO_BASE: usize = MMIO_BASE + 0x20_0000;
    pub const PL011_UART_BASE: usize = MMIO_BASE + 0x20_1000;
    // pub const SPI1_BASE: usize = MMIO_BASE + 0x21_5080;
    // pub const SPI2_BASE: usize = MMIO_BASE + 0x21_50c0;
}

pub struct DriverManager {
    gpio: SpinMutex<Gpio>,
    uart: SpinMutex<PL011Uart>,
}

impl DriverManager {
    /// Initialize all available drivers
    ///
    /// # Safety
    ///
    /// Must be called only once to avoid double-initializing peripherals.
    pub unsafe fn new() -> Self {
        let mut gpio = Gpio::new(mmap::GPIO_BASE);
        let uart = PL011Uart::new(mmap::PL011_UART_BASE);
        uart.init(&mut gpio, 921_600).unwrap();

        let gpio = SpinMutex::new(gpio);
        let uart = SpinMutex::new(uart);

        Self {
            gpio,
            uart,
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &dyn Compatible> {
        core::array::IntoIter::new([&self.gpio as &dyn Compatible, &self.uart])
    }

    pub fn stdout(&self) -> SpinMutexMut<dyn ufmt::uWrite<Error = WriteError>> {
        self.uart.borrow()
    }
}
