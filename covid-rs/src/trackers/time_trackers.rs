use getset::{CopyGetters, Getters, Setters};
use std::{
    fmt::Debug,
    thread::sleep,
    time::{Duration, Instant},
};
use crate::{
    prelude::Real,
    utils::{Accumulator, Sample},
};
use super::Tracker;

/// A simple throttle that limit the frequency to some specified amount.
#[derive(Getters, CopyGetters, Setters, Debug, Clone)]
pub struct Throttle {
    #[getset(get_copy = "pub")]
    instant: Instant,
    #[getset(get = "pub", set = "pub")]
    duration: Duration,
}

impl Throttle {
    pub fn new(duration: Duration) -> Self {
        Throttle {
            instant: Instant::now(),
            duration,
        }
    }

    pub fn new_sec(dt: Real) -> Self {
        let sec = dt as u64;
        let nano = (1e9 * (dt - sec as Real)) as u32;
        Self::new(Duration::new(sec, nano))
    }
}

impl<T> Tracker<T> for Throttle {
    fn track(&mut self, _: &T) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.instant);
        sleep(self.duration - elapsed)
    }
}

/// A simple clock that count the duration of each iteration (or group of n
/// iterations, if registered to execute every n steps).
#[derive(Getters, CopyGetters, Debug, Clone)]
pub struct Ticks<S: Sample + Clone> {
    #[getset(get_copy = "pub")]
    instant: Instant,
    #[getset(get = "pub")]
    sampler: S,
}

impl<S: Sample + Clone> Ticks<S> {
    pub fn new_from(sampler: S) -> Self {
        Ticks {
            instant: Instant::now(),
            sampler: sampler,
        }
    }
}

impl Ticks<Accumulator> {
    pub fn new() -> Self {
        Self::new_from(Accumulator::new())
    }
}

impl<S: Default + Sample + Clone> Default for Ticks<S> {
    fn default() -> Self {
        Ticks {
            instant: Instant::now(),
            sampler: S::default(),
        }
    }
}

impl<S: Sample + Clone + 'static, T> Tracker<T> for Ticks<S> {
    fn track(&mut self, _: &T) {
        let now = Instant::now();
        let delta = now.duration_since(self.instant);
        self.instant = now;
        self.sampler.observe(delta.as_secs_f64())
    }
}
