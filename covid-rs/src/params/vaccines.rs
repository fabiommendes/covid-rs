use crate::prelude::{Age, AgeParam, ForAge, Real, Time};
use lazy_static::lazy_static;

/// Enumeration of applied Covid19 vaccines.
///
/// All vaccines have a required time value describing the time (in days) to vaccine application.
/// An optional integer may specify the dose for multi-dose vaccination schemes.
///
/// Using an enumeration is slightly more memory efficient than duplicating all parameters for all vaccines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CovidVaccine {
    Pfizer(Time, u8),
    Oxford(Time, u8),
    Johnson(Time),
}

impl CovidVaccine {
    #[inline]
    fn param_index(&self) -> usize {
        match self {
            Self::Pfizer(_, 0) => 0,
            Self::Pfizer(_, 1) => 1,
            Self::Pfizer(_, _) => 2,
            Self::Oxford(_, 0) => 3,
            Self::Oxford(_, 1) => 4,
            Self::Oxford(_, _) => 5,
            Self::Johnson(_) => 6,
        }
    }

    #[inline]
    pub fn params(&self) -> &VaccineParams<AgeParam> {
        return &VACCINE_PARAMETERS[self.param_index()];
    }

    pub fn prob_death(&self, age: Age) -> Real {
        return self.params().prob_death.for_age(age);
    }

    pub fn prob_contamination(&self, age: Age) -> Real {
        return self.params().prob_contamination.for_age(age);
    }

    /// Return a new copy of vaccination state mapping the time component by the given
    /// function.
    #[inline]
    pub fn map_time(&self, f: impl FnOnce(Time) -> Time) -> Self {
        match self {
            &Self::Pfizer(t, n) => Self::Pfizer(f(t), n),
            &Self::Oxford(t, n) => Self::Oxford(f(t), n),
            &Self::Johnson(t) => Self::Johnson(f(t)),
        }
    }

    /// Return a new copy of vaccination state mapping the dose component by the given
    /// function.
    #[inline]
    pub fn map_dose(&self, f: impl FnOnce(u8) -> u8) -> Self {
        match self {
            &Self::Pfizer(t, n) => Self::Pfizer(t, f(n)),
            &Self::Oxford(t, n) => Self::Oxford(t, f(n)),
            &Self::Johnson(t) => Self::Johnson(t),
        }
    }
}

pub struct VaccineParams<T> {
    prob_death: T,
    prob_contamination: T,
}

impl VaccineParams<AgeParam> {
    pub fn new(prob_death: impl Into<AgeParam>, prob_contamination: impl Into<AgeParam>) -> Self {
        let p1: AgeParam = prob_death.into();
        let p2: AgeParam = prob_contamination.into();
        return VaccineParams {
            prob_death: p1,
            prob_contamination: p2,
        };
    }
}

macro_rules! v {
    ($a:expr, $b:expr) => {
        VaccineParams::new($a, $b)
    };
}

lazy_static! {
    static ref VACCINE_PARAMETERS: [VaccineParams<AgeParam>; 7] = [
        // Pfizer
        v!(0.5, 0.5),
        v!(0.5, 0.5),
        v!(0.5, 0.5),
        // AstraZeneca
        v!(0.5, 0.5),
        v!(0.5, 0.5),
        v!(0.5, 0.5),
        // Johnson
        v!(0.5, 0.5),
        ];
}
