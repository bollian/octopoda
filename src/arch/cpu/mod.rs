pub mod smp {
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn core_id() -> u64 {
        use cortex_a::regs::*;
        MPIDR_EL1.get() & 0b11
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[inline(always)]
    pub fn core_id() -> u64 {
        todo!()
    }
}
