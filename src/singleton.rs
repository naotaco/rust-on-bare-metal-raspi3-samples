#[macro_export]
macro_rules! singleton {
    (: $ty:ty = $expr:expr) => {
        $crate::interrupt::free(|_| unsafe {
            static mut USED: bool = false;
            static mut VAR: $ty = $expr;

            if USED {
                None
            } else {
                USED = true;
                let var: &'static mut _ = &mut VAR;
                Some(var)
            }
        })
    };
}
