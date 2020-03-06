use register::{
    mmio::{ReadOnly, ReadWrite},
    register_bitfields,
};

const INTC_BASE: u32 = super::MMIO_BASE + 0xB200;

pub struct InterruptId {}
impl InterruptId {
    pub const DMA: u32 = 16;
    pub const TIMER1: u32 = 1;
    pub const TIMER3: u32 = 1;
}

pub struct BasicInterruptId {}
impl BasicInterruptId {
    pub const ARM_TIMER: u32 = 0;
}

pub struct Interrupt {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    BASIC_PENDING: ReadOnly<u32, BASIC_PENDING::Register>,
    /// Holds interrupts from GPU side. INT no. 0-63.
    IRQ_PENDING: [ReadWrite<u32>; 2],
    FIQ_CONTROL: ReadWrite<u32>,
    /// Write 1 to enable
    ENABLE_IRQ: [ReadWrite<u32>; 2],
    ENABLE_BASIC_IRQ: ReadWrite<u32, ENABLE_BASIC::Register>,
    /// Write 1 to disable
    DISABLE_IRQ: [ReadWrite<u32>; 2],
    DISABLE_BASIC_IRQ: ReadWrite<u32>,
}

register_bitfields! {
    u32,
    BASIC_PENDING[
        /// GPU interrupts.
        GPU_IRQ_62 OFFSET(20) NUMBITS(1)[],
        GPU_IRQ_57 OFFSET(19) NUMBITS(1)[],
        GPU_IRQ_56 OFFSET(18) NUMBITS(1)[],
        GPU_IRQ_55 OFFSET(17) NUMBITS(1)[],
        GPU_IRQ_54 OFFSET(16) NUMBITS(1)[],
        GPU_IRQ_53 OFFSET(15) NUMBITS(1)[],
        GPU_IRQ_19 OFFSET(14) NUMBITS(1)[],
        GPU_IRQ_18 OFFSET(13) NUMBITS(1)[],
        GPU_IRQ_10 OFFSET(12) NUMBITS(1)[],
        GPU_IRQ_9 OFFSET(11) NUMBITS(1)[],
        GPU_IRQ_7 OFFSET(10) NUMBITS(1)[],
        /// One or more bits are in pending at IRQ_PENDING[1]
        PENDING_1 OFFSET(9) NUMBITS(1)[],
        /// One or more bits are in pending at IRQ_PENDING[0]
        PENDING_0 OFFSET(8) NUMBITS(1)[],
        /// Out of bounds access(bit 31/30) on ARM periphral bus.
        ILLEGAL_ACCESS_TYPE0 OFFSET(7) NUMBITS(1) [],
        /// Out of bounds access(bit 29..26) on GPU Peripheral bus.
        ILLEGAL_ACCESS_TYPE1 OFFSET(6) NUMBITS(1) [],
        GPU1_HALTED_IRQ OFFSET(5) NUMBITS(1)[],
        GPU0_HALTED_IRQ OFFSET(4) NUMBITS(1)[],
        ARM_DOORBELL_1_IRQ_PENDING OFFSET(3) NUMBITS(1)[],
        ARM_DOORBELL_0_IRQ_PENDING OFFSET(2) NUMBITS(1)[],
        ARM_MAILBOX_IRQ_PENDING OFFSET(1) NUMBITS(1)[],
        ARM_TIMER_IRQ_PENDING OFFSET(0) NUMBITS(1)[]
    ],
    ENABLE_BASIC[
        UNUSED OFFSET(8) NUMBITS(24)[]
        // bit 0-7: enable bits for irqs like illegal access, etc.
    ]
}

impl core::ops::Deref for Interrupt {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

#[allow(dead_code)]
impl Interrupt {
    pub fn new() -> Interrupt {
        Interrupt {}
    }

    fn ptr() -> *const RegisterBlock {
        INTC_BASE as *const _
    }

    pub fn get_raw_pending(&self) -> u64 {
        let l = self.IRQ_PENDING[0].get();
        let h = self.IRQ_PENDING[1].get();
        return ((h as u64) << (32 as u64)) + l as u64;
    }

    pub fn enable_irq(&self, id: u32) {
        if id < 32 {
            self.ENABLE_IRQ[0].set(1 << id);
        } else if id < 64 {
            self.ENABLE_IRQ[1].set(1 << (id - 32));
        } else {
            return;
        }
    }

    pub fn disable_irq(&self, id: u32) {
        if id < 32 {
            self.DISABLE_IRQ[0].set(1 << id);
        } else if id < 64 {
            self.DISABLE_IRQ[1].set(1 << (id - 32));
        } else {
            return;
        }
    }

    pub fn is_irq_enabled(&self, id: u32) -> bool {
        if id < 32 {
            (self.IRQ_PENDING[0].get() & (1 << id)) != 0
        } else if id < 64 {
            (self.IRQ_PENDING[1].get() & (1 << (id - 32))) != 0
        } else {
            return false;
        }
    }

    pub fn enable_basic_irq(&self, id: u32) {
        if id < 8 {
            self.ENABLE_BASIC_IRQ.set(1 << id);
        }
    }

    pub fn disable_basic_irq(&self, id: u32) {
        if id < 8 {
            self.DISABLE_BASIC_IRQ.set(1 << id);
        }
    }

    pub fn is_basic_irq_pending(&self, id: u32) -> bool {
        if id < 8 {
            return (self.BASIC_PENDING.get() & (1 << id)) != 0;
        }

        false
    }

    pub fn is_any_irq_pending(&self) -> bool {
        self.BASIC_PENDING.is_set(BASIC_PENDING::PENDING_0)
            || self.BASIC_PENDING.is_set(BASIC_PENDING::PENDING_1)
    }

    pub fn get_raw_basic_pending(&self) -> u32 {
        self.BASIC_PENDING.get()
    }
}
