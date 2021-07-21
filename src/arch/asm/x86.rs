#[inline(always)]
pub fn _nop() {
    unsafe {
        asm!("nop");
    }
}

#[inline(always)]
pub fn _wait_forever() -> ! {
    unsafe {
        loop {
            x86::halt()
        }
    }
}
