struct InterruptContext {
    id: u32,
}

trait InterruptDevice {
    fn on_fire(context: &InterruptContext, &mut self);
}