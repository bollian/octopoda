pub mod gpio;
pub mod uart;

#[derive(Debug)]
pub enum Error {
    Unknown
}

pub trait Driver {
    fn init(&mut self) -> Result<(), Error>;
}
