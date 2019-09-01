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

const MMIO_BASE: u32 = 0x3F00_0000;

mod arm_debug;
mod dmac;
mod gpio;
// mod interrupt;
mod mbox;
mod uart;

fn init(data_addr: u32, size: usize) {
    for i in 0..size / 4 {
        let p: *mut u32 = (data_addr + (i * 4) as u32) as *mut u32;
        unsafe {
            *p = 0xFF00_0000 + i as u32;
        }
    }
}

fn dump(data_addr: u32, size: usize, uart: &uart::Uart) {
    for i in 0..size / 4 {
        let p: *mut u32 = (data_addr + (i * 4) as u32) as *mut u32;
        unsafe {
            uart.hex(*p);
        }
        if i % 4 == 3 {
            uart.puts("\n");
        }
    }
}

fn kernel_entry() -> ! {
    arm_debug::setup_debug();

    let uart = uart::Uart::new();
    let mut mbox = mbox::Mbox::new();
    // set up serial console
    match uart.init(&mut mbox) {
        Ok(_) => uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            unsafe { asm!("wfe" :::: "volatile") }; // If UART fails, abort early
        },
    }
    // sample 0
    // dmac::DMAC0::write_data();

    // sample1
    // dmac::DMAC1::write_data();

    // let mut dmac = dmac::DMAC2::new();
    // dmac.write_data();

    let src = 0x200_0000;
    let dest = 0x300_0000;
    let size = 64;
    init(src, size);
    uart.puts("src\n");
    dump(src, size, &uart);
    uart.puts("\ndest\n");
    dump(dest, size, &uart);
    uart.puts("kick DMA\n");
    let cb = dmac::ControlBlock3::new(src, dest, size as u32);

    let value = cb.get_ti();
    uart.puts("TI\n");
    uart.hex(value);

    let dmac = dmac::DMAC::new();
    dmac.turn_on();
    dmac.exec3(&cb);

    for i in 0..30 {
        uart.puts("dump\n");
        dump(dest, size, &uart);
    }

    loop {}
}

raspi3_boot::entry!(kernel_entry);
