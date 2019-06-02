/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 * Copyright (c) 2019 Nao Taco <naotaco@gmail.com>
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

#![no_std]
#![no_main]
#![feature(asm)]
#![feature(alloc)]

const MMIO_BASE: u32 = 0x3F00_0000;

mod arm_debug;
mod gpio;
mod mbox;
mod uart;

extern crate alloc;
extern crate nt_allocator;

use alloc::vec::Vec;
use nt_allocator::NtGlobalAlloc;

#[global_allocator]
static mut GLOBAL_ALLOCATOR: NtGlobalAlloc = NtGlobalAlloc {
    base: 0x0100_0000,
    size: 0x0400_0000,
};

fn alloc_test_u32(uart: &uart::Uart) {
    let max: u32 = 32;
    let mut v: Vec<u32> = Vec::new();
    let mut last_pointer = 0 as *const u32;

    for i in 0..max {
        let value = i as u32;

        v.push(value);

        if last_pointer != &v[0] {
            uart.puts("(Re)alloc detected!\nstart addr: ");
            let start: *const u32 = &v[0];
            uart.hex(start as u32);
            uart.puts("\n");
            uart.puts("len     : 0x");
            uart.hex(v.len() as u32);
            uart.puts("\ncapacity: 0x");
            uart.hex(v.capacity() as u32);
            uart.puts("\n");
            last_pointer = &v[0];
        }
    }

    for i in 0..max {
        uart.hex(i);
        uart.puts(": ");
        uart.hex(v[i as usize]);
        uart.puts("\n")
    }
}

fn alloc_test_f64(uart: &uart::Uart) {
    let max: u32 = 32;
    let mut v: Vec<f64> = Vec::new();
    let mut last_pointer = 0 as *const f64;

    for i in 0..max {
        let value = i as f64;

        v.push(value);

        if last_pointer != &v[0] {
            uart.puts("(Re)alloc detected!\nstart addr: ");
            let start: *const f64 = &v[0];
            uart.hex(start as u32);
            uart.puts("\n");
            uart.puts("len     : 0x");
            uart.hex(v.len() as u32);
            uart.puts("\ncapacity: 0x");
            uart.hex(v.capacity() as u32);
            uart.puts("\n");
            last_pointer = &v[0];
        }
    }

    for i in 0..max {
        uart.hex(i);
        uart.puts(": ");
        uart.hex(v[i as usize] as u32);
        uart.puts("\n")
    }
}

fn float_test(a: f64) -> f64 {
    let b = a * (0x23 as f64);
    b
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();
    arm_debug::setup_debug();

    unsafe {
        GLOBAL_ALLOCATOR.init();
    }

    // set up serial console
    match uart.init(&mut mbox) {
        Ok(_) => uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            unsafe { asm!("wfe" :::: "volatile") }; // If UART fails, abort early
        },
    }

    uart.puts("Greetings fellow Rustacean!\n");

    alloc_test_u32(&uart);
    // alloc_test_f64(&uart);

    uart.hex((float_test(0.1) * 100.1) as u32);

    loop {}
}

raspi3_boot::entry!(kernel_entry);
