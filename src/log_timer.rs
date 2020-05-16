use std::time::{ Duration, Instant };
use log::*;


pub struct TraceTimer {
    name: & 'static str,
    last: Instant,
    last_log: Instant,
    ticks_since_log: usize,
    duration_since_log: Duration,
}

impl TraceTimer {
    pub fn new(name: & 'static str) -> TraceTimer {
        TraceTimer {
            name,
            last: Instant::now(),
            last_log: Instant::now(),
            ticks_since_log: 0,
            duration_since_log: Duration::from_secs(0),
        }
    }

    pub fn tick(&mut self) {
        if self.last_log.elapsed() > Duration::from_secs(1) {
            self.last_log = Instant::now();
            let mean_seconds = self.duration_since_log.as_secs_f32() / self.ticks_since_log as f32;
            let mean_duration = Duration::from_secs_f32(mean_seconds);
            trace!("{} at {}ms", self.name, mean_duration.as_millis());
            self.ticks_since_log = 0;
            self.duration_since_log = Duration::from_secs(0);
        }
        self.duration_since_log += self.last.elapsed();
        self.ticks_since_log += 1;
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
