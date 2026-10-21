use std::{fmt::Debug, ops::Mul, time::{Duration, Instant}};

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Policy {
    pub max_retries: u32,
    #[serde(deserialize_with = "deserialize_duration")]
    pub backoff_unit: Duration,
    pub backoff_factor: u32,
    #[serde(deserialize_with = "deserialize_duration")]
    pub max_backoff: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub memory: Duration, // how long to remember a failure
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = u64::deserialize(deserializer)?;

    Ok(Duration::from_millis(millis))
}

const DEFAULT_BACKOFF_FACTOR: u32 = 2;
const DEFAULT_MAX_RETRIES: u32 = 20;
const DEFAULT_BACKOFF_DELAY: Duration = Duration::from_millis(5_000);

impl Default for Policy {
    fn default() -> Self {
        let backoff_unit = DEFAULT_BACKOFF_DELAY;

        // default memory should be greater than both
        // 1. max_backoff
        // 2. maximum duration based on backoff_unit, backoff_factor, and max_retries
        // in order that retry failure is not impossible, by default.

        // 1. max_backoff
        let default_max_backoff_ms = backoff_unit.checked_mul(20).unwrap_or(Duration::MAX);

        // 2. maximum duration based on backoff_unit, backoff_factor, and max_retries
        let max_retries = DEFAULT_MAX_RETRIES;
        let backoff_factor = DEFAULT_BACKOFF_FACTOR;

        // default memory
        let default_memory = max_cumulative_retry_duration(backoff_unit, backoff_factor, max_retries)
            .max(default_max_backoff_ms)
            .checked_add(backoff_unit) // a little bit longer than the max. possible
            .unwrap_or(Duration::MAX);

        Self {
            max_retries: max_retries,
            backoff_unit: backoff_unit ,
            backoff_factor: backoff_factor,
            max_backoff: default_max_backoff_ms,
            memory: default_memory,
        }
    }
}

fn compute_backoff_delay(policy: &Policy, retry: u32) -> Duration {
    let units = policy.backoff_factor.pow(retry);
    let backoff = policy.backoff_unit.mul(units);
    core::cmp::min(backoff, policy.max_backoff)
}

// Determine how much time will be spent only sleeping/waiting for max_retries (the worst case).
fn max_cumulative_retry_duration(backoff_unit: Duration, backoff_factor: u32, max_retries: u32) -> Duration {
        // https://www.wolframalpha.com/input?i=sum+of+a%5Ek+from+k%3D0+to+k%3Dj
        let (num, den) = if backoff_factor < 1 {
            let num = Some(1 - backoff_factor.pow(max_retries+1));
            let den = 1 - backoff_factor;
            (num, den)
        } else {
            let num = backoff_factor
                .checked_pow(max_retries + 1)
                .map(|x| x-1);

            let den = backoff_factor - 1;

            (num, den)
        };

        let max_retryable = match (num, den) {
            (Some(v), den) => v / den,
            (None, _) => u32::MAX,
        };

        backoff_unit.checked_mul(max_retryable).unwrap_or(Duration::MAX)
}

pub fn retry_operation<T, E>(op: impl Fn() -> Result<T, E>, policy: &Policy) -> Result<T, E>
where
    E: Debug,
{
    let mut retry = 0;
    let mut last: Option<Instant> = None;



    loop {
        let result = op();

        // reset the counter if the failure hasn't occurred for a while
        let now = std::time::Instant::now();
        if retry != 0 && now.duration_since(last.unwrap_or(now)) > policy.memory {
            retry = 0;
        }

        last = Some(std::time::Instant::now());

        match result {
            Ok(x) => break Ok(x),
            Err(err) if retry < policy.max_retries => {
                log::warn!("retryable operation error: {:?}", err);

                let backoff = compute_backoff_delay(policy, retry);

                retry += 1;

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
            memory: Duration::from_secs(5),
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
            memory: Duration::from_secs(5),
        };

        let start = std::time::Instant::now();
        let result = retry_operation(op, &policy);
        let elapsed = start.elapsed();

        assert!(result.is_err());

        // not an exact science, should be 1024, adding +/- 10%
        assert!(elapsed.as_millis() >= 1024*9/10);
        assert!(elapsed.as_millis() <= 1024*11/10);
    }

    #[test]
    fn honors_memory() {
        // For all cases the backoff factor is 2 and the backoff unit is 10ms. So, for each case
        // the delays will look like [10ms, 20ms, 40ms, 80ms, 160ms, ... ].
        struct Case {
            name: &'static str,
            num_failures: u32, // number of op() failures
            max_retries: u32, // retry policy
            memory: Duration, // forget prior failures after this duration
            expect_err: bool, // if retry should fail
            expect_runs: u32, // number of expected iterations
            expect_dur: Duration,
        }

        let cases: Vec<Case> = vec![
            Case {
                name: "max_fails occurs before memory is reached",
                num_failures: 6,
                max_retries: 5,
                memory: Duration::from_secs(std::u64::MAX),
                expect_err: true,
                expect_runs: 6,
                expect_dur: Duration::from_millis(10*((1<<5)-1)),
            }
            , Case {
                name: "forget the 1st 4 failures",
                num_failures: 5, // 10ms, 20ms, 40ms, 80ms, 160ms
                max_retries: 3,
                memory: Duration::from_millis(30),
                expect_err: false,
                expect_runs: 5,
                expect_dur: Duration::from_millis(10*((1<<3)-1) + 10 + 20),
            }
            , Case {
                name: "forget all failures",
                num_failures: 11,
                max_retries: 10,
                memory: Duration::from_millis(0),
                expect_err: false,
                expect_runs: 11,
                expect_dur: Duration::from_millis(10*11),
            }
        ];

        cases.iter().for_each(|x| {
            let start = Instant::now();
            let counter = Rc::new(RefCell::new(0));

            let failure_counter = counter.clone();
            let op = move || -> Result<(), String> {
                if *failure_counter.borrow() < x.num_failures {
                    *failure_counter.borrow_mut() += 1;
                    Err("very bad stuff happened".to_string())
                }
                else {
                    Ok(())
                }
            };

            let policy = Policy {
                max_retries: x.max_retries,
                backoff_unit: Duration::from_millis(10),
                backoff_factor: 2,
                max_backoff: Duration::from_millis(1024),
                memory: x.memory,
            };

            let failed = retry_operation(op, &policy).is_err();

            assert!(failed == x.expect_err,  "case '{}' failed in error check - {} vs. {}", x.name, x.expect_err, failed);
            assert_eq!(*counter.borrow(), x.expect_runs, "case '{}' failed in run count check - {} vs. {}", x.name, x.expect_runs, counter.borrow());

            let elapsed = start.elapsed();
            assert!(elapsed < x.expect_dur*11/10 && elapsed > x.expect_dur*9/10, "case '{}' failed in duration check - {} vs. {}", x.name, x.expect_dur.as_millis(), elapsed.as_millis());
        });

    }
}
