use crate::prelude::Real;
use getset::{CopyGetters, Setters};

/// A moving average tracker.
#[derive(Debug, Default, PartialEq, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct MaExp {
    mean: Real,
    prob: Real,
}

impl MaExp {
    /// Add n repeated observations of value x
    pub fn add_many(&mut self, x: Real, n: usize) {
        for _ in 0..n {
            self.add(x);
        }
    }

    /// Add single observation of value x
    pub fn add(&mut self, x: Real) {
        self.mean = self.mean * self.prob + (1. - self.prob) * x;
    }
}

/// A moving average tracker.
#[derive(Debug, Default, PartialEq, CopyGetters)]
pub struct Window {
    #[getset(get_copy = "pub")]
    window: usize,
    offset: usize,
    buffer: Vec<Real>,
}

impl Window {
    pub fn new(n: usize) -> Window {
        if n == 0 {
            panic!("Cannot create empty moving window");
        }
        Window {
            window: n,
            offset: 0,
            buffer: Vec::with_capacity(2 * n - 1),
        }
    }

    /// Add single observation of value x
    pub fn add(&mut self, x: Real) {
        if self.buffer.len() < self.window {
        } else if self.buffer.len() < self.buffer.capacity() {
            self.offset += 1;
        } else {
            for i in 0..self.window - 1 {
                self.buffer[i] = self.buffer[i + self.offset + 1];
            }
            self.buffer.truncate(self.window - 1);
            self.offset = 0;
        }
        self.buffer.push(x);
    }

    pub fn mean(&self) -> Real {
        let slice = &self.buffer[self.offset..self.buffer.len()];
        let tot: Real = slice.iter().cloned().sum();
        return tot / slice.len() as Real;
    }
}

mod test {
    #[test]
    fn window_simple() {
        let mut w = super::Window::new(3);
        w.add(10.0);
        assert_eq!(w.mean(), 10.0);
        w.add(5.0);
        assert_eq!(w.mean(), 7.5);
        w.add(3.0);
        assert_eq!(w.mean(), 6.0);
        w.add(1.0);
        assert_eq!(w.mean(), 3.0);
        w.add(2.0);
        assert_eq!(w.mean(), 2.0);
        w.add(3.0);
        assert_eq!(w.mean(), 2.0);
        w.add(4.0);
        assert_eq!(w.mean(), 3.0);
    }
}
