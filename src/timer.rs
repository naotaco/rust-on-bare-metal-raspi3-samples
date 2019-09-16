use register::{
    mmio::{ReadOnly, ReadWrite},
    register_bitfields,
};

const TIMER_BASE: u32 = super::MMIO_BASE + 0x3000;

pub struct TIMER {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CS: ReadWrite<u32>,
    CLO: ReadOnly<u32, CLO::Register>,
    CLH: ReadOnly<u32, CLH::Register>,
    C0: ReadWrite<u32>,
    C1: ReadWrite<u32>,
    C2: ReadWrite<u32>,
    C3: ReadWrite<u32>,
}

register_bitfields! {
    u32,
    /// Control and status. common for 0-14 ch.
    CLO [
        TIME OFFSET(0) NUMBITS(32) [ ]
    ],
    CLH [ TIME OFFSET(0) NUMBITS(32) [ ] ]
}

impl core::ops::Deref for TIMER {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl TIMER {
    pub fn new() -> TIMER {
        TIMER {}
    }
    fn ptr() -> *const RegisterBlock {
        TIMER_BASE as *const _
    }
    pub fn get_counter64(&self) -> u64 {
        let h: u64 = self.CLH.read(CLH::TIME) as u64;
        let l: u64 = self.CLO.read(CLO::TIME) as u64;
        (h << 32) + l
    }

    pub fn get_counter32(&self) -> u32 {
        self.CLO.read(CLO::TIME)
    }
}
