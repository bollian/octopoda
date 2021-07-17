/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod gpio;
pub mod uart;

#[derive(Debug)]
pub enum Error {
    Unknown
}

pub trait Driver {
    /// Compatibility string identifying the driver
    fn compatible(&self) -> &'static str;
}
