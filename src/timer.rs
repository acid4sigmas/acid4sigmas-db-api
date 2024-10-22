use std::time::{Duration, Instant};

pub struct Timer {
    start_point: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_point: Instant::now(), // Start the timer at creation
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.start_point)
    }

    pub fn elapsed_as_millis(&self) -> u128 {
        self.elapsed().as_millis()
    }

    pub fn elapsed_as_micros(&self) -> u128 {
        self.elapsed().as_micros()
    }
}
