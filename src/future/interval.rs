//! This is almost the exact code of the tokio Interval implementation with a simple difference:
//!
//! In scavenger we request the `MiningInfo` in a certain interval. If the pool or wallet is not
//! available in a certain time frame the request will take longer than `Interval`. As soon as the
//! pool or wallet is available a bunch of requests are going to be fired at once.
//!
//! Here we delay once an item has been processed, e.g.:
//! 1. We have a interval of 3s.
//! 2. We fire our request at time = 0s.
//! 3. We timeout after time = 10s
//! 4. We fire our next request at time t = 13s

use futures::{try_ready, Future, Poll, Stream};
use std::time::{Duration, Instant};
use tokio::clock;
use tokio::timer::Delay;
use tokio::timer::Error;

/// A stream representing notifications at fixed interval
#[derive(Debug)]
pub struct Interval {
    /// Future that completes the next time the `Interval` yields a value.
    delay: Delay,

    /// The duration between values yielded by `Interval`.
    duration: Duration,
}

impl Interval {
    /// Create a new `Interval` that starts at `at` and yields every `duration`
    /// interval after that.
    ///
    /// Note that when it starts, it produces item too.
    ///
    /// The `duration` argument must be a non-zero duration.
    ///
    /// # Panics
    ///
    /// This function panics if `duration` is zero.
    pub fn new(at: Instant, duration: Duration) -> Interval {
        assert!(
            duration > Duration::new(0, 0),
            "`duration` must be non-zero."
        );

        Interval::new_with_delay(Delay::new(at), duration)
    }

    /// Creates new `Interval` that yields with interval of `duration`.
    ///
    /// The function is shortcut for `Interval::new(Instant::now() + duration, duration)`.
    ///
    /// The `duration` argument must be a non-zero duration.
    ///
    /// # Panics
    ///
    /// This function panics if `duration` is zero.
    pub fn new_interval(duration: Duration) -> Interval {
        Interval::new(clock::now(), duration)
    }

    pub(crate) fn new_with_delay(delay: Delay, duration: Duration) -> Interval {
        Interval { delay, duration }
    }
}

impl Stream for Interval {
    type Item = Instant;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Wait for the delay to be done
        let _ = try_ready!(self.delay.poll());

        self.delay.reset(Instant::now() + self.duration);

        // Get the `now` by looking at the `delay` deadline
        let now = self.delay.deadline();

        // Return the current instant
        Ok(Some(now).into())
    }
}
