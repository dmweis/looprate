pub mod log_timer;
pub mod shared_timers;

use std::thread::sleep;
use std::time::{ Duration, Instant };

pub struct RateTimer {
    last: Instant,
}

impl RateTimer {
    pub fn new() -> RateTimer {
        RateTimer {
            last: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.last = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        self.last.elapsed()
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.last.elapsed().as_millis() as u32
    }

    pub fn elapsed_hz(&self) -> f32 {
        1000.0 / self.last.elapsed().as_millis() as f32
    }
}

pub struct Rate {
    time_step: Duration,
    next: Instant,
}

impl Rate {
    pub fn from_time(time_step: Duration) -> Rate {
        Rate {
            time_step,
            next: Instant::now() + time_step,
        }
    }

    pub fn from_frequency(frequency: f32) -> Rate {
        let time_step = Duration::from_secs_f32(1.0 / frequency);
        Rate {
            time_step,
            next: Instant::now() + time_step,
        }
    }

    pub fn wait(&mut self) {
        if let Some(sleep_duration) = self.next.checked_duration_since(Instant::now()) {
            sleep(sleep_duration)
        }
        self.next = Instant::now() + self.time_step;
    }

    pub fn check(&mut self) -> bool {
        if let Some(_) = self.next.checked_duration_since(Instant::now()) {
            return false;
        } else {
            self.next = Instant::now() + self.time_step;
            return true;
        }
    }
}
