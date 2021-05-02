
use std::time::Instant;

pub struct GlobalTimer {
    last_update_time: Instant
}

impl GlobalTimer {
    pub fn new() -> GlobalTimer {
        GlobalTimer {
            last_update_time: Instant::now()
        }
    }

    pub fn pull_time_step_millis(&mut self) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        self.last_update_time = now;
        elapsed.as_millis() as u64
    }
}
