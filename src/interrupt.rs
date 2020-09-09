use crate::int_device;

pub trait Interrupt {
    fn enable_int(&mut self, dev: int_device::Device) -> bool;
    fn set_target_cpu(&self, dev: int_device::Device, cpu: u32);
    fn enable_distribution(&self);
    fn get_first_pending_id(&self) -> Option<u32>;
    fn get_first_pending_device(&self) -> Option<int_device::Device>;
    fn end_interrupt_handling(&self, dev: int_device::Device);

    // debug
    fn device_to_id(&self, dev: int_device::Device) -> u32;
}
