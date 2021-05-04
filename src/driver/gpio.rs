use crate::arch;
use register::mmio::ReadWrite;
use register::{register_bitfields, register_structs};

// Description taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1. Used for switching pins between read/write/IO device mode.
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input         = 0b000,
            Output        = 0b001,
            MINI_UART_RX  = 0b010, // mini-uart alternate function 5
            PL011_UART_RX = 0b100 // PL011 uart controller RX
        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input         = 0b000,
            Output        = 0b001,
            MINI_UART_TX  = 0b010, // mini-uart alternate function 5
            PL011_UART_TX = 0b100 // PL011 uart controller TX
        ]
    ],

    /// GPIO Pull-up/down register
    ///
    /// BCM2837 only
    GPPUD [
        /// Controls the activation of the internal pull-up/down line to ALL gpio pins
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect    = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect    = 0,
            AssertClock = 1
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => GPFSEL0: ReadWrite<u32>),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => GPFSEL2: ReadWrite<u32>),
        (0x0c => GPFSEL3: ReadWrite<u32>),
        (0x10 => GPFSEL4: ReadWrite<u32>),
        (0x14 => GPFSEL5: ReadWrite<u32>),
        (0x18 => _reserved),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9c => GPPUDCLK1: ReadWrite<u32>),
        (0xa0 => @END),
    }
}

// FIXME: This provides shared mutability over the register block. Add some sort of Mutex to
// regulate that interior mutability.
pub struct Gpio {
    regs: &'static RegisterBlock,
}

impl Gpio {
    /// # Safety
    /// The user must verify that the address for the register block is correct.
    pub unsafe fn new(base_address: usize) -> Self {
        Self {
            regs: &*(base_address as *const _),
        }
    }

    #[cfg(feature = "bsp_rpi3")]
    pub fn setup_uart(&mut self) {
        const DELAY: u32 = 2000;

        // map UART1 to GPIO pins
        self.regs.GPFSEL1.modify(GPFSEL1::FSEL14::PL011_UART_TX + GPFSEL1::FSEL15::PL011_UART_RX);
        arch::asm::spin_for_nops(DELAY);

        self.regs.GPPUD.write(GPPUD::PUD::Off);
        arch::asm::spin_for_nops(DELAY);

        self.regs.GPPUDCLK0.write(
            GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock,
        );
        arch::asm::spin_for_nops(DELAY);

        self.regs.GPPUD.write(GPPUD::PUD::Off);
        self.regs.GPPUDCLK0.set(0);
    }
}
