
use crate::timer::Timer;

pub struct NullTimer {}

impl NullTimer {
    pub fn new() -> NullTimer {
        NullTimer {}
    }
}

impl Timer for NullTimer {
    fn pull_time_step_millis(&mut self) -> u64 {
        0
    }
}
