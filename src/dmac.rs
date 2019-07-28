use super::MMIO_BASE;
use core::ops;
use register::{
    mmio::{ReadOnly, ReadWrite, WriteOnly},
    register_bitfields,
};

pub struct DMAC {
    _some_data: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CS: ReadWrite<u32, CS::Register>, // 0x00, Status
    CONBLK_AD: ReadWrite<u32>,        // 0x04, Control block address
    TI: ReadWrite<u32>,               // 0x08, CB word 0
    SOURCE_AD: u32,                   // 0x0c, CB word 1
    DEST_AD: u32,                     // 0x10, CB word 2
    TXFR_LEN: u32,                    // 0x14, CB word 3
    STRIDE: u32,                      // 0x18, CB word 4
    NEXTCONBK: u32,                   // 0x1c, CB word 5
    DEBUG: u32,                       // 0x20, debug
}

register_bitfields! {
    u32,
    CS [
        RESET OFFSET(31) NUMBITS(1) [
            Reset = 1 // write to perform reset
        ],
        ABORT OFFSET(30) NUMBITS(1) [
            Abort = 1 // write to abort
        ],
        DISDEBUG OFFSET(29) NUMBITS(1) [
            DisableDebugPause = 1, /// DMA ignores debug pause signal
            Normal = 0
        ],
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [

        ],
        // 27-24: reserved. Write as 0.
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
        PRIORITY OFFSET(16) NUMBITS(4) [],
        // 15-9: reserved. Write as 0.
        ERROR OFFSET(8) NUMBITS(1) [],
        // 7 : reserved
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
        PAUSED OFFSET(4) NUMBITS(1) [],
        DREQ OFFSET(3) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) [],
        END OFFSET(1) NUMBITS(1) [],
        ACTIVE OFFSET (0) NUMBITS(1) []
    ],
    TI [
        // 31-27: reserved. write as 0.
        NO_WIDE_BURTS OFFSET(26) NUMBITS(1) [],
        WAITS OFFSET(21) NUMBITS(5) [],
        PERMAP OFFSET(16) NUMBITS(5) [],
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
        SRC_INC OFFSET(8) NUMBITS(1) [],
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
        DEST_WIDTH OFFSET(5) NUMBITS(1) [
            Use128Bits = 1,
            Use32Bits = 0
        ],
        DEST_INC OFFSET(4) NUMBITS(1) [
            Enabled = 1, /// Dest address increments
            Disabled = 0
        ],
        WAIT_RESP OFFSET(3) NUMBITS(1) [
            Wait = 1,
            NoWait = 0
        ],
        // 2: reserved.
        TDMODE OFFSET(1) NUMBITS(1) [
            TdMode = 1,
            LinearMode = 0
        ],
        INTEN OFFSET(0) NUMBITS(1) [
            Enabled = 1,
            Disabled = 0
        ]
    ]

}

/// Data structure used to order DMA settings/options.
/// Write data on DDR accordingly and tell it's address to DMA.
/// Values ordered by these members can be observed on register as comment follows.
#[repr(align(256))]
pub struct ControlBlock {
    transfer_information: u32,       // 0x00, accociated to TI register.
    source_address: u32,             // 0x04, SOURCE_AD
    destination_address: u32,        // 0x08, DEST_AD
    transfer_length: u32,            // 0x0C, TXFR_LEN
    two_d_mode_stride: u32,          // 0x10, STRIDE
    next_control_block_address: u32, // 0x14, NEXTCONBK
    __reserved: [u32; 2],            // N/A
}

const DMAC_BASE: u32 = super::MMIO_BASE + 0x7200;

impl ops::Deref for DMAC {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl DMAC {
    pub fn new() -> DMAC {
        DMAC { _some_data: 0 }
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        DMAC_BASE as *const _
    }

    pub fn reset(&self) {
        // self.DR.set(c as u32);
        self.CS.write(CS::RESET::Reset);
    }
}
