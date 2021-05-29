
pub mod null;
pub mod global;

pub trait Timer {
    fn pull_time_step_millis(&mut self) -> u64;
}
