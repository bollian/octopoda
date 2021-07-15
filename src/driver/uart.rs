use core::convert::Infallible;
use crate::arch;
use crate::driver::{self, gpio::Gpio, Driver};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

pub trait Uart {
    /// Checks to see if the UART is ready to send more data.
    ///
    /// This function is called repeatedly and often as the kernel tries to
    /// schedule the best time for transmission, so try to
    /// make it performant.
    fn send_ready(&self) -> bool;

    /// Send a single byte down the transmit wire.
    ///
    /// Implementors of this function can take as a precondition that
    /// `self.send_ready()` returns true, as the kernel always checks that
    /// before calling this.
    ///
    /// Calls to this function should never block.
    fn send(&mut self, byte: u8);

    /// Checks to see if the UART has data in its buffer to receive.
    ///
    /// This function is called repeatedly and often as the kernel tries to
    /// schedule the best time for receiving, so try to
    /// make it performant.
    fn receive_ready(&self) -> bool;

    /// Receive a single byte from the receive wire.
    ///
    /// Read binary data from the UART. If the data is meant to be human-readable, use
    /// [Uart::receive_native] instead.
    ///
    /// Implementors of this function can take as a precondition that
    /// `self.receive_ready()` returns true, as the kernel always checks that
    /// before calling this.
    ///
    /// Calls to this function should never block.
    fn receive(&mut self) -> u8;

    /// Receive a single byte from the receive wire, converting to the kernel locale.
    ///
    /// Most of the time, this does the same thing as [Uart::receive]. However, many UART devices take
    /// the position that newlines are encoded with `\r` instead of `\n`. This doesn't match with what
    /// the rest of the kernel expects. When the data coming from the UART is meant to be read by
    /// humans, use this function to perform the necessary conversions. For binary data, use
    /// [Uart::receive].
    ///
    /// Implementors of this trait may choose to perform additional, or zero, conversions as part of
    /// this function depending on the semantics of their device.
    fn receive_native(&mut self) -> u8 {
        match self.receive() {
            b'\r' => b'\n',
            c => c,
        }
    }
}

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Flag Register
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, UARTLCR_ LCRH.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty. If
        /// the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does
        /// not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_ LCRH Register.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is full. If
        /// the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_H Register.
        ///
        /// If the FIFO is disabled, this bit is set when the receive holding register is empty. If
        /// the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) []
    ],

    /// Integer Baud rate divisor
    IBRD [
        /// Integer Baud rate divisor
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud rate divisor
    FBRD [
        /// Fractional Baud rate divisor
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control register
    LCRH [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers
        ///
        /// 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for UART signals. When the UART is disabled in the middle of
        /// reception, it completes the current character before stopping.
        RXE    OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for UART signals. When the UART is disabled in the middle of
        /// transmission, it completes the current character before stopping.
        TXE    OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCRH: WriteOnly<u32, LCRH::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

// FIXME:
// Currently this is unsafe since it gives shared mutability over the register block. Really this
// should contain some sort of Mutex to guard interior mutability.
pub struct PL011Uart {
    regs: &'static mut RegisterBlock
}

impl PL011Uart {
    /// # Safety
    /// The user must verify that the address for the register block is correct.
    pub unsafe fn new(base_address: usize) -> Self {
        Self {
            regs: &mut *(base_address as *mut _)
        }
    }

    pub fn init(&self, gpio: &mut Gpio, _baud: u32) -> Result<(), driver::Error> {
        gpio.setup_uart();

        // set control register to 0 to turn off during initialization phase
        self.regs.CR.set(0);

        // initialize UART
        self.regs.ICR.write(ICR::ALL::CLEAR); // clear pending interrupts
        // set dividers for 921_600 baud
        self.regs.IBRD.write(IBRD::BAUD_DIVINT.val(3));
        self.regs.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
        self.regs.LCRH.write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8bit chars + Fifo on
        // enable UART + enable transmit + enable receive
        self.regs.CR.write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);

        Ok(())
    }
}

impl Uart for PL011Uart {
    fn send_ready(&self) -> bool {
        !self.regs.FR.is_set(FR::TXFF)
    }

    fn send(&mut self, byte: u8) {
        while !self.send_ready() {
            arch::asm::nop()
        }

        // write the character to the buffer
        // Since we have the FIFO enabled, writing to this buffer may not immediately result in a
        // full buffer. `self.regs.FR` could still indicate that we're ready to send.
        self.regs.DR.set(byte as u32)
    }

    fn receive_ready(&self) -> bool {
        !self.regs.FR.is_set(FR::RXFE)
    }

    fn receive(&mut self) -> u8 {
        while !self.receive_ready() {
            arch::asm::nop()
        }

        // read the character from the buffer
        self.regs.DR.get() as u8
    }
}

impl ufmt::uWrite for PL011Uart {
    type Error = core::convert::Infallible;

    fn write_str(&mut self, msg: &str) -> Result<(), Self::Error> {
        for byte in msg.bytes() {
            if byte == b'\n' {
                self.send(b'\r')
            }
            self.send(byte)
        }
        Ok(())
    }
}

impl Driver for PL011Uart {
    fn compatible(&self) -> &'static str {
        "BCM PL011 UART"
    }
}

impl AsMut<dyn Driver> for PL011Uart {
    fn as_mut(&mut self) -> &mut (dyn Driver + 'static) {
        self
    }
}

impl AsMut<dyn ufmt::uWrite<Error=Infallible>> for PL011Uart {
    fn as_mut(&mut self) -> &mut (dyn ufmt::uWrite<Error=Infallible> + 'static) {
        self
    }
}
