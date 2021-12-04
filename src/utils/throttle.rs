use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct Throttle {
    last_action: Instant,
    min_delay: Duration,
}

impl Throttle {
    pub fn new(min_delay: Duration) -> Throttle {
        Throttle {
            last_action: Instant::now(),
            min_delay,
        }
    }

    pub fn wait_turn(&mut self) {
        let remaining = self.min_delay.checked_sub(self.last_action.elapsed());

        if let Some(remaining) = remaining {
            sleep(remaining);
        }

        self.last_action = Instant::now();
    }
}
