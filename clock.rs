pub const MAX_SECS: u32 = 359_999; // 99:59:59

pub trait Clock {
    fn start(&mut self);
    fn stop(&mut self);
    fn reset(&mut self);
    fn tick(&mut self);
    fn is_running(&self) -> bool;
    fn seconds(&self) -> u32; // shown on screen
    fn session_seconds(&self) -> u32; // what gets saved
    fn is_finished(&self) -> bool {
        false
    }
}

pub fn format_time(secs: u32) -> String {
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

#[derive(Default)]
pub struct CountdownTimer {
    remaining: u32,
    initial: u32,
    running: bool,
}

impl CountdownTimer {
    pub fn add(&mut self, secs: u32) {
        if !self.running {
            self.remaining = (self.remaining + secs).min(MAX_SECS);
        }
    }
}

impl Clock for CountdownTimer {
    fn start(&mut self) {
        if self.remaining > 0 {
            self.initial = self.remaining;
            self.running = true;
        }
    }
    fn stop(&mut self) {
        self.running = false;
    }
    fn reset(&mut self) {
        self.remaining = 0;
        self.initial = 0;
        self.running = false;
    }
    fn tick(&mut self) {
        if self.running && self.remaining > 0 {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.running = false;
            }
        }
    }
    fn is_running(&self) -> bool {
        self.running
    }
    fn seconds(&self) -> u32 {
        self.remaining
    }
    fn session_seconds(&self) -> u32 {
        self.initial
    }
    fn is_finished(&self) -> bool {
        self.initial > 0 && self.remaining == 0
    }
}

#[derive(Default)]
pub struct Stopwatch {
    elapsed: u32,
    running: bool,
}

impl Clock for Stopwatch {
    fn start(&mut self) {
        self.running = true;
    }
    fn stop(&mut self) {
        self.running = false;
    }
    fn reset(&mut self) {
        self.elapsed = 0;
        self.running = false;
    }
    fn tick(&mut self) {
        if self.running {
            self.elapsed = (self.elapsed + 1).min(MAX_SECS);
        }
    }
    fn is_running(&self) -> bool {
        self.running
    }
    fn seconds(&self) -> u32 {
        self.elapsed
    }
    fn session_seconds(&self) -> u32 {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_works() {
        assert_eq!(format_time(1500), "00:25:00");
        assert_eq!(format_time(3661), "01:01:01");
    }

    #[test]
    fn timer_add_is_capped() {
        let mut t = CountdownTimer::default();
        t.add(MAX_SECS);
        t.add(60);
        assert_eq!(t.seconds(), MAX_SECS);
    }

    #[test]
    fn timer_counts_down_and_finishes() {
        let mut t = CountdownTimer::default();
        t.add(2);
        t.start();
        t.tick();
        assert_eq!(t.seconds(), 1);
        t.tick();
        assert!(t.is_finished());
        assert!(!t.is_running());
        assert_eq!(t.session_seconds(), 2);
    }

    #[test]
    fn timer_does_not_start_at_zero() {
        let mut t = CountdownTimer::default();
        t.start();
        assert!(!t.is_running());
    }

    #[test]
    fn stopwatch_counts_up_only_when_running() {
        let mut s = Stopwatch::default();
        s.tick();
        assert_eq!(s.seconds(), 0);
        s.start();
        s.tick();
        s.tick();
        assert_eq!(s.seconds(), 2);
    }

    #[test]
    fn stopwatch_reset_clears_time() {
        let mut s = Stopwatch::default();
        s.start();
        s.tick();
        s.reset();
        assert_eq!(s.seconds(), 0);
        assert!(!s.is_running());
    }
}
