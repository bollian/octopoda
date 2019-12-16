// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::MMIO_BASE;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::gpio;
use register::mmio::{ReadOnly, ReadWrite, WriteOnly};
use register::register_bitfields;

pub trait Uart {
    fn init(&self);

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
    fn send(&self, byte: u8);


    /// Checks to see if the UART has data in its buffer to receive.
    ///
    /// This function is called repeatedly and often as the kernel tries to
    /// schedule the best time for receiving, so try to
    /// make it performant.
    fn receive_ready(&self) -> bool;

    /// Receive a single byte from the receive wire.
    ///
    /// Implementors of this function can take as a precondition that
    /// `self.receive_ready()` returns true, as the kernel always checks that
    /// before calling this.
    ///
    /// Calls to this function should never block.
    fn receive(&self) -> u8;
}

pub struct UartPutsFuture<'uart, 'message> {
    uart: &'uart dyn Uart,
    message: &'message str,
}

impl Future for UartPutsFuture<'_, '_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        let uart = self.uart;
        for i in 0..self.message.len() {
            if uart.send_ready() {
                uart.send(self.message.as_bytes()[i]);
            } else {
                self.get_mut().message = &self.message[i..];
                return Poll::Pending;
            }
        }
        return Poll::Ready(());
    }
}

impl<'uart, 'msg, U: Uart> crate::ext::Puts<'uart, 'msg> for U {
    type Future = UartPutsFuture<'uart, 'msg>;

    fn puts(&'uart self, message: &'msg str) -> Self::Future {
        UartPutsFuture {
            uart: self,
            message: message,
        }
    }
}

// Auxilary mini UART registers
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// Auxilary Enables
    AUX_ENABLES [
        /// If set, the mini UART is enabled. The UART will immediately
        /// start receiving data, especially if the UART1_RX line is low.
        /// If clear, the mini UART is disabled. That also disables any
        /// mini UART register access.
        MINI_UART_ENABLE OFFSET(0) NUMBITS(1) []
    ],

    /// Mini UART Interrupt Identify
    AUX_MU_IIR [
        /// Writing with bit 1 set will clear the receive FIFO
        /// Writing with bit 2 set will clear the transmit FIFO
        FIFO_CLEAR OFFSET(1) NUMBITS(2) [
            Rx  = 0b01,
            Tx  = 0b10,
            All = 0b11
        ]
    ],

    /// Mini UART Line Control
    AUX_MU_LCR [
        /// Mode the UART works in
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            SevenBit = 0b00,
            EightBit = 0b11
        ]
    ],

    /// Mini UART Line Status
    AUX_MU_LSR [
        /// This bit is set if the transmit FIFO can accept at least one byte.
        TX_EMPTY   OFFSET(5) NUMBITS(1) [],

        /// This bit is set if the receive FIFO holds at least 1 symbol.
        DATA_READY OFFSET(0) NUMBITS(1) []
    ],

    /// Mini UART Extra Control
    AUX_MU_CNTL [
        /// If this bit is set, the mini UART transmitter is enabled.
        TX_EN OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled  = 1
        ],

        /// If this bit is set, the mini UART receiver is enabled.
        RX_EN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled  = 1
        ]
    ],

    /// Mini UART Baudrate
    AUX_MU_BAUD [
        /// Mini UART baudrate counter
        RATE OFFSET(0) NUMBITS(16) []
    ]
}

const MINI_UART_BASE: u32 = MMIO_BASE + 0x21_5000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                                  // 0x00
    AUX_ENABLES: ReadWrite<u32, AUX_ENABLES::Register>, // 0x04
    __reserved_1: [u32; 14],                            // 0x08
    AUX_MU_IO: ReadWrite<u32>,                          // 0x40 - Mini Uart I/O Data
    AUX_MU_IER: WriteOnly<u32>,                         // 0x44 - Mini Uart Interrupt Enable
    AUX_MU_IIR: WriteOnly<u32, AUX_MU_IIR::Register>,   // 0x48
    AUX_MU_LCR: WriteOnly<u32, AUX_MU_LCR::Register>,   // 0x4C
    AUX_MU_MCR: WriteOnly<u32>,                         // 0x50
    AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>,    // 0x54
    __reserved_2: [u32; 2],                             // 0x58
    AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>, // 0x60
    __reserved_3: u32,                                  // 0x64
    AUX_MU_BAUD: WriteOnly<u32, AUX_MU_BAUD::Register>, // 0x68
}

pub struct MiniUart;

impl MiniUart {
    pub fn new() -> MiniUart {
        MiniUart
    }

    pub fn regs(&self) -> &RegisterBlock {
        unsafe { &*Self::ptr() }
    }

    pub fn ptr() -> *const RegisterBlock {
        MINI_UART_BASE as *const _
    }
}

impl Uart for MiniUart {
    fn init(&self) {
        let regs = self.regs();

        // initialize UART
        regs.AUX_ENABLES.modify(AUX_ENABLES::MINI_UART_ENABLE::SET);
        regs.AUX_MU_IER.set(0);
        regs.AUX_MU_CNTL.set(0);
        regs.AUX_MU_LCR.write(AUX_MU_LCR::DATA_SIZE::EightBit);
        regs.AUX_MU_MCR.set(0);
        regs.AUX_MU_IER.set(0);
        regs.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        regs.AUX_MU_BAUD.write(AUX_MU_BAUD::RATE.val(270)); // 115200 baud

        // map UART1 to GPIO pins
        unsafe {
            (*gpio::GPFSEL1).modify(gpio::GPFSEL1::FSEL14::TXD1 + gpio::GPFSEL1::FSEL15::RXD1);

            (*gpio::GPPUD).set(0); // enable pins 14 and 15
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }

            (*gpio::GPPUDCLK0).write(
                gpio::GPPUDCLK0::PUDCLK14::AssertClock + gpio::GPPUDCLK0::PUDCLK15::AssertClock,
            );
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }

            (*gpio::GPPUDCLK0).set(0);
        }
    }

    fn send_ready(&self) -> bool {
        self.regs().AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY)
    }

    fn send(&self, byte: u8) {
        self.regs().AUX_MU_IO.set(byte as u32);
    }

    fn receive_ready(&self) -> bool {
        self.regs().AUX_MU_LSR.is_set(AUX_MU_LSR::DATA_READY)
    }

    fn receive(&self) -> u8 {
        self.regs().AUX_MU_IO.get() as u8
    }
}
