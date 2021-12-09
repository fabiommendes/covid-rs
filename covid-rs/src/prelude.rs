use serde::{Deserialize, Serialize};
// pub use crate::agent::Ag;
pub use crate::epidemic::*;
// pub use crate::pop_builder::PopBuilder;
// pub use crate::simulation::Simulation;
// pub use crate::reporter::{Report};
pub use crate::sampler::{
    AnySampler, ContactMatrixSampler, PopulationSampler, Sampler, SimpleSampler,
};

/// Basic representation of time. This crate usually assumes time is measured
/// in days.
pub type Time = u32;

/// Base Real type used by this crate. Uses an alias to easily change precision
/// if necessary.   
pub type Real = f64;
pub(crate) const INF: Real = Real::INFINITY;
pub(crate) const NAN: Real = Real::NAN;

/// Age of an agent.
pub type Age = u8;

/// An age distribution array in bins of 10 years.
pub type AgeDistribution10 = [Real; 9];
pub const AGE_DISTRIBUTION_UNIFORM: AgeDistribution10 = [1.0; 9];
pub const AGE_DISTRIBUTION_BRAZIL: AgeDistribution10 = [
    0.16119603,
    0.1589343,
    0.15688859,
    0.15378477,
    0.12989326,
    0.1076683,
    0.07449618,
    0.03880221,
    0.01539567 + 0.0029407,
];

/// Count population in each bin of 10 years.
pub type AgeCount10 = [u32; 9];

/// Simple trait to simplify the use of age-dependent values/parameters.
/// Basically, ForAge data is simply an encoding for a function like
/// fn(Age) -> Output;
pub trait ForAge {
    type Output;

    /// Return the content of parameter for agents with the given age.
    fn for_age(&self, age: Age) -> Self::Output;
}

impl<T: Clone> ForAge for AgeIndependent<T> {
    type Output = T;

    #[inline(always)]
    fn for_age(&self, _: Age) -> T {
        self.0.clone()
    }
}

impl<T, R> ForAge for T
where
    T: Fn(Age) -> R,
{
    type Output = R;

    fn for_age(&self, age: Age) -> R {
        return self(age);
    }
}

impl<T> ForAge for [T; 9]
where
    T: Copy,
{
    type Output = T;

    #[inline]
    fn for_age(&self, age: Age) -> T {
        let idx = (age / 10).min(8);
        self[idx as usize]
    }
}

/// A wrapper to declare independent values
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AgeIndependent<T>(T);

pub trait ToAgeIndependent
where
    Self: Sized,
{
    #[inline(always)]
    fn age_independent(self) -> AgeIndependent<Self> {
        AgeIndependent(self)
    }
}

impl<T: Sized> ToAgeIndependent for T {}

/// A simple enumeration that may contain a scalar param or an AgeDistribution10
/// value
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgeParam {
    Scalar(Real),
    Distribution(AgeDistribution10),
}

impl AgeParam {
    #[inline]
    pub fn map(&self, f: impl Fn(Real) -> Real) -> Self {
        match self {
            Self::Scalar(x) => Self::Scalar(f(*x)),
            Self::Distribution(xs) => Self::Distribution(xs.map(f)),
        }
    }
}

impl Default for AgeParam {
    fn default() -> Self {
        AgeParam::Scalar(0.)
    }
}

impl From<AgeDistribution10> for AgeParam {
    fn from(v: AgeDistribution10) -> Self {
        AgeParam::Distribution(v)
    }
}

impl From<Real> for AgeParam {
    fn from(v: Real) -> Self {
        AgeParam::Scalar(v)
    }
}

impl ForAge for AgeParam {
    type Output = Real;

    fn for_age(&self, age: Age) -> Real {
        match self {
            &AgeParam::Scalar(v) => v,
            &AgeParam::Distribution(ages) => ages.for_age(age),
        }
    }
}
