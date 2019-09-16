/// Disables all interrupts
#[inline(always)]
pub fn disable() {
    unsafe {
        asm!("msr daifset, #2"
                 :
                 :
                 :
                 : "volatile");
    }
}

/// Enables all the interrupts
///
/// # Safety
///
/// - Do not call this function inside an `interrupt::free` critical section
#[inline(always)]
pub unsafe fn enable() {
    asm!("msr daifclr, #2" // to clear only "I" bit.
                 :
                 :
                 :
                 : "volatile");
}

/// Critical section context
///
/// Indicates that you are executing code within a critical section
pub struct CriticalSection {
    _0: (),
}

/// Execute closure `f` in an interrupt-free context.
///
/// This as also known as a "critical section".
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    // disable interrupts
    disable();

    let r = f(&CriticalSection { _0: () });

    // If the interrupts were active before our `disable` call, then re-enable
    // them. Otherwise, keep them disabled
    unsafe { enable() }

    r
}
