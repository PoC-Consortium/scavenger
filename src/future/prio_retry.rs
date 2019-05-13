//! Prio retry consumes a stream and yields elements with exponential backoffs.
//!
//! An element that is enqueued will be yielded instantly if it is a new element.
//! Otherwise it will be delayed according to the number of times that it has been enqueued
//! consecutively.
//! New items will replace old items and start with a delay of 0.

use futures::stream::{Fuse, Stream};
use futures::{Async, Future, Poll};
use std::time::{Duration, Instant};
use tokio::timer::{self, Delay};

struct DelayedItem<Item> {
    attempt: u32,
    delay: Option<Delay>,
    value: Item,
}

impl<Item> DelayedItem<Item> {
    fn new(i: Item) -> Self {
        Self {
            attempt: 0,
            delay: None,
            value: i,
        }
    }

    fn exp_backoff(&mut self, delay: Duration) {
        let backoff = 2u32.pow(self.attempt) * delay;
        self.delay = Some(Delay::new(Instant::now() + backoff));
        self.attempt += 1;
    }

    fn pause(&mut self) {
        self.delay = None;
    }
}

pub struct PrioRetry<S>
where
    S: Stream,
    S::Item: Ord + Clone + Eq,
{
    delay_duration: Duration,
    delayed_item: Option<DelayedItem<S::Item>>,
    stream: Fuse<S>,
}

impl<S> PrioRetry<S>
where
    S: Stream,
    S::Item: Ord + Clone + Eq,
{
    pub fn new(stream: S, delay_duration: Duration) -> Self {
        Self {
            delay_duration,
            delayed_item: None,
            stream: stream.fuse(),
        }
    }
}

/// Error returned by `PrioRetry`.
#[derive(Debug)]
pub struct Error<T>(Kind<T>);

/// PrioRetry error variants
#[derive(Debug)]
enum Kind<T> {
    /// Inner value returned an error
    Inner(T),

    /// Timer returned an error.
    Timer(timer::Error),
}

impl<S> Stream for PrioRetry<S>
where
    S: Stream,
    S::Item: Ord + Clone + Eq,
{
    type Item = S::Item;
    type Error = Error<S::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.stream.poll() {
                Ok(Async::NotReady) => {
                    break;
                }
                Ok(Async::Ready(Some(new_item))) => {
                    // check if we currently have a delay item
                    if let Some(ref mut delayed_item) = self.delayed_item {
                        if delayed_item.value < new_item {
                            // we have new item, this one will be yielded instantly
                            self.delayed_item = Some(DelayedItem::new(new_item.clone()));
                            return Ok(Async::Ready(Some(new_item)));
                        } else if delayed_item.value == new_item {
                            // if the current item was requeued, then we will yield it with a backoff
                            delayed_item.exp_backoff(self.delay_duration);
                        }
                    } else {
                        // we have new item, this one will be yielded instantly
                        self.delayed_item = Some(DelayedItem::new(new_item.clone()));
                        return Ok(Async::Ready(Some(new_item)));
                    }
                }
                Ok(Async::Ready(None)) => {
                    return Ok(Async::Ready(None));
                }
                Err(e) => {
                    return Err(Error(Kind::Inner(e)));
                }
            }
        }

        if let Some(ref mut delayed_item) = self.delayed_item {
            if let Some(ref mut delay) = delayed_item.delay {
                match delay.poll() {
                    Ok(Async::NotReady) => {}
                    Ok(Async::Ready(())) => {
                        // we yield a clone, since we need the old copy to check if an item was requeued
                        delayed_item.pause();
                        return Ok(Async::Ready(Some(delayed_item.value.clone())));
                    }
                    Err(e) => {
                        return Err(Error(Kind::Timer(e)));
                    }
                }
            }
        };

        Ok(Async::NotReady)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use tokio::timer::Interval;

    #[test]
    fn test_prio_retry() {
        let mut items = vec![0, 1, 2, 3, 3, 3, 0, 1, 2, 2, 6, 5, 7].into_iter();
        let len = items.len();
        let items = Interval::new(Instant::now(), Duration::from_millis(200))
            .take(len as u64)
            .map(move |_| items.next().unwrap())
            .map_err(|e| error!("can't consume interval: {:?}", e));
        let exp: Vec<i64> = vec![0, 1, 2, 3, 3, 3, 6, 7];
        let stream = PrioRetry::new(items, Duration::from_millis(100));
        let res = stream.collect();
        tokio::run(res.then(move |res| {
            match res {
                Err(_) => assert!(false),
                Ok(items) => assert_eq!(items, exp, "can't get expected items from prio retry"),
            };
            Ok(())
        }));
    }
}
