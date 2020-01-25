use super::MMIO_BASE;
use core::ops::{Deref, DerefMut};
use register::{
    mmio::{ReadOnly, ReadWrite, WriteOnly},
    register_bitfields,
};

const TIMER_BASE: u32 = super::MMIO_BASE + 0xB400;

pub struct ArmTimer {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    LOAD: ReadWrite<u32>,
    VALUE: ReadOnly<u32>,
    CONTROL: ReadWrite<u32, CONTROL::Register>,
    IRQ_CLEAR: WriteOnly<u32>,
    RAW_IRQ: ReadOnly<u32, RAW_IRQ::Register>,
    MASKED_IRQ: ReadOnly<u32, MASKED_IRQ::Register>,
    RELOAD: ReadWrite<u32>,
    PRE_DIVIDER: ReadWrite<u32>,
    FREE_RUN_COUNTER: ReadWrite<u32>,
}

register_bitfields! {
    u32,
    /// Control and status. common for 0-14 ch.
    CONTROL[
        PRE_SCALAR OFFSET(16) NUMBITS(8)[],
        FREE_RUN OFFSET(9) NUMBITS(1) [
            Enabled=1,
            Disabled=0
        ],
        RUN_IN_HALT OFFSET(8) NUMBITS(1)[],
        ENABLED OFFSET(7) NUMBITS(1)[],
        INT_EN OFFSET(5) NUMBITS(1)[],
        PRE_SCALE OFFSET(2) NUMBITS(2)[
            C_1_1=0, // clock/1
            C_1_16=1, // clock /16
            C_1_256=2 // clock /256
        ],
        BIT_WIDTH OFFSET(1) NUMBITS(1)[
            BIT_16=0,
            BIT_23=1
        ]
    ],
    RAW_IRQ[
        PENDING OFFSET(0) NUMBITS(1)[]
    ],
    MASKED_IRQ[
        ASSERTED OFFSET(0) NUMBITS(1)[]
    ]
}

impl core::ops::Deref for ArmTimer {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl ArmTimer {
    pub fn new() -> ArmTimer {
        ArmTimer {}
    }
    fn ptr() -> *const RegisterBlock {
        TIMER_BASE as *const _
    }

    pub fn StartFreeRun(&self) {
        self.CONTROL
            .modify(CONTROL::FREE_RUN::Enabled + CONTROL::BIT_WIDTH::BIT_23);
    }

    pub fn ReadFreeFun(&self) -> u32 {
        self.FREE_RUN_COUNTER.get()
    }
}