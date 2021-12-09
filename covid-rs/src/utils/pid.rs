use std::ops::{Add, Mul, Sub, Div};

use getset::{CopyGetters, Getters, Setters};

/// A simple implementation of a PID (Proportional, Integral, Derivative)
/// controller.
///
/// This controller is used to callibrate parameters during the simulation
/// initialization.
#[derive(Debug, Clone, Copy, Default, Getters, Setters, CopyGetters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct PID<N: Copy> {
    kp: N,
    ki: N,
    kd: N,
    error: N,
    acc: N,
}

impl<
        N: Add<Output = N>
            + Sub<Output = N>
            + Mul<Output = N>
            + Div<Output = N>
            + Default
            + Clone
            + Copy,
    > PID<N>
{
    /// Create a new controller from PID coefficients.
    pub fn new(kp: N, ki: N, kd: N) -> Self {
        let zero: N = Default::default();
        PID {
            kp,
            ki,
            kd,
            error: zero,
            acc: zero,
        }
    }

    /// Add measurement and return the corresponding feedback. This function
    /// updates the internal state tracking the error term and the cumulative
    /// error term.
    pub fn feedback(&mut self, error: N, dt: N) -> N {
        let diff = (error - self.error) / dt;
        self.error = error;
        let acc = self.acc + error * dt;
        self.acc = acc;

        return self.kp * error + self.kd * diff + self.ki * acc;
    }
}
