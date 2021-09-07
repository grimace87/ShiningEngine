
use std::time::Instant;

/// GlobalTimer struct
/// Global timer intended as a reference for all components of the engine to get time on the same
/// scale.
pub struct GlobalTimer {
    last_update_time: Instant
}

impl GlobalTimer {

    /// Construct a new instance initialised as having started at the present time
    pub fn new() -> GlobalTimer {
        GlobalTimer {
            last_update_time: Instant::now()
        }
    }
}

impl crate::timer::Timer for GlobalTimer {

    /// Produce the next time step as milliseconds elapsed since the last call
    fn pull_time_step_millis(&mut self) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        self.last_update_time = now;
        elapsed.as_millis() as u64
    }
}
