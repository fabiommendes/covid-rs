use super::{
    constants as cte,
    epi_params::EpiParams,
    MultiComponent,
    ParamSet,
};
use crate::prelude::Real;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, rc::Rc};

/// Stores a minimal set of epidemiological params.
///
/// EpiParamsCached can be slightly more efficient to use as it stores the value of
/// some derived parameters in dedicated struct fields. Users should usually
/// use EpiParamsCached for its performance gain, unless the internal parameter
/// suffers from lots of mutations.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Getters, Setters)]
#[serde(default)]
#[getset(set = "pub")]
pub struct EpiParamsData<T> {
    // Epidemiological parameters
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

    // Clinical parameters
    #[getset(get = "pub with_prefix")]
    pub(crate) severe_period: T,

    #[getset(get = "pub with_prefix")]
    pub(crate) critical_period: T,

    #[getset(get = "pub with_prefix")]
    pub(crate) prob_severe: T,

    #[getset(get = "pub with_prefix")]
    pub(crate) prob_critical: T,
}

impl<T: Default> Default for EpiParamsData<T> {
    default fn default() -> Self {
        EpiParamsData {
            incubation_period: T::default(),
            infectious_period: T::default(),
            asymptomatic_infectiousness: T::default(),
            prob_asymptomatic: T::default(),
            case_fatality_ratio: T::default(),
            severe_period: T::default(),
            critical_period: T::default(),
            prob_severe: T::default(),
            prob_critical: T::default(),
        }
    }
}

impl<T> EpiParamsData<T> {
    /// Creates a new set of parameters, but can specify new clinical attributes.
    pub fn new_with_clinical(
        incubation_period: T,
        infectious_period: T,
        asymptomatic_infectiousness: T,
        prob_asymptomatic: T,
        case_fatality_ratio: T,
        severe_period: T,
        critical_period: T,
        prob_severe: T,
        prob_critical: T,
    ) -> Self {
        EpiParamsData {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    }

    /// Create a new object from default components
    pub fn default_from_scalars() -> Self
    where
        T: MultiComponent<Elem = Real>,
    {
        EpiParamsData {
            incubation_period: T::from_component(cte::INCUBATION_PERIOD),
            infectious_period: T::from_component(cte::INFECTIOUS_PERIOD),
            asymptomatic_infectiousness: T::from_component(cte::ASYMPTOMATIC_INFECTIOUSNESS),
            prob_asymptomatic: T::from_component(cte::PROB_ASYMPTOMATIC),
            case_fatality_ratio: T::from_component(cte::CASE_FATALITY_RATIO),
            severe_period: T::from_component(cte::SEVERE_PERIOD),
            critical_period: T::from_component(cte::CRITICAL_PERIOD),
            prob_severe: T::from_component(cte::PROB_SEVERE),
            prob_critical: T::from_component(cte::PROB_CRITICAL),
        }
    }

    /// Maps each param to function and construct a new EpiParamsMin
    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> EpiParamsData<S> {
        EpiParamsData {
            incubation_period: f(&self.incubation_period),
            infectious_period: f(&self.infectious_period),
            asymptomatic_infectiousness: f(&self.asymptomatic_infectiousness),
            prob_asymptomatic: f(&self.prob_asymptomatic),
            case_fatality_ratio: f(&self.case_fatality_ratio),
            severe_period: f(&self.severe_period),
            critical_period: f(&self.critical_period),
            prob_severe: f(&self.prob_severe),
            prob_critical: f(&self.prob_critical),
        }
    }

    pub fn with_incubation_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S {
        todo!()
    }
    pub fn with_infectious_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S {
        todo!()
    }
    pub fn with_severe_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S {
        todo!()
    }
    pub fn with_critical_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S {
        todo!()
    }

    /// Helper method that may make it easier to implement with_*_data() methods
    /// for missing values.
    pub fn with_scalar_data<R, S>(&self, scalar: R, f: impl FnOnce(&T) -> S) -> S
    where
        T: MultiComponent<Elem = R>,
    {
        let data = T::from_component(scalar);
        return f(&data);
    }
}

impl EpiParamsData<Real> {
    pub fn new(
        incubation_period: Real,
        infectious_period: Real,
        asymptomatic_infectiousness: Real,
        prob_asymptomatic: Real,
        case_fatality_ratio: Real,
    ) -> Self {
        EpiParamsData {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
            severe_period: 0.0,
            critical_period: 0.0,
            prob_severe: 1.0,
            prob_critical: 1.0,
        }
    }
}

// impl EpiParams for &EpiParamsData<Real> {
//     epi_param_methods!(
//         by_field: {
//             incubation_period,
//             infectious_period,
//             asymptomatic_infectiousness,
//             prob_asymptomatic,
//             case_fatality_ratio,
//         }
//         by_value: {
//             severe_period: 0.0,
//             critical_period: 0.0,
//             prob_critical: 1.0,
//             prob_severe: 1.0,
//             prob_protect: 0.0,
//         }
//     );
// }

macro_rules! epi_params_impls {
    // Create functions that receive no arguments but self
    (
        $(by_field: { $($name:ident),* $(,)? })?
        $(by_value: { $($vname:ident: $value:expr),* $(,)? })?
    ) => {
        $($(
            fn $name(&self) -> Real {
                self.$name
            }
        )*)*
        $($(
            fn $vname(&self) -> Real {
                $value
            }
        )*)*
    };
}

impl EpiParams for EpiParamsData<Real> {
    epi_params_impls!(
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
            prob_severe: 1.0,
            prob_critical: 1.0,
            prob_protect: 0.0,
        }
    );
}

/*
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
    fn from_local_params(params: &impl EpiParams) -> Self {
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

*/

impl<ST> ParamSet<ST> for EpiParamsData<Real> {
    type BoundParams = Self;

    fn bind(&self, _: &ST) -> Self::BoundParams {
        return *self;
    }
}

impl<ST> ParamSet<ST> for Rc<EpiParamsData<Real>> {
    type BoundParams = Self;

    fn bind(&self, _: &ST) -> Self::BoundParams {
        return self.clone();
    }
}
