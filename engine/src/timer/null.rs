
/// NullTimer struct
/// Timer which does nothing, always returning zero elapsed time
pub struct NullTimer {}

impl Default for NullTimer {

    /// Construct a new instance; currently an empty struct
    fn default() -> NullTimer {
        NullTimer {}
    }
}

impl crate::timer::Timer for NullTimer {

    /// Return zero elapsed time
    fn pull_time_step_millis(&mut self) -> u64 {
        0
    }
}
