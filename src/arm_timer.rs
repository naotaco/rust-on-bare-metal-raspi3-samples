use crate::optional_cell::OptionalCell;
use register::{
    mmio::{ReadOnly, ReadWrite, WriteOnly},
    register_bitfields,
};

const TIMER_BASE: u32 = super::MMIO_BASE + 0xB400;

pub struct ArmTimer {
    fired: OptionalCell<bool>,
}

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
        ENABLED OFFSET(7) NUMBITS(1)[
            Enabled=1,
            Disabled=0
        ],
        INT_EN OFFSET(5) NUMBITS(1)[
            Enabled=1,
            Disabled=0
        ],
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
        PENDING OFFSET(0) NUMBITS(1)[
            Enabled=1,
            Disabled=0
        ]
    ],
    MASKED_IRQ[
        ASSERTED OFFSET(0) NUMBITS(1)[
            Enabled=1,
            Disabled=0
        ]
    ]
}

impl core::ops::Deref for ArmTimer {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

const BASIC_INT_NO_ARM_TIMER: u32 = 0;

impl crate::exception::InterruptionSource for ArmTimer {
    fn on_interruption(&self, id: u32) {
        if id == BASIC_INT_NO_ARM_TIMER {
            self.clear_irq();
            self.fired.insert(Some(true));
        }
    }
}

#[allow(dead_code)]
impl ArmTimer {
    pub fn new() -> ArmTimer {
        ArmTimer {
            fired: OptionalCell::new(false),
        }
    }
    fn ptr() -> *const RegisterBlock {
        TIMER_BASE as *const _
    }

    pub fn enable(&self) {
        self.CONTROL.modify(CONTROL::ENABLED::Enabled);
    }

    pub fn start_free_run(&self) {
        self.CONTROL
            .modify(CONTROL::FREE_RUN::Enabled + CONTROL::BIT_WIDTH::BIT_23);
    }

    pub fn read_free_run(&self) -> u32 {
        self.FREE_RUN_COUNTER.get()
    }

    pub fn set_count_down(&self, t: u32) {
        self.LOAD.set(t);
    }

    pub fn read_count_down(&self) -> u32 {
        self.VALUE.get()
    }

    pub fn enable_int(&self) {
        self.CONTROL.modify(CONTROL::INT_EN::Enabled);
    }

    pub fn clear_irq(&self) {
        self.IRQ_CLEAR.set(1);
    }

    pub fn has_fired(&self) -> bool {
        match self.fired.take() {
            Some(f) => f,
            None => false,
        }
    }
}
