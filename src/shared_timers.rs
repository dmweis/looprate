use std::time::{ Duration, Instant };
use std::collections::HashMap;
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::{ Arc, Weak, Mutex };


pub struct SharedTimer {
    last: Instant,
    shared_elapsed: Arc<AtomicU32>,
}

impl SharedTimer {
    fn new(shared_elapsed: Arc<AtomicU32>) -> SharedTimer {
        SharedTimer {
            last: Instant::now(),
            shared_elapsed,
        }
    }

    pub fn tick(&mut self) {
        self.shared_elapsed.store(self.last.elapsed().as_millis() as u32, Ordering::Release);
        self.last = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(self.shared_elapsed.load(Ordering::Acquire) as u64)
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.shared_elapsed.load(Ordering::Acquire)
    }

    pub fn elapsed_hz(&self) -> f32 {
        1000.0 / self.shared_elapsed.load(Ordering::Acquire) as f32
    }
}

#[derive(Clone)]
pub struct SharedTimerFactory {
    counters: Arc<Mutex<HashMap<String, Weak<AtomicU32>>>>,
}

impl SharedTimerFactory {
    pub fn new() -> SharedTimerFactory {
        SharedTimerFactory {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_timer(&mut self, name: String) -> SharedTimer {
        let shared_elapsed = Arc::new(AtomicU32::new(0));
        let weak = Arc::downgrade(&shared_elapsed);
        self.counters.lock().unwrap().insert(name, weak);
        SharedTimer::new(shared_elapsed)
    }

    pub fn get_elapsed(&mut self, name: &str) -> Option<Duration> {
        let mut counters = self.counters.lock().expect("Mutex poisoned");
        if let Some(weak_elapsed) = counters.get(name) {
            if let Some(elapsed) = weak_elapsed.upgrade() {
                Some(Duration::from_millis(elapsed.load(Ordering::Acquire) as u64))
            } else {
                // weak reference is dead so remove it
                counters.remove(name);
                None
            }
        } else {
            None
        }
    }

    pub fn get_all_elapsed(&mut self) -> Vec<(String, Duration)> {
        let mut counters = self.counters.lock().expect("Mutex poisoned");
        let mut result = Vec::with_capacity(counters.len());
        let mut to_remove = vec![];
        for (name, weak_elapsed) in counters.iter() {
            if let Some(elapsed) = weak_elapsed.upgrade() {
                let elapsed = Duration::from_millis(elapsed.load(Ordering::Acquire) as u64);
                result.push((name.to_owned(), elapsed));
            } else {
                to_remove.push(name.to_owned());
            }
        }
        for name in to_remove {
            counters.remove(&name);
        }
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_can_send() {
        let mut timer_factory = SharedTimerFactory::new();
        let timer_name = "timer_1";
        let mut timer = timer_factory.get_timer(timer_name.to_owned());

        std::thread::spawn(move || {
            timer.tick();
        });
        let _ = timer_factory.get_elapsed(timer_name);
    }

    #[test]
    fn cloned_factory_still_shares() {
        let timer_1_name = "timer_1";
        let timer_2_name = "timer_2";
        let mut timer_factory = SharedTimerFactory::new();
        let mut local_timer = timer_factory.get_timer(timer_1_name.to_owned());
        local_timer.tick();
        let mut factory_clone = timer_factory.clone();
        let returned_timer = std::thread::spawn(move || {
            let mut other_timer = factory_clone.get_timer(timer_2_name.to_owned());
            other_timer.tick();
            // send timer back to prevent arc from going out of scope
            other_timer
        }).join().unwrap();
        assert!(timer_factory.get_elapsed(timer_1_name).is_some());
        assert!(timer_factory.get_elapsed(timer_2_name).is_some());
        let _ = returned_timer.elapsed();
    }

    #[test]
    fn timer_gets_dropped() {
        let timer_1_name = "timer_1";
        let timer_2_name = "timer_2";
        let mut timer_factory = SharedTimerFactory::new();
        let mut local_timer = timer_factory.get_timer(timer_1_name.to_owned());
        local_timer.tick();
        let mut factory_clone = timer_factory.clone();
        std::thread::spawn(move || {
            let mut other_timer = factory_clone.get_timer(timer_2_name.to_owned());
            other_timer.tick();
        }).join().unwrap();
        assert!(timer_factory.get_elapsed(timer_1_name).is_some());
        assert!(timer_factory.get_elapsed(timer_2_name).is_none());
    }
}