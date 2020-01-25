//! Interrupts

use core::cell::UnsafeCell;

/// A "mutex" based on critical sections
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> Mutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&self, _ctxt: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

/// Interrupt number
pub unsafe trait Nr {
    /// Returns the number associated with this interrupt
    fn nr(&self) -> u8;
}

// NOTE `Mutex` can be used as a channel so, the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. interrupt tokens) across
// different execution contexts (e.g. interrupts)
unsafe impl<T> Sync for Mutex<T> where T: Send {}

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

// https://gist.github.com/heechul/3018642

pub unsafe fn test() {
    asm!("ldxr x0, [x1]"); // load exclusive
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
