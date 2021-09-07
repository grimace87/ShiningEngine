
pub mod null;
pub mod global;

/// Timer trait
/// Producer of time steps. Each invocation of pull_time_step_millis should pull a time step
/// elapsed since the last invocation; this need not reflect a real-world sense of time though.
pub trait Timer {
    fn pull_time_step_millis(&mut self) -> u64;
}
