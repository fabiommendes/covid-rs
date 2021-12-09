use super::{
    constants as cte, epi_local_params::EpiParamsLocalT, epi_params::daily_probability, ForBind,
    FromLocalParams, MultiComponent,
};
use crate::{
    epi_param_method,
    prelude::{AgeDistribution10, Real},
};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// EpiParamsClinical store information about the clinical evolution of cases.
///
/// Composing with EpidemicSEIRParams, it is possible to implement arbitrary values
/// for the full set of EpiLocalParams methods.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Getters, Setters)]
#[serde(default)]
#[getset(set = "pub")]
pub struct EpiParamsClinical<T> {
    #[getset(get = "pub with_prefix")]
    pub(crate) severe_period: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) critical_period: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) prob_severe: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) prob_critical: T,
}

impl<T> EpiParamsClinical<T> {
    pub fn new(severe_period: T, critical_period: T, prob_severe: T, prob_critical: T) -> Self {
        EpiParamsClinical {
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    }

    pub fn default_components() -> Self
    where
        T: MultiComponent<Elem = Real>,
    {
        EpiParamsClinical {
            severe_period: T::from_component(cte::SEVERE_PERIOD),
            critical_period: T::from_component(cte::CRITICAL_PERIOD),
            prob_severe: T::from_component(cte::PROB_SEVERE),
            prob_critical: T::from_component(cte::PROB_CRITICAL),
        }
    }

    pub fn default_distributions() -> EpiParamsClinical<AgeDistribution10> {
        EpiParamsClinical {
            severe_period: cte::SEVERE_PERIOD_DISTRIBUTION,
            critical_period: cte::CRITICAL_PERIOD_DISTRIBUTION,
            prob_severe: cte::PROB_SEVERE_DISTRIBUTION,
            prob_critical: cte::PROB_CRITICAL_DISTRIBUTION,
        }
    }

    /// Maps function to each component of struct
    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> EpiParamsClinical<S> {
        EpiParamsClinical {
            severe_period: f(&self.severe_period),
            critical_period: f(&self.critical_period),
            prob_severe: f(&self.prob_severe),
            prob_critical: f(&self.prob_critical),
        }
    }

    pub fn severe_transition_prob<S>(&self, obj: &S) -> Real
    where
        T: ForBind<S, Output = Real>,
    {
        daily_probability(self.severe_period(obj))
    }

    pub fn critical_transition_prob<S>(&self, obj: &S) -> Real
    where
        T: ForBind<S, Output = Real>,
    {
        daily_probability(self.critical_period(obj))
    }

    epi_param_method!(severe_period<S>);
    epi_param_method!(critical_period<S>);
    epi_param_method!(prob_severe<S>);
    epi_param_method!(prob_critical<S>);
}

impl<T: Default> Default for EpiParamsClinical<T> {
    default fn default() -> Self {
        EpiParamsClinical {
            severe_period: T::default(),
            critical_period: T::default(),
            prob_severe: T::default(),
            prob_critical: T::default(),
        }
    }
}

impl FromLocalParams for EpiParamsClinical<Real> {
    fn from_local_params(params: &impl EpiParamsLocalT) -> Self {
        Self::new(
            params.severe_period(),
            params.critical_period(),
            params.prob_severe(),
            params.prob_critical(),
        )
    }
}
