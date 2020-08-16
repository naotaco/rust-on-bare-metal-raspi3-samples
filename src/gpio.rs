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

    GPPUD[
        PUD OFFSET(0) NUMBITS(2)[
            Disabled = 0,
            EnablePullDown = 1,
            EnablePullUp = 2,
            Reserved = 3
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
    __reserved3: [u32; 0x1a],
    GPPUD: ReadWrite<u32, GPPUD::Register>,
    GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x98
}

impl core::ops::Deref for GPIO {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

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

    pub fn map_gpio_to_uart(&self) {
        self.GPFSEL1
            .modify(GPFSEL1::FSEL14::TXD0 + GPFSEL1::FSEL15::RXD0);
    }

    pub fn assert_uart_clock(&self) {
        self.GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
    }

    pub fn negate_all_clock(&self) {
        self.GPPUDCLK0.set(0);
    }

    pub fn disable_pull_up_down(&self) {
        self.GPPUD.write(GPPUD::PUD::Disabled);
    }
}
