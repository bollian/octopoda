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
