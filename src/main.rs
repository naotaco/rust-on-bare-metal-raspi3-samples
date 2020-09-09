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
#![feature(llvm_asm)]
#![feature(new_uninit)]
#![feature(const_fn)]

mod arm_debug;
mod arm_timer;
mod dmac;
mod exception;
mod gpio;
mod int_device;
mod interrupt;

#[cfg(feature = "interrupt_legacy")]
mod interrupt_legacy;

#[cfg(feature = "gic400v2")]
mod interrupt_gic;

mod mbox;
mod optional_cell;
mod timer;
mod uart;
mod utils;

use nt_allocator::NtGlobalAlloc;
extern crate alloc;

const MMIO_BASE: u32 = raspi_bsp::RasPiBsp::MMIO_BASE;

#[cfg(feature = "gic400v2")]
const MMIO_BASE2: u32 = raspi_bsp::RasPiBsp::MMIO_BASE2;

#[global_allocator]
static mut GLOBAL_ALLOCATOR: NtGlobalAlloc = NtGlobalAlloc {
    base: 0x600_0000,
    size: 0x200_0000,
};

#[allow(dead_code)]
fn init(data_addr: u32, size: usize, init_data: u32) {
    for i in 0..size / 4 {
        let p: *mut u32 = (data_addr + (i * 4) as u32) as *mut u32;
        unsafe {
            *p = init_data + i as u32;
        }
    }
}

#[allow(dead_code)]
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

#[cfg(feature = "interrupt_legacy")]
unsafe fn get_intc() -> &'static mut dyn crate::interrupt::Interrupt {
    let int = static_init!(
        crate::interrupt_legacy::InterruptLegacy,
        crate::interrupt_legacy::InterruptLegacy::new()
    );
    let intc: &'static mut dyn crate::interrupt::Interrupt = int;
    intc
}

#[cfg(feature = "gic400v2")]
unsafe fn get_intc() -> &'static mut dyn crate::interrupt::Interrupt {
    let gic = static_init!(crate::interrupt_gic::Gic, crate::interrupt_gic::Gic::new());
    gic.init();
    let intc: &'static mut dyn crate::interrupt::Interrupt = gic;
    intc
}

unsafe fn user_main() -> ! {
    arm_debug::setup_debug();

    //let uart = uart::Uart::new();
    let uart = static_init!(uart::Uart, uart::Uart::new());
    let mut mbox = mbox::Mbox::new();

    // set up serial console
    match uart.init(&mut mbox) {
        Ok(_) => uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            llvm_asm!("wfe" :::: "volatile"); // If UART fails, abort early
        },
    }

    GLOBAL_ALLOCATOR.init();

    let addr = exception::set_vbar_el1();
    uart.puts("set vbar");
    uart.hex((addr & 0xFFFF_FFFF) as u32);
    uart.puts("\n");

    // Section 2.4, 2.5
    let src = 0x200_0000;
    let dest = 0x800_0000;
    let size = 0x600_0000;

    uart.puts("Initializing GIC...\n");

    let intc = get_intc();

    intc.enable_int(int_device::Device::Timer1);
    intc.enable_int(int_device::Device::ArmTimer);
    intc.enable_int(int_device::Device::Dma);

    // PPIs are CPU private. Can't set target CPU.
    intc.set_target_cpu(int_device::Device::Timer1, 0);
    intc.set_target_cpu(int_device::Device::ArmTimer, 0);
    intc.set_target_cpu(int_device::Device::Dma, 0);

    intc.enable_distribution();

    // init(src, size, 0xFF00_0000);
    // init(dest, size, 0x1200_0000);

    // dump(src, size, &uart);
    // dump(dest, size, &uart);

    // create static instances of drivers.
    let timer_flags = static_init!(
        [optional_cell::OptionalCell<bool>; 4],
        [
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty()
        ]
    );

    let arm_timer_flag = static_init!(
        optional_cell::OptionalCell<bool>,
        optional_cell::OptionalCell::empty()
    );
    let dma_flags = static_init!(
        [optional_cell::OptionalCell<bool>; 16],
        [
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
            optional_cell::OptionalCell::empty(),
        ]
    );

    let timer = static_init!(timer::TIMER, timer::TIMER::new(timer_flags));
    let arm_timer = static_init!(
        arm_timer::ArmTimer,
        arm_timer::ArmTimer::new(arm_timer_flag)
    );
    let dma = static_init!(dmac::DMAC4, dmac::DMAC4::new(dma_flags));

    // setup irq handlers with drivers that have capability of irq handling.
    setup_irq_handlers(timer, arm_timer, dma, uart, intc);

    // enable interrupt handling at int controller.
    // let int = interrupt_legacy::interrupt_legacy::new();
    // int.enable_basic_irq(interrupt_legacy::BasicInterruptId::ARM_TIMER);
    // uart.puts("Enabling Irq1\n");
    // int.enable_irq(interrupt_legacy::InterruptId::TIMER1);
    // int.enable_irq(interrupt_legacy::InterruptId::DMA);
    uart.puts("going to enable CPU irq\n");
    // enable receiving irq at CPU
    armv8_boot::enable_irq();

    // timer
    let current = timer.get_counter32();
    let duration = 20_0000; // maybe 1sec.

    uart.puts("Starting timer\n");
    // timer.set(0, duration + current);
    timer.set(1, duration + current); // Ch1 is available on RasPi3.
                                      // timer.set(2, duration + current);
                                      // timer.set(3, duration + current); // it looks ch3 is available on RasPi4.

    // arm timer
    arm_timer.enable();
    arm_timer.start_free_run();
    arm_timer.enable_int();
    arm_timer.set_count_down(10000000);

    // dma
    let cb = dmac::ControlBlock4::new(src, dest, size as u32, 0);
    dma.turn_on(0);
    dma.exec(0, &cb);

    // main looooop
    loop {
        let mut context = MainTaskContext {
            timer_fired: false,
            arm_timer_fired: false,
            dma_fired: false,

            timer: &timer,
            arm_timer: &arm_timer,
            dma: &dma,
            uart: &uart,
        };

        {
            // critical section start:
            armv8_boot::disable_irq();

            context.timer_fired = timer.has_fired(1);
            context.arm_timer_fired = arm_timer.has_fired();
            context.dma_fired = dma.has_fired(0);

            // critical section end
            armv8_boot::enable_irq();
        }

        // perform main task once.
        main_task(context);

        // sleep until next event (e.g. interrupt)
        armv8_boot::wfe();
    }
}

#[allow(dead_code)]
struct MainTaskContext<'a> {
    timer_fired: bool,
    arm_timer_fired: bool,
    dma_fired: bool,

    timer: &'a timer::TIMER,
    arm_timer: &'a arm_timer::ArmTimer,
    dma: &'a dmac::DMAC4,
    uart: &'a uart::Uart,
}

fn main_task(context: MainTaskContext) {
    if context.timer_fired {
        context.uart.puts("[main] Timer fired ch1\n");
        let current = context.timer.get_counter32();
        let duration = 200_0000; // maybe 1sec.
        context.timer.set(1, duration + current);
    }
    if context.arm_timer_fired {
        context.uart.puts("[main] Arm timer fired\n");
    }
    if context.dma_fired {
        context.uart.puts("[main] DMA trans done. Kick again.\n");
        let src = 0x200_0000;
        let dest = 0x800_0000;
        let size = 0x600_0000;
        let cb = dmac::ControlBlock4::new(src, dest, size as u32, 0);
        context.dma.init();
        context.dma.turn_on(0);
        context.dma.exec(0, &cb);
    }
}

unsafe fn setup_irq_handlers(
    timer: &'static timer::TIMER,
    arm_timer: &'static arm_timer::ArmTimer,
    dma: &'static dmac::DMAC4,
    uart: &'static uart::Uart,
    intc: &'static dyn interrupt::Interrupt,
) {
    let timer_int_ids = static_init!(
        [int_device::Device; 4],
        [
            int_device::Device::Timer0,
            int_device::Device::Timer1,
            int_device::Device::Timer2,
            int_device::Device::Timer3,
        ]
    );
    let dma_int_ids = static_init!([int_device::Device; 1], [int_device::Device::Dma]);
    let arm_timer_int_ids = static_init!([int_device::Device; 1], [int_device::Device::ArmTimer]);
    let irq_devices = static_init!(
        [exception::IrqHandler; 3],
        [
            exception::IrqHandler::new(optional_cell::OptionalCell::new(timer), timer_int_ids),
            exception::IrqHandler::new(
                optional_cell::OptionalCell::new(arm_timer),
                arm_timer_int_ids
            ),
            exception::IrqHandler::new(optional_cell::OptionalCell::new(dma), dma_int_ids),
        ]
    );

    let handler_info = static_init!(
        exception::IrqHandlersSettings,
        exception::IrqHandlersSettings::new(irq_devices)
    );

    let register_result = exception::set_irq_handlers2(handler_info);

    let debug_context = static_init!(
        exception::DebugContext,
        exception::DebugContext::new(optional_cell::OptionalCell::new(uart))
    );

    let register_result_uart = exception::set_debug_context(debug_context);
    if register_result && register_result_uart {
        uart.puts("Successfully registerd handlers!\n");
    } else {
        uart.puts("Something wrong in handler registeration\n");
    }

    let register_intc = exception::set_int_controller(intc);
    if register_intc {
        uart.puts("Successfully registerd int controller!\n");
    } else {
        uart.puts("Something wrong in intc registeration\n");
    }
}
armv8_boot::entry!(kernel_entry);
