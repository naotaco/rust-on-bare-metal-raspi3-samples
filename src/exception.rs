// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use crate::optional_cell::OptionalCell;
use cortex_a::{asm, barrier, regs::*};
use register::mmio::ReadWrite;

// Assembly counterpart to this file.
global_asm!(include_str!("exception.S"));

/// Wrapper struct for memory copy of SPSR_EL1.
#[repr(transparent)]
struct SpsrEL1(ReadWrite<u32, SPSR_EL1::Register>);

/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
    // General Purpose Registers.
    gpr: [u64; 30],
    // The link register, aka x30.
    lr: u64,
    // Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,
    // Saved program status.
    spsr_el1: SpsrEL1,
}

pub trait InterruptDevice {
    fn on_fire(&self, id: u32);
}

pub struct IrqHandler {
    device: OptionalCell<&'static dyn InterruptDevice>,
    int_no: &'static [crate::int_device::Device],
}

impl IrqHandler {
    pub fn new(
        device: OptionalCell<&'static dyn InterruptDevice>,
        int_no: &'static [crate::int_device::Device],
    ) -> IrqHandler {
        IrqHandler { device, int_no }
    }
}

pub struct IrqHandlersSettings {
    pub devices: &'static [IrqHandler],
}

impl IrqHandlersSettings {
    pub fn new(devices: &'static [IrqHandler]) -> IrqHandlersSettings {
        IrqHandlersSettings { devices }
    }
}

static mut DEVICES: Option<&'static IrqHandlersSettings> = None;

pub trait ConsoleOut {
    fn puts(&self, s: &str);
    fn hex(&self, h: u32);
}

pub struct DebugContext {
    callback: OptionalCell<&'static dyn ConsoleOut>,
}

impl DebugContext {
    pub fn new(callback: OptionalCell<&'static dyn ConsoleOut>) -> DebugContext {
        DebugContext { callback }
    }
}

static mut DEBUG_CONTEXT: Option<&'static DebugContext> = None;

static mut INT_CONTROLLER: Option<&'static dyn crate::interrupt::Interrupt> = None;

#[allow(dead_code)]
unsafe fn puts(s: &str) {
    let s2 = ["[Exception] ", s].concat();
    DEBUG_CONTEXT.unwrap().callback.map(|c| c.puts(&s2));
}

#[allow(dead_code)]
unsafe fn puts2(s: &str) {
    let s2 = ["[Exception] ", s].concat();
    DEBUG_CONTEXT.unwrap().callback.map(|c| c.puts(&s2));
}

#[allow(dead_code)]
unsafe fn hex(v: u32) {
    DEBUG_CONTEXT.unwrap().callback.map(|c| c.hex(v));
}

#[allow(dead_code)]
unsafe fn hexln(v: u32) {
    DEBUG_CONTEXT.unwrap().callback.map(|c| c.hex(v));
    DEBUG_CONTEXT.unwrap().callback.map(|c| c.puts("\n"));
}

/// Print verbose information about the exception and the panic.
fn default_exception_handler(e: &ExceptionContext) {
    let lr = e.lr;
    unsafe {
        puts("At exception handler from 0x");
        hexln(lr as u32);
    }
}

/// Print verbose information about the exception and the panic.
fn irq_handler(e: &ExceptionContext) {
    unsafe {
        puts("IRQ handler from 0x");
        hexln(e.elr_el1 as u32);

        let intc = INT_CONTROLLER.unwrap();

        loop {
            let dev = intc.get_first_pending_device();
            match dev {
                Some(dev) => {
                    puts("Found pending id on GIC ");
                    let devs = DEVICES.unwrap().devices;
                    let id: u32 = 0;
                    let mut handled: bool = false;
                    for d in devs.iter() {
                        if d.int_no.contains(&dev) {
                            d.device.map(|d| d.on_fire(id));
                            handled = true;
                        }
                    }
                    if !handled {
                        puts("Unhandled exception ");
                        hexln(id);
                    }
                    intc.end_interrupt_handling(dev);
                }
                None => {
                    puts("No more pending id.\n");
                    break;
                }
            }
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Current, ELx
//--------------------------------------------------------------------------------------------------

/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    irq_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch64
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch32
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

/// Set the exception vector base address register.
///
/// # Safety
///
/// - The vector table and the symbol `__exception_vector_table_start` from the linker script must
///   adhere to the alignment and size constraints demanded by the AArch64 spec.
pub unsafe fn set_vbar_el1() -> u64 {
    // Provided by exception.S.
    extern "C" {
        static mut __exception_vector_start: u64;
    }
    let addr: u64 = &__exception_vector_start as *const _ as u64;

    VBAR_EL1.set(addr);

    // Force VBAR update to complete before next instruction.
    barrier::isb(barrier::SY);

    addr
}

#[inline(always)]
pub unsafe fn el2_to_el1_transition(addr: u64) -> ! {
    // Set EL1 execution state to AArch64.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    //
    // First, fake a saved program status, where all interrupts were masked and SP_EL1 was used as a
    // stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to init().
    ELR_EL2.set(addr);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
    SP_EL1.set(0x80000);

    // Use `eret` to "return" to EL1. This will result in execution of `reset()` in EL1.
    asm::eret()
}

pub unsafe fn set_irq_handlers2(h: &'static IrqHandlersSettings) -> bool {
    (*DEVICES.get_or_insert(h)) as *const _ == h
}

pub unsafe fn set_debug_context(c: &'static DebugContext) -> bool {
    (*DEBUG_CONTEXT.get_or_insert(c)) as *const _ == c
}

pub unsafe fn set_int_controller(c: &'static dyn crate::interrupt::Interrupt) -> bool {
    (*INT_CONTROLLER.get_or_insert(c)) as *const _ == c
}
