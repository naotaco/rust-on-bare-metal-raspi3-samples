use crate::int_device;
use crate::interrupt::Interrupt;
use core::ops;
use register::{mmio::ReadWrite, register_bitfields};

const GIC_BASE: u32 = super::MMIO_BASE2 + 0x4_1000;
// const GICC_BASE: u32 = super::MMIO_BASE2 + 0x4_2000;
// const GICH_BASE: u32 = super::MMIO_BASE2 + 0x4_4000;
// const GICV_BASE: u32 = super::MMIO_BASE2 + 0x4_6000;

// dtsi: https://github.com/raspberrypi/linux/blob/rpi-4.19.y/arch/arm/boot/dts/bcm2838.dtsi
//ch

// TIMER:
// interrupts = <GIC_PPI 13 (GIC_CPU_MASK_SIMPLE(4) | IRQ_TYPE_LEVEL_LOW)>,
// <GIC_PPI 14 (GIC_CPU_MASK_SIMPLE(4) | IRQ_TYPE_LEVEL_LOW)>,
// <GIC_PPI 11 (GIC_CPU_MASK_SIMPLE(4) | IRQ_TYPE_LEVEL_LOW)>,
// <GIC_PPI 10 (GIC_CPU_MASK_SIMPLE(4) | IRQ_TYPE_LEVEL_LOW)>;

// DMA: interrupts = <GIC_SPI 80 IRQ_TYPE_LEVEL_HIGH>,

const _SGI_BASE: u32 = 0;
const PPI_BASE: u32 = 16;
const SPI_BASE: u32 = 32;

const ARMC_PERI_IRQS_BASE: u32 = 64;
const VIDEOCORE_IRQS_BASE: u32 = 96;

pub struct Id {}
#[allow(dead_code)]
impl Id {
    pub const TIMER0: u32 = PPI_BASE + 13;
    pub const TIMER1: u32 = PPI_BASE + 14;
    pub const TIMER2: u32 = PPI_BASE + 11;
    pub const TIMER3: u32 = PPI_BASE + 10;
    pub const LOCAL_TIMER: u32 = 53;
    pub const ARMC_TIMER: u32 = ARMC_PERI_IRQS_BASE; // ARMC int #0
    pub const VC_TIMER0: u32 = VIDEOCORE_IRQS_BASE + 0;
    pub const VC_TIMER1: u32 = VIDEOCORE_IRQS_BASE + 1;
    pub const VC_TIMER2: u32 = VIDEOCORE_IRQS_BASE + 2;
    pub const VC_TIMER3: u32 = VIDEOCORE_IRQS_BASE + 3;
    pub const DMA0: u32 = SPI_BASE + 80; // VideoCore interrupt #16
}

#[allow(non_snake_case)]
#[repr(C)]
// Represents GID Distributor registers
pub struct GICD {
    CTLR: ReadWrite<u32, GICD_CTLR::Register>,
    TYPER: ReadWrite<u32, GICD_TYPER::Register>, // 4
    _padding0: [u32; 0x3e],
    ISENABLER: [ReadWrite<u32>; 8], // from 0x100. Int Set-Enable
    _padding1: [u32; 0x18],
    ICENABLER: [ReadWrite<u32>; 8], // from 0x180. Int Clear-Enable
    _padding2: [u32; 0x98],
    IPRIORITYR: [ReadWrite<u32>; 8], // 0x400- Int Priority
    _padding3: [u32; 0xf8],
    ITARGETSR: [ReadWrite<u32>; 32], // 0x800- Int Target Processor
    _padding4: [u32; 0x1e0],
}

#[allow(non_snake_case)]
#[repr(C)]
// Represents GIC CPU interface registers
pub struct GICC {
    CTLR: ReadWrite<u32>,
    _dummy: [u32; 2],
    IAR: ReadWrite<u32, GICC_IAR::Register>, // 0xc- Int Acknowledge
    EOIR: ReadWrite<u32>,                    // 0x10- End of Int
}

fn _assert_struct_size() {
    unsafe {
        const _GICD_SIZE: usize = 0x1000;
        core::mem::transmute::<[u8; _GICD_SIZE], GICD>([0; _GICD_SIZE]);

        const _GICC_SIZE: usize = 0x14;
        core::mem::transmute::<[u8; _GICC_SIZE], GICC>([0; _GICC_SIZE]);
    }
}

// #[allow(non_snake_case)]
// #[repr(C)]
// pub struct GICH {}
// #[allow(non_snake_case)]
// #[repr(C)]
// pub struct GICV {}

register_bitfields! {
    u32,
    GICD_TYPER[
        ITLN OFFSET(0) NUMBITS(5)[] // Int Line Number.
    ],
    GICD_CTLR[
        ENABLE_GRP0 OFFSET(0) NUMBITS(1)[
            ENABLE=1,
            DISABLE=0
        ],
        ENABLE_GRP1NS OFFSET(1) NUMBITS(1)[
            ENABLE=1,
            DISABLE=0
        ],
        ENABLE_GRP1S OFFSET(2) NUMBITS(1)[
            ENABLE=1,
            DISABLE=0
        ]
    ],
    GICC_IAR[
        INT_ID OFFSET(0) NUMBITS(10)[],
        CPU_ID OFFSET(10) NUMBITS(3)[]
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    GICD: GICD,
    GICC: GICC,
}

pub struct Gic {
    supported_lines: u32,
}

impl ops::Deref for Gic {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Interrupt for Gic {
    fn enable_int(&mut self, dev: int_device::Device) -> bool {
        let id = Gic::to_id(dev);
        if id > 2u32.pow(self.supported_lines) {
            return false;
        }

        let orig = self.GICD.ISENABLER[(id / 32) as usize].get();
        self.GICD.ISENABLER[(id / 32) as usize].set(orig | 1 << (id % 32));

        true
    }

    fn set_target_cpu(&self, dev: int_device::Device, cpu: u32) {
        let id = Gic::to_id(dev);
        let c = 1 << cpu;
        self.GICD.ITARGETSR[(id / 4) as usize].set(c << ((id % 4) * 8));
    }

    fn enable_distribution(&self) {
        self.GICD
            .CTLR
            .modify(GICD_CTLR::ENABLE_GRP0::ENABLE + GICD_CTLR::ENABLE_GRP1NS::ENABLE);
    }

    fn get_first_pending_id(&self) -> Option<u32> {
        let id = self.GICC.IAR.read(GICC_IAR::INT_ID);
        if id > 2u32.pow(8) {
            return None;
        }

        Some(id)
    }

    fn get_first_pending_device(&self) -> Option<int_device::Device> {
        let id = self.get_first_pending_id();
        match id {
            Some(i) => Some(Gic::to_device(i)),
            None => None,
        }
    }

    fn end_interrupt_handling(&self, dev: int_device::Device) {
        let id = Gic::to_id(dev);
        self.GICC.EOIR.set(id);
    }

    fn device_to_id(&self, dev: int_device::Device) -> u32 {
        Gic::to_id(dev)
    }
}

impl Gic {
    pub fn new() -> Gic {
        Gic { supported_lines: 0 }
    }

    pub fn to_id(dev: int_device::Device) -> u32 {
        match dev {
            int_device::Device::Timer1 => Id::VC_TIMER1,
            int_device::Device::Timer3 => Id::VC_TIMER3,
            int_device::Device::ArmTimer => Id::ARMC_TIMER,
            int_device::Device::Dma => Id::DMA0,
            _ => 0,
        }
    }

    fn to_device(id: u32) -> int_device::Device {
        match id {
            Id::VC_TIMER1 => int_device::Device::Timer1,
            Id::VC_TIMER3 => int_device::Device::Timer3,
            Id::ARMC_TIMER => int_device::Device::ArmTimer,
            Id::DMA0 => int_device::Device::Dma,
            _ => int_device::Device::Invalid,
        }
    }

    pub fn init(&mut self) {
        self.supported_lines = self.get_supported_int_number();
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        GIC_BASE as *const _
    }

    pub fn get_supported_int_number(&self) -> u32 {
        self.GICD.TYPER.read(GICD_TYPER::ITLN)
    }
}
