use cortex_a::asm::*;

pub use cortex_a::asm::nop as _nop;

#[inline(always)]
pub fn _wait_forever() -> ! {
    loop {
        wfe()
    }
}
