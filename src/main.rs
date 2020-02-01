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
#![feature(global_asm)]
#![feature(new_uninit)]

const MMIO_BASE: u32 = 0x3F00_0000;

mod arm_debug;
mod arm_timer;
mod dmac;
mod exception;
mod gpio;
mod interrupt;
mod mbox;
mod timer;
mod uart;

use nt_allocator::NtGlobalAlloc;
extern crate alloc;

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

fn kernel_entry() {
    unsafe {
        exception::el2_to_el1_transition(user_main as *const () as u64);
    }
}

const DMA_CH0_CONT: u32 = 0x3F00_7000;

fn irq_callback(id: u32) {
    let uart = uart::Uart::new();
    let timer = timer::TIMER::new();
    uart.puts("IRQ callback!!!\n");

    match id {
        1 => {
            timer.clear_c1();
        }
        3 => {
            uart.puts("Clearing timer C3\n");
            timer.clear_c3();
        }
        crate::interrupt::Interrupt::INT_NO_DMA => {
            uart.puts("Clear DMA int.\n");
            unsafe {
                *(DMA_CH0_CONT as *mut u32) |= 0x1 << 2;
            }
        }
        _ => {
            uart.puts("Unknown int: ");
            uart.hex(id);
            uart.puts("\n");
        }
    }
}

fn basic_irq_callback(id: u32) {
    let uart = uart::Uart::new();
    uart.puts("IRQ callback!!!\n");

    match id {
        1 => {}
        _ => {}
    }
}

fn user_main() -> ! {
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

    unsafe {
        GLOBAL_ALLOCATOR.init();

        let addr = exception::set_vbar_el1();
        uart.puts("set vbar");
        uart.hex((addr & 0xFFFF_FFFF) as u32);
    }

    // Section 2.4, 2.5
    let src = 0x200_0000;
    let dest = 0x300_0000;
    let size = 64;

    uart.puts("Initializing...\n");

    init(src, size, 0xFF00_0000);
    init(dest, size, 0x1200_0000);

    dump(src, size, &uart);
    dump(dest, size, &uart);

    let handlers = exception::IrqHandlers::new(irq_callback, basic_irq_callback);
    exception::set_irq_handlers(handlers);

    unsafe {
        exception::set_irq_source_to_core0();
        raspi3_boot::enable_irq();
    }

    let int = interrupt::Interrupt::new();
    int.enable_basic_irq(interrupt::Interrupt::BASIC_INT_NO_ARM_TIMER);
    // uart.puts("Enabling Irq0\n");
    // int.EnableIrq(0);
    uart.puts("Enabling Irq1\n");
    int.enable_irq(1);

    let timer = timer::TIMER::new();
    let current = timer.get_counter32();
    let duration = 200_0000; // maybe 1sec.
    uart.puts("Starting timer\n");
    timer.set_c1(duration + current);

    let arm_timer = arm_timer::ArmTimer::new();
    arm_timer.start_free_run();
    arm_timer.enable_int();
    arm_timer.set_count_down(1000000);

    loop {}
}

raspi3_boot::entry!(kernel_entry);
