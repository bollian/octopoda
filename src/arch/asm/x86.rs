#[inline(always)]
pub fn _nop() {
    unsafe {
        llvm_asm!("nop" :::: "volatile");
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
