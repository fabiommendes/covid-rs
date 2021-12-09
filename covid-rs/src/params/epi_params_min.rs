use super::{
    constants as cte,
    epi_local_params::EpiParamsLocalT,
    epi_params::{EpiParamsData, EpiParamsT},
    epi_params_clinical::EpiParamsClinical,
    ForBind, FromLocalParams, MultiComponent,
};
use crate::{
    epi_param_method, epi_param_methods,
    prelude::{AgeDistribution10, AgeParam, Real},
};
use getset::{Getters, Setters};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Stores a minimal set of epidemiological params.
///
/// Compose with EpiParamsClinical to control the full set of parameters instead
/// of relying of default values that assume an effective SEIR model.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Getters, Setters)]
#[serde(default)]
#[getset(set = "pub")]
pub struct EpiParamsMin<T> {
    #[getset(get = "pub with_prefix")]
    pub(crate) incubation_period: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) infectious_period: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) asymptomatic_infectiousness: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) prob_asymptomatic: T,
    #[getset(get = "pub with_prefix")]
    pub(crate) case_fatality_ratio: T,
}

impl<T: Default> Default for EpiParamsMin<T> {
    default fn default() -> Self {
        EpiParamsMin {
            incubation_period: T::default(),
            infectious_period: T::default(),
            asymptomatic_infectiousness: T::default(),
            prob_asymptomatic: T::default(),
            case_fatality_ratio: T::default(),
        }
    }
}

impl<T> EpiParamsMin<T> {
    pub fn new(
        incubation_period: T,
        infectious_period: T,
        asymptomatic_infectiousness: T,
        prob_asymptomatic: T,
        case_fatality_ratio: T,
    ) -> Self {
        EpiParamsMin {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
    }

    /// Create a new object from default components
    pub fn default_components() -> Self
    where
        T: MultiComponent<Elem = Real>,
    {
        EpiParamsMin {
            incubation_period: T::from_component(cte::INCUBATION_PERIOD),
            infectious_period: T::from_component(cte::INFECTIOUS_PERIOD),
            asymptomatic_infectiousness: T::from_component(cte::ASYMPTOMATIC_INFECTIOUSNESS),
            prob_asymptomatic: T::from_component(cte::PROB_ASYMPTOMATIC),
            case_fatality_ratio: T::from_component(cte::CASE_FATALITY_RATIO),
        }
    }

    /// Create a new object from epidemic distributions
    pub fn default_distributions() -> EpiParamsMin<AgeDistribution10> {
        EpiParamsMin {
            incubation_period: cte::INCUBATION_PERIOD_DISTRIBUTION,
            infectious_period: cte::INFECTIOUS_PERIOD_DISTRIBUTION,
            asymptomatic_infectiousness: cte::ASYMPTOMATIC_INFECTIOUSNESS_DISTRIBUTION,
            prob_asymptomatic: cte::PROB_ASYMPTOMATIC_DISTRIBUTION,
            case_fatality_ratio: cte::CASE_FATALITY_RATIO_DISTRIBUTION,
        }
    }

    /// Maps each param to function and construct a new EpidemicSEIRParams
    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> EpiParamsMin<S> {
        EpiParamsMin {
            incubation_period: f(&self.incubation_period),
            infectious_period: f(&self.infectious_period),
            asymptomatic_infectiousness: f(&self.asymptomatic_infectiousness),
            prob_asymptomatic: f(&self.prob_asymptomatic),
            case_fatality_ratio: f(&self.case_fatality_ratio),
        }
    }
}

impl<T, S> EpiParamsT<S> for EpiParamsMin<T>
where
    T: ForBind<S, Output = Real>,
{
    epi_param_methods!(
        by_field[S]: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
        by_value[S]: {
            severe_period: 0.0,
            critical_period: 0.0,
            prob_severe: 1.0,
            prob_critical: 1.0,
        }
    );

    fn prob_protect(&self, _obj: &S) -> Real {
        return 0.0;
    }
}

impl EpiParamsLocalT for EpiParamsMin<Real> {
    epi_param_methods!(
        by_field: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
        by_value: {
            severe_period: 0.0,
            critical_period: 0.0,
            prob_critical: 1.0,
            prob_severe: 1.0,
        }
    );

    fn prob_protect(&self) -> Real {
        return 0.0;
    }
}

impl<T> EpiParamsData<T> for EpiParamsMin<T>
where
    T: MultiComponent<Elem = Real>,
{
    epi_param_method!(data = incubation_period[T]);
    epi_param_method!(data = infectious_period[T]);
    epi_param_method!(data = severe_period[T], value = 0.0);
    epi_param_method!(data = critical_period[T], value = 0.0);
}

impl FromLocalParams for EpiParamsMin<Real> {
    fn from_local_params(params: &impl EpiParamsLocalT) -> Self {
        Self::new(
            params.incubation_period(),
            params.infectious_period(),
            params.asymptomatic_infectiousness(),
            params.prob_asymptomatic(),
            params.case_fatality_ratio(),
        )
    }
}

macro_rules! register_defaults {
    ($ty:ty, $name:ident) => {
        impl Default for EpiParamsMin<$ty> {
            fn default() -> Self {
                Self::$name().map(|x| (*x).into())
            }
        }

        impl Default for EpiParamsClinical<$ty> {
            fn default() -> Self {
                Self::$name().map(|x| (*x).into())
            }
        }
    };
}

register_defaults!(Real, default_components);
register_defaults!(AgeDistribution10, default_distributions);
register_defaults!(AgeParam, default_distributions);
