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
mod singleton;
mod timer;
mod uart;

extern crate alloc;
extern crate nt_allocator;
use alloc::vec::Vec;
use nt_allocator::NtGlobalAlloc;

#[global_allocator]
static mut GLOBAL_ALLOCATOR: NtGlobalAlloc = NtGlobalAlloc {
    base: 0x400_0000,
    size: 0x200_0000,
};

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
        dump(data_addr + size as u32 - 64, 64, &uart);
        uart.puts("\n");
    }
}

fn memcpy_dmac(src: u32, dest: u32, size: usize, burst: u8) {
    let cb = dmac::ControlBlock4::new(src, dest, size as u32, burst);
    let d4 = dmac::DMAC4::new();
    let ch: usize = 0;
    d4.init();
    d4.turn_on(ch);
    d4.exec(ch, &cb);
    d4.wait_end(ch);
    d4.clear(ch);
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

fn run_trans_test(
    gpio: &gpio::GPIO,
    uart: &uart::Uart,
    src: u32,
    dest: u32,
    size: usize,
    burst: u8,
    use_dma: bool,
) {
    let timer = timer::TIMER::new();
    let start = timer.get_counter64();
    gpio.pin5(true);
    //print_time(&uart);
    //uart.puts("starting memcpy.\n");

    if use_dma {
        memcpy_dmac(src, dest, size, burst);
    } else {
        memcpy_cpu(src, dest, size);
    }
    // memcpy_cpu(src, dest, size);

    gpio.pin5(false);
    let end = timer.get_counter64();

    uart.puts("done! size: 0x");
    uart.hex(size as u32);
    uart.puts(" burst: ");
    uart.hex(burst as u32);
    uart.puts(" duration: 0x");
    uart.hex(((end - start) & 0xFFFF_FFFF) as u32);
    uart.puts("\n");
}

fn vec_trans_test(uart: &uart::Uart) {
    let mut src: Vec<u32> = Vec::new();
    let mut dest: Vec<u32> = Vec::new();

    for i in 0..0x100 {
        src.push(i);
        dest.push(0);
    }

    dump(src.as_ptr() as u32, src.len(), &uart);
    dump(dest.as_ptr() as u32, dest.len(), &uart);

    let dmac = dmac::DMACWrapper::new(0, &src, &mut dest);
    dmac.exec();
    let dest2 = dmac.wait_end();
    dump(dest2.as_ptr() as u32, dest2.len(), &uart);
}

fn static_trans_test(uart: &uart::Uart) {
    uart.puts("Start\n");
    const BUF_LEN: usize = 0x100;
    static SRC: &'static [u32] = &[0xff; BUF_LEN];
    let dest: &'static mut [u32] = match singleton!(: [u32; BUF_LEN] = [0; BUF_LEN]) {
        Some(v) => v,   // 1st attempt
        None => return, // 2nd or later
    };

    dump(SRC.as_ptr() as u32, SRC.len() * 4, &uart);
    dump(dest.as_ptr() as u32, dest.len() * 4, &uart);

    let dmac = dmac::DMACWrapper2::new(0, SRC, dest);

    uart.puts("---\n");
    dmac.exec();

    let dest2 = dmac.wait_end();

    dump(dest2.as_ptr() as u32, dest2.len() * 4, &uart);
    uart.puts("done\n");
}

fn kernel_entry() -> ! {
    arm_debug::setup_debug();

    unsafe {
        GLOBAL_ALLOCATOR.init();
    }

    let uart = uart::Uart::new();
    let mut mbox = mbox::Mbox::new();

    // set up serial console
    match uart.init(&mut mbox) {
        Ok(_) => uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            unsafe { asm!("wfe" :::: "volatile") }; // If UART fails, abort early
        },
    }

    // Chapter 3.2
    vec_trans_test(&uart);
    static_trans_test(&uart);

    loop {}
}

raspi3_boot::entry!(kernel_entry);
