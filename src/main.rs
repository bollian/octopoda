//! An experimental kernel I'm developing as a thought experiment.

#![deny(const_err, illegal_floating_point_literal_pattern, trivial_numeric_casts)]
#![forbid(bare_trait_objects, improper_ctypes, no_mangle_generic_items, patterns_in_fns_without_body)]

#![no_std]
#![no_main]

#![feature(llvm_asm, asm)]
#![feature(naked_functions)]

mod arch;
mod bsp;
mod driver;
mod memory;
mod panic_wait;
mod runtime_init;

// use core::fmt::Write;
use driver::uart::Uart;

/// The "main" entrypoint of the kernel. Called after stopping other cores
/// and initializing the bss section.
fn main() -> ! {
    use core::fmt::Write;

    let mut gpio = unsafe { driver::gpio::Gpio::new(bsp::mmap::GPIO_BASE) };
    let mut uart = unsafe { driver::uart::PL011Uart::new(bsp::mmap::PL011_UART_BASE) };
    uart.init(&mut gpio, 921_600).unwrap();

    let _ = writeln!(&mut uart, "Hello, World!");
    let _ = writeln!(&mut uart, "Echoing input now.");

    loop {
        let c = uart.receive_native();
        if c == b'\n' {
            uart.send(b'\r')
        }
        uart.send(c);
    }
}
