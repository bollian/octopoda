//! Memory mappings and other things for specific boards

use core::cell::UnsafeCell;
use core::ops::RangeInclusive;
use core::convert::Infallible;
use crate::driver::{Driver, gpio::Gpio, uart::PL011Uart};
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

// boundaries of the .bss section, provided by the linker script
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = SpinMutexMut<dyn Driver>> {
        core::array::IntoIter::new([self.gpio.borrow(), self.uart.borrow()])
    }

    pub fn stdout(&self) -> SpinMutexMut<dyn ufmt::uWrite<Error = Infallible>> {
        self.uart.borrow()
    }
}
