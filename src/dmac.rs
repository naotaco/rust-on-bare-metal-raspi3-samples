use core::ops::{Deref, DerefMut};
use core::sync::atomic::compiler_fence;
use register::{mmio::ReadWrite, register_bitfields, InMemoryRegister};

pub struct DMAC {
    _some_data: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    Channels: [DmaChannelRegister; 15], // ch 0 - 15
    __reserved: [u32; 0x38],
    INT_STATUS: ReadWrite<u32, GLOBAL_INT::Register>, // 0xfe0
    __reserved1: [u32; 0x3],
    ENABLE: ReadWrite<u32, GLOBAL_ENABLE::Register>, // 0xff0
}

#[allow(non_snake_case)]
#[repr(C)]
/// Only for ch 0-6. from 7th ch, differenct map is assigned.
pub struct DmaChannelRegister {
    CS: ReadWrite<u32, CS::Register>,             // 0x00, Status
    CONBLK_AD: ReadWrite<u32>,                    // 0x04, Control block address
    TI: ReadWrite<u32, TI::Register>,             // 0x08, CB word 0
    SOURCE_AD: u32,                               // 0x0c, CB word 1
    DEST_AD: u32,                                 // 0x10, CB word 2
    TXFR_LEN: ReadWrite<u32, TXFR_LEN::Register>, // 0x14, CB word 3
    STRIDE: ReadWrite<u32, STRIDE::Register>,     // 0x18, CB word 4
    NEXTCONBK: u32,                               // 0x1c, CB word 5
    DEBUG: ReadWrite<u32, DEBUG::Register>,       // 0x20, debug
    __reserved: [u32; 0x37],                      // padding~ 0x100
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
        PERMAP OFFSET(16) NUMBITS(5) [
            None = 0,
            DSI = 1,
            PCM_TX = 2,
            PCM_RX = 3,
            SMI = 4,
            PWM = 5,
            SPI_TX = 6,
            SPI_RX = 7,
            BSC_SPI_Slave_TX = 8,
            BSC_SPI_Slave_RX = 9,
            unused = 10,
            EMMC = 11,
            UART_TX = 12,
            SD_HOST = 13,
            UART_RX = 14,
            DSI_ = 15,
            SLIMBUS_MCTX = 16,
            HDMI = 17,
            SLIMBUS_MCRX = 18,
            SLIMBUS_DC0 = 19,
            SLIMBUS_DC1 = 20,
            SLIMBUS_DC2 = 21,
            SLIMBUS_DC3 = 22,
            SLIMBUS_DC4 = 23,
            Scaler_FIFO_0_SMI = 24,
            Scaler_FIFO_1_SMI = 25,
            Scaler_FIFO_2_SMI = 26,
            SLIMBUS_DC5 = 27,
            SLIMBUS_DC6 = 28,
            SLIMBUS_DC7 = 29,
            SLIMBUS_DC8 = 30,
            SLIMBUS_DC9 = 31
        ],
        /// number of bursts that DMA will try. 0 for single transfer.
        BURST_LENGTH OFFSET(12) NUMBITS(4) [
            Single = 0,
            Burst2 = 2,
            Burst4 = 4,
            Burst8 = 8,
            Burst16 = 16
        ],
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
    ],
    GLOBAL_INT[
        // 31-16: reserved.
        INT15 OFFSET(15) NUMBITS(1) [],
        INT14 OFFSET(14) NUMBITS(1) [],
        INT13 OFFSET(13) NUMBITS(1) [],
        INT12 OFFSET(12) NUMBITS(1) [],
        INT11 OFFSET(11) NUMBITS(1) [],
        INT10 OFFSET(10) NUMBITS(1) [],
        INT9 OFFSET(9) NUMBITS(1) [],
        INT8 OFFSET(8) NUMBITS(1) [],
        INT7 OFFSET(7) NUMBITS(1) [],
        INT6 OFFSET(6) NUMBITS(1) [],
        INT5 OFFSET(5) NUMBITS(1) [],
        INT4 OFFSET(4) NUMBITS(1) [],
        INT3 OFFSET(3) NUMBITS(1) [],
        INT2 OFFSET(2) NUMBITS(1) [],
        INT1 OFFSET(1) NUMBITS(1) [],
        INT0 OFFSET(0) NUMBITS(1) []
    ],
    GLOBAL_ENABLE[
        ENABLE15 OFFSET(15) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE14 OFFSET(14) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE13 OFFSET(13) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE12 OFFSET(12) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE11 OFFSET(11) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE10 OFFSET(10) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE9 OFFSET(9) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE8 OFFSET(8) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE7 OFFSET(7) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE6 OFFSET(6) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE5 OFFSET(5) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE4 OFFSET(4) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE3 OFFSET(3) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE2 OFFSET(2) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE1 OFFSET(1) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ],
        ENABLE0 OFFSET(0) NUMBITS(1) [
            Enable = 1,
            Disable = 0
        ]
    ]
}

/// Data structure used to order DMA settings/options.
/// Write data on DDR accordingly and tell it's address to DMA.
/// Values ordered by these members can be observed on register as comment follows.
#[repr(C, align(32))]
pub struct ControlBlock {
    pub transfer_information: u32,       // 0x00, accociated to TI register.
    pub source_address: u32,             // 0x04, SOURCE_AD
    pub destination_address: u32,        // 0x08, DEST_AD
    pub transfer_length: u32,            // 0x0C, TXFR_LEN
    pub two_d_mode_stride: u32,          // 0x10, STRIDE
    pub next_control_block_address: u32, // 0x14, NEXTCONBK
    __reserved: [u32; 2],                // N/A
}

impl ControlBlock {
    pub fn new(src: u32) -> ControlBlock {
        ControlBlock {
            transfer_information: (1 << 8) + (1 << 4),
            source_address: 0x100_0160,
            destination_address: 0x200_0000,
            transfer_length: 64,
            two_d_mode_stride: 0,
            next_control_block_address: 0,
            __reserved: [0; 2],
        }
    }
}

const DMAC_BASE: u32 = super::MMIO_BASE + 0x7200;

impl Deref for DMAC {
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
        // self.Channels.CS[0].write(CS::RESET::Reset);
        self.Channels[0].CS.write(CS::RESET::Reset);
    }

    pub fn turn_on(&self) {
        self.ENABLE.write(GLOBAL_ENABLE::ENABLE0::Enable);
    }

    pub fn exec(&self, cs: &ControlBlock) {
        let raw_addr: *const ControlBlock = cs;
        self.Channels[0].CONBLK_AD.set(raw_addr as u32);
        self.Channels[0].CS.write(CS::ACTIVE::Enable);
    }

    pub fn exec4(&self, cs: &ControlBlock4) {
        let raw_addr: *const ControlBlock4 = cs;
        self.Channels[0].CONBLK_AD.set(raw_addr as u32);
        self.Channels[0].CS.write(CS::ACTIVE::Enable);
    }

    fn __assert_size(&self) {
        unsafe {
            // compile time size assertion
            const _DMA_REG_SIZE: usize = 0x100; // in bytes.
            core::mem::transmute::<[u8; _DMA_REG_SIZE], DmaChannelRegister>([0; _DMA_REG_SIZE]);

            const _REG_SIZE: usize = 0xff4; // 0 - 0xff0
            core::mem::transmute::<[u8; _REG_SIZE], RegisterBlock>([0; _REG_SIZE]);
        }
    }
}

pub struct DMAC0 {}
impl DMAC0 {
    pub fn write_data() {
        unsafe {
            (*(0x3F00_7200 as *mut u32)) = 1; // CS
            (*(0x3F00_7204 as *mut u32)) = 1; // CONBLK_AD
                                              // ...
        }
    }
}

pub struct DMAC1 {}

#[allow(non_snake_case)]
pub struct DmacRegs {
    CS: u32, // 0x00, Status
    CONBLK_AD: u32, // 0x04, Control block address
             // ...
}

impl DMAC1 {
    pub fn write_data() {
        const BASE: u32 = 0x3F00_7200;
        let mut register: *mut DmacRegs = BASE as *mut DmacRegs;
        unsafe {
            (*register).CS = 1;
            (*register).CONBLK_AD = 2;
            // ...
        }
    }
}

pub struct DMAC2 {
    base_addr: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterDMAC2 {
    SOME_DATA: u32,
}

impl Deref for DMAC2 {
    type Target = RegisterDMAC2;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl DerefMut for DMAC2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr() }
    }
}

impl DMAC2 {
    fn ptr(&self) -> *mut RegisterDMAC2 {
        self.base_addr as *mut _
    }

    pub fn new() -> DMAC2 {
        let b: u32 = 0x3F00_7200;
        DMAC2 { base_addr: b }
    }

    pub fn write_data(&mut self) {
        self.SOME_DATA = 0x10;
        // ...
    }
}

// -----

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock3 {
    Channels: [DmaChannelRegister3; 15], // ch 0 - 15
    __reserved: [u32; 0x38],
    INT_STATUS: ReadWrite<u32, GLOBAL_INT::Register>, // 0xfe0
    __reserved1: [u32; 0x3],
    ENABLE: ReadWrite<u32, GLOBAL_ENABLE::Register>, // 0xff0
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct DmaChannelRegister3 {
    CS: ReadWrite<u32, CS::Register>,             // 0x00, Status
    CONBLK_AD: ReadWrite<u32>,                    // 0x04, Control block address
    TI: ReadWrite<u32, TI::Register>,             // 0x08, CB word 0
    SOURCE_AD: u32,                               // 0x0c, CB word 1
    DEST_AD: u32,                                 // 0x10, CB word 2
    TXFR_LEN: ReadWrite<u32, TXFR_LEN::Register>, // 0x14, CB word 3
    STRIDE: ReadWrite<u32, STRIDE::Register>,     // 0x18, CB word 4
    NEXTCONBK: u32,                               // 0x1c, CB word 5
    DEBUG: ReadWrite<u32, DEBUG::Register>,       // 0x20, debug
    __reserved: [u32; 0x37],                      // padding~ 0x100
}

impl Deref for DMAC3 {
    type Target = RegisterBlock3;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

pub struct DMAC3 {}

impl DMAC3 {
    pub fn new() -> DMAC3 {
        DMAC3 {}
    }

    fn ptr() -> *const RegisterBlock3 {
        const DMAC_BASE: u32 = 0x3F00_7200;
        DMAC_BASE as *const _
    }

    pub fn turn_on(&self) {
        self.ENABLE.modify(GLOBAL_ENABLE::ENABLE0::Enable);
    }
}

/// Data structure used to order DMA settings/options.
/// Write data on DDR accordingly and tell it's address to DMA.
/// Values ordered by these members can be observed on register as comment follows.
#[allow(non_snake_case)]
#[repr(C, align(32))]
pub struct ControlBlock4 {
    pub TI: InMemoryRegister<u32, TI::Register>, // 0x00, accociated to TI register.
    pub source_address: u32,                     // 0x04, SOURCE_AD
    pub destination_address: u32,                // 0x08, DEST_AD
    pub transfer_length: u32,                    // 0x0C, TXFR_LEN
    pub two_d_mode_stride: u32,                  // 0x10, STRIDE
    pub next_control_block_address: u32,         // 0x14, NEXTCONBK
    __reserved: [u32; 2],                        // N/A
}

impl ControlBlock4 {
    pub fn new(src: u32, dest: u32, length: u32, burst: u8) -> ControlBlock4 {
        let cb = ControlBlock4 {
            TI: InMemoryRegister::<u32, TI::Register>::new(0),
            source_address: src,
            destination_address: dest,
            transfer_length: length,
            two_d_mode_stride: 0,
            next_control_block_address: 0,
            __reserved: [0; 2],
        };

        match burst {
            2 => {
                cb.TI.modify(
                    TI::BURST_LENGTH::Burst2
                        + TI::SRC_WIDTH::Use128Bits
                        + TI::DEST_WIDTH::Use128Bits,
                );
            }
            4 => {
                cb.TI.modify(
                    TI::BURST_LENGTH::Burst4
                        + TI::SRC_WIDTH::Use128Bits
                        + TI::DEST_WIDTH::Use128Bits,
                );
            }
            8 => {
                cb.TI.modify(
                    TI::BURST_LENGTH::Burst8
                        + TI::SRC_WIDTH::Use128Bits
                        + TI::DEST_WIDTH::Use128Bits,
                );
            }
            16 => {
                cb.TI.modify(
                    TI::BURST_LENGTH::Burst16
                        + TI::SRC_WIDTH::Use128Bits
                        + TI::DEST_WIDTH::Use128Bits,
                );
            }
            _ => cb.TI.modify(TI::BURST_LENGTH::Single),
        };

        cb.TI.modify(TI::DEST_INC::Enabled + TI::SRC_INC::Enabled);
        cb
    }
}

impl Deref for DMAC4 {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

pub struct DMAC4 {}

impl DMAC4 {
    pub fn new() -> DMAC4 {
        DMAC4 {}
    }
    fn ptr() -> *const RegisterBlock {
        DMAC_BASE as *const _
    }

    pub fn init(&self) {
        self.ENABLE.write(
            GLOBAL_ENABLE::ENABLE0::Disable
                + GLOBAL_ENABLE::ENABLE1::Disable
                + GLOBAL_ENABLE::ENABLE2::Disable
                + GLOBAL_ENABLE::ENABLE3::Disable
                + GLOBAL_ENABLE::ENABLE4::Disable
                + GLOBAL_ENABLE::ENABLE5::Disable
                + GLOBAL_ENABLE::ENABLE6::Disable
                + GLOBAL_ENABLE::ENABLE7::Disable
                + GLOBAL_ENABLE::ENABLE8::Disable
                + GLOBAL_ENABLE::ENABLE9::Disable
                + GLOBAL_ENABLE::ENABLE10::Disable
                + GLOBAL_ENABLE::ENABLE11::Disable
                + GLOBAL_ENABLE::ENABLE12::Disable
                + GLOBAL_ENABLE::ENABLE13::Disable
                + GLOBAL_ENABLE::ENABLE14::Disable
                + GLOBAL_ENABLE::ENABLE15::Disable,
        );
    }

    pub fn turn_on(&self, ch: usize) {
        let en = match ch {
            0 => GLOBAL_ENABLE::ENABLE0::Enable,
            1 => GLOBAL_ENABLE::ENABLE1::Enable,
            2 => GLOBAL_ENABLE::ENABLE2::Enable,
            3 => GLOBAL_ENABLE::ENABLE3::Enable,
            4 => GLOBAL_ENABLE::ENABLE4::Enable,
            5 => GLOBAL_ENABLE::ENABLE5::Enable,
            6 => GLOBAL_ENABLE::ENABLE6::Enable,
            7 => GLOBAL_ENABLE::ENABLE7::Enable,
            8 => GLOBAL_ENABLE::ENABLE8::Enable,
            9 => GLOBAL_ENABLE::ENABLE9::Enable,
            10 => GLOBAL_ENABLE::ENABLE10::Enable,
            11 => GLOBAL_ENABLE::ENABLE11::Enable,
            12 => GLOBAL_ENABLE::ENABLE12::Enable,
            13 => GLOBAL_ENABLE::ENABLE13::Enable,
            14 => GLOBAL_ENABLE::ENABLE14::Enable,
            15 => GLOBAL_ENABLE::ENABLE15::Enable,
            _ => {
                return;
            }
        };
        self.ENABLE.modify(en);
    }

    pub fn turn_off(&self, ch: usize) {
        let disable = match ch {
            0 => GLOBAL_ENABLE::ENABLE0::Disable,
            1 => GLOBAL_ENABLE::ENABLE1::Disable,
            2 => GLOBAL_ENABLE::ENABLE2::Disable,
            3 => GLOBAL_ENABLE::ENABLE3::Disable,
            4 => GLOBAL_ENABLE::ENABLE4::Disable,
            5 => GLOBAL_ENABLE::ENABLE5::Disable,
            6 => GLOBAL_ENABLE::ENABLE6::Disable,
            7 => GLOBAL_ENABLE::ENABLE7::Disable,
            8 => GLOBAL_ENABLE::ENABLE8::Disable,
            9 => GLOBAL_ENABLE::ENABLE9::Disable,
            10 => GLOBAL_ENABLE::ENABLE10::Disable,
            11 => GLOBAL_ENABLE::ENABLE11::Disable,
            12 => GLOBAL_ENABLE::ENABLE12::Disable,
            13 => GLOBAL_ENABLE::ENABLE13::Disable,
            14 => GLOBAL_ENABLE::ENABLE14::Disable,
            15 => GLOBAL_ENABLE::ENABLE15::Disable,
            _ => {
                return;
            }
        };
        self.ENABLE.modify(disable);
    }

    pub fn exec(&self, ch: usize, cs: &ControlBlock4) {
        if ch > 15 {
            return;
        }
        compiler_fence(core::sync::atomic::Ordering::Release);
        let raw_addr: *const ControlBlock4 = cs;
        self.Channels[ch].CONBLK_AD.set(raw_addr as u32);
        self.Channels[ch].CS.write(CS::ACTIVE::Enable);
    }

    pub fn wait_end(&self, ch: usize) {
        if ch > 15 {
            return;
        }
        while self.Channels[ch].CS.read(CS::END) == 0 {
            // wait a while
        }
    }

    pub fn clear(&self, ch: usize) {
        if ch > 15 {
            return;
        }
        self.Channels[ch].CS.write(CS::END::Clear);
    }
}
