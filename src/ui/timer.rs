use iced::time::Duration;
use iced::time::Instant;

pub struct Timer {
    start: Instant,
    elapsed: Duration,
    total_duration: Duration,
}

impl Timer {
    pub fn new(start: Instant, duration: Duration) -> Self {
        Self {
            start,
            elapsed: Duration::ZERO,
            total_duration: duration,
        }
    }

    pub fn new_elapsed() -> Self {
        Self::new(Instant::now(), Duration::ZERO)
    }

    pub fn tick(&mut self, now: Instant) {
        self.elapsed = now.saturating_duration_since(self.start);
    }

    pub fn in_progress(&self) -> bool {
        self.elapsed < self.total_duration
    }

    pub fn remaining_duration(&self) -> Duration {
        self.total_duration.saturating_sub(self.elapsed)
    }

    pub fn elapsed_ratio(&self) -> f32 {
        if !self.in_progress() {
            1.0
        } else {
            1.0 - self
                .remaining_duration()
                .div_duration_f32(self.total_duration)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;
    use std::time::Duration;

    #[test]
    fn basic() {
        let start = super::Instant::now();
        let mut timer = Timer::new(start, Duration::from_secs(2));

        assert!(timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 0.0);

        timer.tick(start + Duration::from_secs(1));
        assert!(timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 0.5);

        timer.tick(start + Duration::from_secs(2));
        assert!(!timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 1.0);

        timer.tick(start + Duration::from_secs(5));
        assert!(!timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 1.0);
    }

    #[test]
    fn elapsed() {
        let mut timer = Timer::new_elapsed();
        assert!(!timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 1.0);

        timer.tick(super::Instant::now());
        assert!(!timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 1.0);
    }

    #[test]
    fn timer_from_the_future() {
        let start = super::Instant::now();
        let mut timer = Timer::new(start, Duration::from_secs(1));
        let earlier_time = start - Duration::from_secs(1);

        timer.tick(earlier_time);
        assert!(timer.in_progress());
        assert_eq!(timer.elapsed_ratio(), 0.0);
    }
}
