use std::{time::{Instant, Duration}, thread::sleep};

pub struct Throttle {
    last_action: Instant,
    min_delay: Duration,
}

impl Throttle {
    pub fn new(min_delay: Duration) -> Throttle {
        Throttle { last_action: Instant::now(), min_delay }
    }

    pub fn wait_turn(&mut self) {
        let remaining = self.min_delay.as_millis() as i64 - self.last_action.elapsed().as_millis() as i64;
        
        if remaining > 0 {
            sleep(Duration::from_millis(remaining as u64));
        }

        self.last_action = Instant::now();
    }
}