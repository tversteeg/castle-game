/// Simple timer.
#[derive(Debug, Clone)]
pub struct Timer {
    /// How long to wait in seconds.
    interval: f64,
    /// How far the interval is.
    progress: f64,
}

impl Timer {
    /// Construct a new timer with a fixed interval in seconds.
    pub fn new(interval: f64) -> Self {
        let progress = 0.0;

        Self { interval, progress }
    }

    /// Trigger the timer and reset to the interval.
    pub fn trigger(&mut self) {
        self.progress = self.interval;
    }

    /// Update the timer by incrementing it with a delta time.
    ///
    /// Returns whether it triggered.
    pub fn update(&mut self, dt: f64) -> bool {
        self.progress += dt;
        if self.progress >= self.interval {
            self.progress = 0.0;

            true
        } else {
            false
        }
    }
}
