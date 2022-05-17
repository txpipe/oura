use std::{fmt::Debug, ops::Mul, time::Duration};

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Policy {
    pub max_retries: u32,
    #[serde(deserialize_with = "deserialize_duration")]
    pub backoff_unit: Duration,
    pub backoff_factor: u32,
    #[serde(deserialize_with = "deserialize_duration")]
    pub max_backoff: Duration,
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = u64::deserialize(deserializer)?;

    Ok(Duration::from_millis(millis))
}

const DEFAULT_MAX_RETRIES: u32 = 20;
const DEFAULT_BACKOFF_DELAY: u64 = 5_000;

impl Default for Policy {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            backoff_unit: Duration::from_millis(DEFAULT_BACKOFF_DELAY),
            backoff_factor: 2,
            max_backoff: Duration::from_millis(20 * DEFAULT_BACKOFF_DELAY),
        }
    }
}

fn compute_backoff_delay(policy: &Policy, retry: u32) -> Duration {
    let units = policy.backoff_factor.pow(retry);
    let backoff = policy.backoff_unit.mul(units);
    core::cmp::min(backoff, policy.max_backoff)
}

pub fn retry_operation<T, E>(op: impl Fn() -> Result<T, E>, policy: &Policy) -> Result<T, E>
where
    E: Debug,
{
    let mut retry = 0;

    loop {
        let result = op();

        match result {
            Ok(x) => break Ok(x),
            Err(err) if retry < policy.max_retries => {
                log::warn!("retryable operation error: {:?}", err);

                retry += 1;

                let backoff = compute_backoff_delay(policy, retry);

                log::debug!(
                    "backoff for {}s until next retry #{}",
                    backoff.as_secs(),
                    retry
                );

                std::thread::sleep(backoff);
            }
            Err(x) => {
                log::error!("max retries reached, failing whole operation");
                break Err(x);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn honors_max_retries() {
        let counter = Rc::new(RefCell::new(0));

        let inner_counter = counter.clone();
        let op = move || -> Result<(), String> {
            *inner_counter.borrow_mut() += 1;
            Err("very bad stuff happened".to_string())
        };

        let policy = Policy {
            max_retries: 3,
            backoff_unit: Duration::from_secs(1),
            backoff_factor: 0,
            max_backoff: Duration::from_secs(100),
        };

        assert!(retry_operation(op, &policy).is_err());

        assert_eq!(*counter.borrow(), 4);
    }

    #[test]
    fn honors_exponential_backoff() {
        let op = move || -> Result<(), String> { Err("very bad stuff happened".to_string()) };

        let policy = Policy {
            max_retries: 10,
            backoff_unit: Duration::from_millis(1),
            backoff_factor: 2,
            max_backoff: Duration::MAX,
        };

        let start = std::time::Instant::now();
        let result = retry_operation(op, &policy);
        let elapsed = start.elapsed();

        assert!(result.is_err());

        // not an exact science, should be 2046, adding +/- 10%
        assert!(elapsed.as_millis() >= 1842);
        assert!(elapsed.as_millis() <= 2250);
    }
}
