// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use crate::uart;
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

type Callback = fn(id: u32);

pub struct IrqHandlers {
    irq_callback: Callback,
    basic_irq_callback: Callback,
}

impl IrqHandlers {
    pub fn new(irq: Callback, basic_irq: Callback) -> IrqHandlers {
        IrqHandlers {
            irq_callback: irq,
            basic_irq_callback: basic_irq,
        }
    }
}

static mut HANDLERS: Option<IrqHandlers> = None;

//--------------------------------------------------------------------------------------------------
// Exception vector implementation
//--------------------------------------------------------------------------------------------------

/// Print verbose information about the exception and the panic.
fn default_exception_handler(e: &ExceptionContext) {
    let lr = e.lr;
    let uart = uart::Uart::new();
    uart.puts("At exception handler from 0x");
    uart.hex(lr as u32);
    uart.puts("\n");
}

/// Print verbose information about the exception and the panic.
fn irq_handler(e: &ExceptionContext) {
    let uart = uart::Uart::new();
    unsafe {
        uart.puts("IRQ handler from 0x");
        uart.hex(e.elr_el1 as u32);
        uart.puts("\n");

        let int = crate::interrupt::Interrupt::new();

        if int.is_any_irq_pending() {
            let pend = int.get_raw_pending();
            uart.puts("IRQ pending: ");
            uart.hex((pend & 0xFFFF_FFFF) as u32);
            uart.puts(" ");
            uart.hex(((pend >> 32) & 0xFFFF_FFFF) as u32);
            uart.puts("\n");
            for id in 0..63 {
                if (pend & (1 << id)) != 0 {
                    let hs: &IrqHandlers = &HANDLERS.as_ref().unwrap();
                    (hs.irq_callback)(id);
                }
            }
        } else {
            let pend = int.get_raw_basic_pending();
            if pend != 0 {
                uart.puts("Basic IRQ pending: ");
                uart.hex(pend);
                uart.puts("\n");
                for id in 0..7 {
                    if (pend & (1 << id)) != 0 {
                        let hs: &IrqHandlers = &HANDLERS.as_ref().unwrap();
                        (hs.basic_irq_callback)(id);
                    }
                }
            } else {
                uart.puts("Some unknown case...\n");
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

#[allow(dead_code)]
pub trait DaifField {
    fn daif_field() -> register::Field<u32, DAIF::Register>;
}

pub struct Debug;
pub struct SError;
pub struct IRQ;
pub struct FIQ;

#[allow(dead_code)]
impl DaifField for Debug {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::D
    }
}

#[allow(dead_code)]
impl DaifField for SError {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::A
    }
}

#[allow(dead_code)]
impl DaifField for IRQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::I
    }
}

#[allow(dead_code)]
impl DaifField for FIQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::F
    }
}

#[allow(dead_code)]
pub fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn el2_to_el1_transition(addr: u64) -> ! {
    // Enable timer counter registers for EL1.
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters.
    CNTVOFF_EL2.set(0);

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

// https://qiita.com/eggman/items/fd7b2907da71e65b8580

const GPU_INTERRUPTS_ROUTING: u32 = 0x4000000C;
const IRQ_ENABLE1: u32 = 0x3F00B210;

// const CORE0_INTERRUPT_SOURCE: u32 = 0x40000060;

pub unsafe fn set_irq_source_to_core0() {
    *(GPU_INTERRUPTS_ROUTING as *mut u32) = 0; // use core0
    *(IRQ_ENABLE1 as *mut u32) = 1 << 16;
}

pub fn set_irq_handlers(callbacks: IrqHandlers) {
    unsafe {
        HANDLERS = Some(callbacks);
    }
}
