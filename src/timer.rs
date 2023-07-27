/// Simple timer.
#[derive(Debug, Clone)]
pub struct Timer {
    /// How long to wait in seconds.
    interval: f32,
    /// How far the interval is.
    progress: f32,
}

impl Timer {
    /// Construct a new timer with a fixed interval in seconds.
    pub fn new(interval: f32) -> Self {
        let progress = 0.0;

        Self { interval, progress }
    }

    /// Update the timer by incrementing it with a delta time.
    ///
    /// Returns whether it triggered.
    pub fn update(&mut self, dt: f32) -> bool {
        self.progress += dt;
        if self.progress >= self.interval {
            self.progress = 0.0;

            true
        } else {
            false
        }
    }
}
