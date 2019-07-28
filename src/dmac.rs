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
/// Only for ch 0-6. from 7th ch, differenct map is assigned.
pub struct RegisterBlock {
    CS: ReadWrite<u32, CS::Register>,             // 0x00, Status
    CONBLK_AD: ReadWrite<u32>,                    // 0x04, Control block address
    TI: ReadWrite<u32, TI::Register>,             // 0x08, CB word 0
    SOURCE_AD: u32,                               // 0x0c, CB word 1
    DEST_AD: u32,                                 // 0x10, CB word 2
    TXFR_LEN: ReadWrite<u32, TXFR_LEN::Register>, // 0x14, CB word 3
    STRIDE: ReadWrite<u32, STRIDE::Register>,     // 0x18, CB word 4
    NEXTCONBK: u32,                               // 0x1c, CB word 5
    DEBUG: ReadWrite<u32, DEBUG::Register>,       // 0x20, debug
}

register_bitfields! {
    u32,
    /// Control and status. common for 0-14 ch.
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
        ERROR OFFSET(8) NUMBITS(1) [
            Error = 1,
            Ok = 0
        ],
        // 7 : reserved
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [
            Waiting = 1, // DMA is waiting the last write
            None = 0
        ],
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [
            Paused = 1, // DMA is paused due to DREQ inactive
            Running = 0
        ],
        PAUSED OFFSET(4) NUMBITS(1) [
            Paused = 1, // Paused due to cleared active bit etc.
            Running = 0
        ],
        DREQ OFFSET(3) NUMBITS(1) [
            Requesting = 1, // DREQ set: requesting data
            NotRequesting = 0
        ],
        INT OFFSET(2) NUMBITS(1) [
            Clear = 1 // write to clear interrupt.
        ],
        END OFFSET(1) NUMBITS(1) [
            Clear = 1 // write 1 to clear. also readable as 1 when trans has end.
        ],
        ACTIVE OFFSET (0) NUMBITS(1) [
            Enable = 1, // Run or resume DMA after writing CB addr
            Pause = 0 // pause
        ]
    ],
    /// Trans Info. common for 0-6 ch
    TI [
        // 31-27: reserved. write as 0.
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) [
            PreventWide = 1, // prevent 2 beat AXI bursts access.
            NoRestriction = 0
        ],
        /// number of dummy cycle(s) after each DMA read or write.
        WAITS OFFSET(21) NUMBITS(5) [ ],
        /// peripheral mapping.
        /// peripheral number (1-31) used to control transfer rate. 0 for continuous.
        PERMAP OFFSET(16) NUMBITS(5) [],
        /// number of bursts that DMA will try. 0 for single transfer.
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
        SRC_IGNORE OFFSET(11) NUMBITS(1) [
            DontReadSource = 1, // data will be zero: for zero-fill operations.
            Standard = 0
        ],
        SRC_DREQ OFFSET(10) NUMBITS(1) [
            Enabled = 1, // DREQ selected by PER_MAP will starts source reads.
            Disabled = 0 // Ignore DREQ
        ],
        SRC_WIDTH OFFSET(9) NUMBITS(1) [
            Use128Bits = 1,
            Use32Bits = 0
        ],
        SRC_INC OFFSET(8) NUMBITS(1) [
            Enabled = 1, // source addr increment
            Disabled = 0 // source addr fixed
        ],
        DEST_IGNORE OFFSET(7) NUMBITS(1) [
            DontWriteToDest = 1,
            WriteAsStandard = 0
        ],
        DEST_DREQ OFFSET(6) NUMBITS(1) [
            Enabled = 1, // DREQ selected by PER_MAP will starts dest write
            Disabled = 0 // Ignore DREQ
        ],
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
    ],
    // for 0-6 ch
    TXFR_LEN [
        // 31-30: reserved.
        /// In 2d mode, y trans length (time of x trans).
        YLENGTH OFFSET(16) NUMBITS(14)[],
        /// Transfer length in bytes.
        XLENGTH OFFSET(0) NUMBITS(16)[]
    ],
    // for 0-6 ch.
    STRIDE[
        /// signed bytes increment to apply to dest addr at each row.
        D_STRIDE OFFSET(16) NUMBITS(16)[],
        /// signed bytes increment to apply to source addr at each row.
        S_STRIDE OFFSET(16) NUMBITS(16)[]
    ],
    DEBUG[
        // 31-29
        LITE OFFSET(28) NUMBITS(1) [
            Lite = 1 // set if the DMA is a reduced one
        ],
        VERSION OFFSET(25) NUMBITS(3) [],
        // DMA state machine number
        DMA_STATE OFFSET(16) NUMBITS(9) [],
        // DMA AXI ID
        DMA_ID OFFSET(8) NUMBITS(8) [],
        /// DMA Outstanding writes count
        OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
        // 3
        READ_ERROR OFFSET(2) NUMBITS(1) [
            Clear = 1 // Read error occured, write 1 to clear.
        ],
        FIFO_ERROR OFFSET(1) NUMBITS(1) [
            Clear = 1 // FIFO error exists, write 1 to clear
        ],
        READ_LAST_NOT_SET_ERROR OFFSET(0) NUMBITS(1) [
            Clear = 1 // last signal was not send as expect. W1C.
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
