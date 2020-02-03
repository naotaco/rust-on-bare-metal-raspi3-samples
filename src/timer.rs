use register::{
    mmio::{ReadOnly, ReadWrite},
    register_bitfields,
};

const TIMER_BASE: u32 = super::MMIO_BASE + 0x3000;

type Callback = fn(time: u32);

pub struct TIMER {
    callback: Option<Callback>,
    _some_value: u32,
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
        match id {
            1 => self.clear_c1(),
            3 => self.clear_c3(),
            _ => {}
        }
    }
}

#[allow(dead_code)]
impl TIMER {
    pub fn new_with_callback(cb: Callback) -> TIMER {
        TIMER {
            callback: Some(cb),
            _some_value: 0xdeadbeef,
        }
    }

    pub fn new() -> TIMER {
        TIMER {
            callback: None,
            _some_value: 0xdeadbeef,
        }
    }

    fn tick(&self) {
        let value: u32 = 0;
        match self.callback {
            Some(c) => c(value),
            _ => {}
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

    pub fn set_c1(&self, t: u32) {
        self.C1.set(t);
    }

    pub fn set_c3(&self, t: u32) {
        self.C3.set(t);
    }

    pub fn is_match_c1(&self) -> bool {
        //self.CS.read(CS::M1) == CS::M1::Match
        self.CS.is_set(CS::M1)
        // let a: u32 = CS::M1::Match;
        // == CS::M1::Match;
    }

    pub fn is_match_c3(&self) -> bool {
        self.CS.is_set(CS::M3)
    }

    pub fn clear_c1(&self) {
        self.CS.write(CS::M1::Match);
    }

    pub fn clear_c3(&self) {
        self.CS.write(CS::M3::Match);
    }
}
