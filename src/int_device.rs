struct InterruptContext {
    id: u32,
}

trait InterruptionSource {
    fn on_interruption(context: &InterruptContext, &mut self);
}