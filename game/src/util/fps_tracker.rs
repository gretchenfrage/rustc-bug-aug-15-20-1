//! Runtime tracking of FPS. 

use std::time::{Instant, Duration};

/// Runtime tracking of FPS. 
#[derive(Clone)]
pub struct FpsTracker {
    period: Duration,
    instants: Vec<Instant>,
}

impl FpsTracker {
    /// Create a new `FpsTracker` with a certain period to track over. 
    ///
    /// Also see `FpsTracker::default()` for a reasonable default period. 
    pub fn new(period: Duration) -> Self {
        FpsTracker {
            period,
            instants: Vec::new(),
        }
    }

    /// Register a frame happening right now. 
    pub fn log_frame(&mut self) {
        let now = Instant::now();
        for i in (0..self.instants.len()).rev() {
            if self.instants[i] < now - self.period {
                self.instants.swap_remove(i);
            }
        }
        self.instants.push(now);
    }

    /// Compute the current FPS. 
    pub fn get_fps(&mut self) -> u32 {
        let now = Instant::now();
        for i in (0..self.instants.len()).rev() {
            if self.instants[i] < now - self.period {
                self.instants.swap_remove(i);
            }
        }
        (self.instants.len() as f32 / self.period.as_secs_f32()) as u32
    }
}

impl Default for FpsTracker {
    fn default() -> Self {
        FpsTracker::new(Duration::from_secs(1))
    }
}