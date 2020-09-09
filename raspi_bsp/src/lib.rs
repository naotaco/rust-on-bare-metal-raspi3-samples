#![no_std]

pub struct RasPiBsp {}

#[cfg(feature = "bcm2711")]
impl RasPiBsp {
    pub const MMIO_BASE: u32 = 0xfe00_0000; // RasPI common IPs (from BCM2835)
    pub const MMIO_BASE2: u32 = 0xff80_0000; // ARM LOCAL. Includes GIC, local timer
}

#[cfg(feature = "bcm2837")]
impl RasPiBsp {
    pub const MMIO_BASE: u32 = 0x3f00_0000; // RasPI common IPs (from BCM2835)
}

#[cfg(all(feature = "bcm2711", feature = "bcm2837"))]
compile_error!("Don't set more than one target SoC.");
