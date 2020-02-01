/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::MMIO_BASE;
use register::{mmio::ReadWrite, mmio::WriteOnly, register_bitfields};

// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,
    GPFSEL0[
        FSEL5 OFFSET(15) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001
        ],
        FSEL6 OFFSET(18) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001
        ]
    ],
    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            RXD0 = 0b100, // UART0     - Alternate function 0
            RXD1 = 0b010  // Mini UART - Alternate function 5

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD0 = 0b100, // UART0     - Alternate function 0
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],

    GPSET0[
        SET5 OFFSET(5) NUMBITS(1)[
            Assert = 1 // write 1 to set.
        ],
        SET6 OFFSET(5) NUMBITS(1)[
            Assert = 1
        ]
    ],

    GPCLR0[
        CLR5 OFFSET(5) NUMBITS(1)[
            Negate = 1 // write 1 to clear.
        ],
        CLR6 OFFSET(5) NUMBITS(1)[
            Negate = 1 // write 1 to clear
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

const GPIO_BASE: u32 = super::MMIO_BASE + 0x20_0000;

pub struct GPIO {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    GPFSEL0: ReadWrite<u32, GPFSEL0::Register>, // 0
    GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 4
    __reserved: [u32; 0x5],
    GPSET0: WriteOnly<u32, GPSET0::Register>, // 1c
    __reserved2: [u32; 0x2],
    GPCLR0: WriteOnly<u32, GPCLR0::Register>, // 0x28
}

pub const GPFSEL1: *const ReadWrite<u32, GPFSEL1::Register> =
    (MMIO_BASE + 0x0020_0004) as *const ReadWrite<u32, GPFSEL1::Register>;

pub const GPPUD: *const ReadWrite<u32> = (MMIO_BASE + 0x0020_0094) as *const ReadWrite<u32>;

pub const GPPUDCLK0: *const ReadWrite<u32, GPPUDCLK0::Register> =
    (MMIO_BASE + 0x0020_0098) as *const ReadWrite<u32, GPPUDCLK0::Register>;

impl core::ops::Deref for GPIO {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

#[allow(dead_code)]
impl GPIO {
    pub fn new() -> GPIO {
        let g = GPIO {};
        g.init();
        g
    }

    fn ptr() -> *const RegisterBlock {
        GPIO_BASE as *const _
    }

    fn init(&self) {
        self.GPFSEL0
            .modify(GPFSEL0::FSEL5::Output + GPFSEL0::FSEL6::Output);
        self.GPCLR0.write(GPCLR0::CLR5::Negate);
    }

    pub fn pin5(&self, set: bool) {
        if set {
            self.GPSET0.write(GPSET0::SET5::Assert);
        } else {
            self.GPCLR0.write(GPCLR0::CLR5::Negate);
        }
    }
}
