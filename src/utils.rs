#[macro_export]
macro_rules! static_init {
    ($T:ty, $e:expr) => {
        {
            use core::mem::MaybeUninit;
            // Statically allocate a read-write buffer for the value, write our
            // initial value into it (without dropping the initial zeros) and
            // return a reference to it.
            static mut BUF: MaybeUninit<$T> = MaybeUninit::uninit();
            $crate::static_init_half!(&mut BUF, $T, $e)
        };
    }
}

#[macro_export]
macro_rules! static_init_half {
    ($B:expr, $T:ty, $e:expr) => {
        {
            use core::mem::MaybeUninit;
            let buf: &'static mut MaybeUninit<$T> = $B;
            buf.as_mut_ptr().write($e);
            // TODO: use MaybeUninit::get_mut() once that is stabilized (see
            // https://github.com/rust-lang/rust/issues/63568).
            &mut *buf.as_mut_ptr() as &'static mut $T
        }
    };
}
