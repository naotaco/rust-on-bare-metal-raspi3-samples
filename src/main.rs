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
mod interrupt;
mod mbox;
mod timer;
mod uart;

fn init(data_addr: u32, size: usize, init_data: u32) {
    for i in 0..size / 4 {
        let p: *mut u32 = (data_addr + (i * 4) as u32) as *mut u32;
        unsafe {
            *p = init_data + i as u32;
        }
    }
}

fn dump(data_addr: u32, size: usize, uart: &uart::Uart) {
    if size <= 128 {
        for i in 0..size / 4 {
            if i % 4 == 0 {
                uart.hex(data_addr);
                uart.puts(": ");
            }
            let p: *mut u32 = (data_addr + (i * 4) as u32) as *mut u32;
            unsafe {
                uart.hex(*p);
            }
            if i % 4 == 3 {
                uart.puts("\n");
            }
        }
    } else {
        dump(data_addr, 64, &uart);
        uart.puts(".......\n");
        dump((data_addr + size as u32 - 64), 64, &uart);
        uart.puts("\n");
    }
}

fn memcpy_dmac(src: u32, dest: u32, size: usize) {
    let cb = dmac::ControlBlock4::new(src, dest, size as u32);
    let d4 = dmac::DMAC4::new();
    d4.init();
    d4.turn_on_ch0();
    d4.exec(&cb);
    d4.wait_end(0);
    d4.clear(0);
}

fn memcpy_cpu(src: u32, dest: u32, size: usize) {
    if src < dest {
        if src + size as u32 >= dest {
            return;
        }
    } else {
        if dest + size as u32 >= src {
            return;
        }
    }

    unsafe {
        core::intrinsics::copy_nonoverlapping(src as *mut u32, dest as *mut u32, size);
    }
}

fn print_time(uart: &uart::Uart) {
    let timer = timer::TIMER::new();
    uart.puts("time: ");
    uart.hex(timer.get_counter32());
    uart.puts("\n");
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

    let src = 0x200_0000;
    let dest = 0x400_0000;
    let size = 0x1_0000;
    print_time(&uart);
    uart.puts("Initializing...\n");
    init(src, size, 0xFF00_0000);
    init(dest, size, 0x1200_0000);

    print_time(&uart);
    uart.puts("Initializing.......done! \n");

    dump(src, size, &uart);
    dump(dest, size, &uart);

    // memcpy_dmac(src, dest, size);
    memcpy_cpu(src, dest, size);
    print_time(&uart);
    uart.puts("Done!\n");
    dump(dest, size, &uart);

    loop {}
}

raspi3_boot::entry!(kernel_entry);
