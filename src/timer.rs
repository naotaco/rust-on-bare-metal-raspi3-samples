use crate::optional_cell::OptionalCell;
use register::{
    mmio::{ReadOnly, ReadWrite},
    register_bitfields,
};

const TIMER_BASE: u32 = super::MMIO_BASE + 0x3000;

type Callback = fn(time: u32, ch: u32);

pub struct TIMER {
    callback: Option<Callback>,
    fired: &'static [OptionalCell<bool>; 4],
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CS: ReadWrite<u32, CS::Register>,
    CLO: ReadOnly<u32, CLO::Register>,
    CLH: ReadOnly<u32, CLH::Register>,
    C0: ReadWrite<u32>,
    C1: ReadWrite<u32>,
    C2: ReadWrite<u32>,
    C3: ReadWrite<u32>,
}

register_bitfields! {
    u32,
    /// Control and status. common for 0-14 ch.
    CS [
        M3 OFFSET(3) NUMBITS(1)[
            Match=1,
            NoMatch=0
        ],
        M2 OFFSET(2) NUMBITS(1)[
            Match=1,
            NoMatch=0
        ],
        M1 OFFSET(1) NUMBITS(1)[
            Match=1,
            NoMatch=0
        ],
        M0 OFFSET(0) NUMBITS(1)[
            Match=1,
            NoMatch=0
        ]
    ],
    CLO [
        TIME OFFSET(0) NUMBITS(32) [ ]
    ],
    CLH [ TIME OFFSET(0) NUMBITS(32) [ ] ]
}

impl core::ops::Deref for TIMER {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl crate::exception::InterruptDevice for TIMER {
    fn on_fire(&self, id: u32) {
        for ch in 0..=3 {
            if self.is_match(ch) {
                self.clear(ch);
                self.fired[ch as usize].insert(Some(true));
            }
        }

        match self.callback {
            Some(c) => {
                c(0, id);
            }
            None => {}
        }
    }
}

#[allow(dead_code)]
impl TIMER {
    pub fn new(flags: &'static [OptionalCell<bool>; 4]) -> TIMER {
        TIMER {
            callback: None,
            fired: flags,
        }
    }

    fn ptr() -> *const RegisterBlock {
        TIMER_BASE as *const _
    }
    pub fn get_counter64(&self) -> u64 {
        let h: u64 = self.CLH.read(CLH::TIME) as u64;
        let l: u64 = self.CLO.read(CLO::TIME) as u64;
        (h << 32) + l
    }

    pub fn get_counter32(&self) -> u32 {
        self.CLO.read(CLO::TIME)
    }

    pub fn set(&self, ch: u32, t: u32) {
        let r = match ch {
            0 => &self.C0,
            1 => &self.C1,
            2 => &self.C2,
            3 => &self.C3,
            _ => return,
        };
        r.set(t);
    }

    pub fn is_match(&self, ch: u32) -> bool {
        let m = match ch {
            0 => CS::M0,
            1 => CS::M1,
            2 => CS::M2,
            3 => CS::M3,
            _ => return false,
        };
        self.CS.is_set(m)
    }

    fn clear(&self, ch: u32) {
        let m = match ch {
            0 => CS::M0::Match,
            1 => CS::M1::Match,
            2 => CS::M2::Match,
            3 => CS::M3::Match,
            _ => return,
        };
        self.CS.write(m);
    }

    pub fn has_fired(&self, ch: usize) -> bool {
        match ch {
            0 | 1 | 2 | 3 => match self.fired[ch].take() {
                Some(f) => f,
                None => false,
            },
            _ => false,
        }
    }
}
