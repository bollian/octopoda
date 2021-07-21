/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Driver for writing to the text-mode VGA that all x86 systems have.

use crate::driver::WriteError;
use crate::driver::traits::Driver;

const COLUMN_COUNT: u8 = 80;
const LINE_COUNT: u8 = 25;

#[repr(C)]
#[derive(Copy, Clone)]
struct VgaChar {
    character: u8,
    style: VgaStyle,
}

impl VgaChar {
    const WHITESPACE: Self = Self::from_ascii(b' ');

    const fn from_char(c: char) -> Option<Self> {
        if c.is_ascii() {
            Some(Self::from_ascii(c as u8))
        } else {
            None
        }
    }

    const fn from_ascii(c: u8) -> Self {
        Self {
            character: c,
            style: VgaStyle::WHITE_ON_BLACK,
        }
    }

    const fn style(mut self, style: VgaStyle) -> Self {
        self.style = style;
        self
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct VgaStyle(u8);

impl VgaStyle {
    const WHITE_ON_BLACK: Self = Self::new(VgaColor::LightGrey, true, VgaColor::Black, false);

    const fn new(foreground: VgaColor, bright: bool, background: VgaColor, blink: bool) -> Self {
        let foreground = foreground as u8;
        let bright = (bright as u8) << 3;
        let background = (background as u8) << 4;
        let blink = (blink as u8) << 7;
        Self(foreground | bright | background | blink)
    }
}

pub struct TextVga {
    /// Index of the line currently being written to
    line_offset: u8,
    /// Index of the column currently being written to
    column_offset: u8,
    /// The VGA text buffer
    buffer: *mut [[VgaChar; COLUMN_COUNT as usize]; LINE_COUNT as usize],
}

unsafe impl Sync for TextVga {}
unsafe impl Send for TextVga {}

impl TextVga {
    pub unsafe fn new(base_address: usize) -> Self {
        Self {
            line_offset: 0,
            column_offset: 0,
            buffer: base_address as *mut _,
        }
    }

    fn set_char(&mut self, line: usize, column: usize, c: VgaChar) {
        use core::ptr::addr_of_mut;

        unsafe {
            let line = addr_of_mut!((*self.buffer)[line]);
            let char = addr_of_mut!((*line)[column]);
            char.write_volatile(c);
        }
    }

    fn bump_cursor(&mut self) {
        self.column_offset += 1;
        if self.column_offset >= COLUMN_COUNT {
            // move to the next line
            self.column_offset = 0;
            self.bump_line();
        }
    }

    fn bump_line(&mut self) {
        self.line_offset += 1;
        self.column_offset = 0;
        if self.line_offset >= LINE_COUNT {
            self.scroll();
            self.line_offset = LINE_COUNT - 1;
        }
    }

    fn scroll(&mut self) {
        // scroll the text upwards
        let dest = self.buffer as *mut [[VgaChar; COLUMN_COUNT as usize]; LINE_COUNT as usize - 1];
        // SAFETY: we offset by 1, so we can only copy LINE_COUNT - 1 entries. Type type
        // cast above ensures that's the case
        unsafe {
            let src = (self.buffer as *mut [VgaChar; COLUMN_COUNT as usize]).offset(1) as *mut _;
            core::ptr::copy(src, dest, 1);
        }

        // overwrite the last line with spaces
        let last_line = (LINE_COUNT - 1) as usize;
        let whitespace = VgaChar::from_ascii(b' ');
        for column in 0..COLUMN_COUNT {
            self.set_char(last_line, column as usize, whitespace);
        }
    }

    fn write_vga_char(&mut self, c: VgaChar) {
        match c {
            VgaChar { character: b'\n', .. } => {
                self.bump_line();
            }
            VgaChar { character: b'\r', .. } => {
                self.column_offset = 0;
            }
            VgaChar { character: b'\t', .. } => {
                for _ in 0..8 {
                    self.set_char(
                        self.line_offset as usize,
                        self.column_offset as usize,
                        VgaChar::WHITESPACE
                    );
                    self.bump_cursor();
                }
            }
            c => {
                self.set_char(self.line_offset as usize, self.column_offset as usize, c);
                self.bump_cursor();
            }
        }
    }
}

impl ufmt::uWrite for TextVga {
    type Error = WriteError;

    fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
        match VgaChar::from_char(c) {
            Some(c) => {
                self.write_vga_char(c);
                Ok(())
            }
            None => Err(WriteError::UnicodeUnsupported),
        }
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.bytes() {
            // We assume here that the user knows the encoding of the textmode VGA.
            // Artifacts produced by using invalid ascii are hopefully visible enough to tell
            // someone they've used the device wrong, because we still need to support printing
            // those symbols.
            let c = VgaChar::from_ascii(c);
            self.write_vga_char(c);
        }
        return Ok(())
    }
}

impl Driver for TextVga {
    const COMPATIBLE: &'static str = "Textmode VGA";
}

impl AsMut<dyn ufmt::uWrite<Error=WriteError>> for TextVga {
    fn as_mut(&mut self) -> &mut (dyn ufmt::uWrite<Error=WriteError> + 'static) {
        self
    }
}
