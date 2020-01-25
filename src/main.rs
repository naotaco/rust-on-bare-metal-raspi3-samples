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

fn kernel_entry() {
    unsafe {
        exception::el2_to_el1_transition(user_main as *const () as u64);
    }
}

fn user_main() -> ! {
    arm_debug::setup_debug();
    let uart = uart::Uart::new();
    let mut mbox = mbox::Mbox::new();
    let gpio = gpio::GPIO::new();

    // set up serial console
    match uart.init(&mut mbox) {
        Ok(_) => uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            unsafe { asm!("wfe" :::: "volatile") }; // If UART fails, abort early
        },
    }

    unsafe {
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

    // アドレスを渡してControlBlockを初期化.
    let cb = dmac::ControlBlock4::new(src, dest, size as u32, 0);
    let d4 = dmac::DMAC4::new();
    d4.turn_on(0);
    // ControlBlockのアドレスを設定して実行
    d4.exec(0, &cb);

    dump(dest, size, &uart);

    // let a = 10 - 9 - 1;
    // let b = 11 / a;
    // uart.hex(b);

    unsafe {
        exception::SetIrqSourceToCore0();
        raspi3_boot::enable_irq();
    }

    let timer = timer::TIMER::new();
    let current = timer.get_counter32();
    let duration = 100_0000; // maybe 1sec.
    timer.set_c1(duration + current);
    uart.hex(current);
    uart.hex(duration);
    loop {
        if timer.is_match_c1() {
            uart.puts("Matched!");
            break;
        }
    }

    let arm_timer = arm_timer::ArmTimer::new();
    arm_timer.StartFreeRun();
    arm_timer.EnableInt();
    arm_timer.SetCountDown(1000000);

    loop {}
}

raspi3_boot::entry!(kernel_entry);
