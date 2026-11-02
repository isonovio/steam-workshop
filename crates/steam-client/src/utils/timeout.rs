use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Timeout {
    start: Instant,
    duration: Duration,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }

    pub fn check(self) -> Option<Self> {
        (!self.remaining().is_zero()).then_some(self)
    }

    pub fn remaining(self) -> Duration {
        self.duration.saturating_sub(self.start.elapsed())
    }
}

#[cfg(test)]
#[path = "../tests/utils/timeout/tests.rs"]
mod tests;
