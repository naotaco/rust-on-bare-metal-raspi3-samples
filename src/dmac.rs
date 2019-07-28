use super::MMIO_BASE;
use core::ops;
use register::{
    mmio::{ReadOnly, WriteOnly},
    register_bitfields,
};

pub struct DMAC{
    some_data : u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    READ: ReadOnly<u32>,                     // 0x00
}

const DMAC_BASE :u32 = super::MMIO_BASE + 0;

impl ops::Deref for DMAC {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl DMAC {
    pub fn new() -> DMAC {
        DMAC { some_data: 0 }
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        DMAC_BASE as *const _
    }
}