use super::{epi_params::daily_probability, EpiParams, EpiParamsData, MultiComponent};
use crate::prelude::Real;
use getset::Getters;
use std::fmt::Debug;

/// A cached params take a params impl and caches all transition probability
/// values. This avoids some potentially expensive computations involving
/// exponentials by paying a fixed cost upfront when writing data for each
/// corresponding transition period.
#[derive(Copy, Clone, Debug, PartialEq, Getters)]
pub struct EpiParamsCached<T> {
    #[getset(get = "pub")]
    params: EpiParamsData<T>,

    incubation_transition_prob: T,
    infectious_transition_prob: T,
    severe_transition_prob: T,
    critical_transition_prob: T,
}

impl<T> EpiParamsCached<T>
where
    T: MultiComponent<Elem = Real>,
{
    pub fn new(params: EpiParamsData<T>) -> Self {
        EpiParamsCached {
            incubation_transition_prob: params
                .with_incubation_period_data(|xs| xs.map_components(daily_probability)),
            infectious_transition_prob: params
                .with_infectious_period_data(|xs| xs.map_components(daily_probability)),
            severe_transition_prob: params
                .with_severe_period_data(|xs| xs.map_components(daily_probability)),
            critical_transition_prob: params
                .with_critical_period_data(|xs| xs.map_components(daily_probability)),
            params: params,
        }
    }
}

impl<T> Default for EpiParamsCached<T>
where
    T: MultiComponent<Elem = Real> + Default,
{
    default fn default() -> Self {
        Self::new(EpiParamsData::default())
    }
}

impl<T> From<EpiParamsData<T>> for EpiParamsCached<T>
where
    T: MultiComponent<Elem = Real>,
{
    fn from(params: EpiParamsData<T>) -> Self {
        Self::new(params)
    }
}

macro_rules! delegate_to_params {
    ($name:ident, bind=$ty:ident) => {
        fn $name(&self, obj: &$ty) {
            return self.params.$name(obj);
        }
    };
    ($name:ident) => {
        fn $name(&self) -> Real {
            return self.params.$name();
        }
    };
}

macro_rules! read_from_attr {
    ($name:ident, bind=$ty:ident) => {
        fn $name(&self, obj: &$ty) {
            return self.$name(obj);
        }
    };
    ($name:ident) => {
        fn $name(&self) -> Real {
            return self.$name;
        }
    };
}

// impl<B, T> PartialEpiParams<B> for EpiParamsCached<T> {
//     // Delegate to attributes
//     delegate_to_params!(incubation_period, bind = B);
//     delegate_to_params!(infectious_period, bind = B);
//     delegate_to_params!(severe_period, bind = B);
//     delegate_to_params!(critical_period, bind = B);
//     delegate_to_params!(asymptomatic_infectiousness, bind = B);
//     delegate_to_params!(prob_asymptomatic, bind = B);
//     delegate_to_params!(prob_severe, bind = B);
//     delegate_to_params!(prob_critical, bind = B);
//     delegate_to_params!(prob_death, bind = B);
//     delegate_to_params!(case_fatality_ratio, bind = B);
//     delegate_to_params!(infection_fatality_ratio, bind = B);

//     // Read directly from attributes
//     epi_param_methods!(
//        by_field[S]: {
//             incubation_transition_prob,
//             infectious_transition_prob,
//             severe_transition_prob,
//             critical_transition_prob,
//         }
//     );

//     fn prob_protect(&self, _obj: &S) -> Real {
//         return 0.0;
//     }
// }

impl EpiParams for EpiParamsCached<Real> {
    delegate_to_params!(incubation_period);
    delegate_to_params!(infectious_period);
    delegate_to_params!(asymptomatic_infectiousness);
    delegate_to_params!(prob_asymptomatic);
    delegate_to_params!(case_fatality_ratio);
    delegate_to_params!(infection_fatality_ratio);
    delegate_to_params!(prob_death);
    delegate_to_params!(prob_protect);
    delegate_to_params!(severe_period);
    delegate_to_params!(critical_period);
    delegate_to_params!(prob_severe);
    delegate_to_params!(prob_critical);

    // Read directly from attributes
    read_from_attr!(incubation_transition_prob);
    read_from_attr!(infectious_transition_prob);
    read_from_attr!(severe_transition_prob);
    read_from_attr!(critical_transition_prob);
}

/*
impl<P> FromLocalParams for EpiParamsCached<P, Real>
where
    P: FromLocalParams + EpiParamsData<Real> + Clone,
{
    fn from_local_params(params: &impl EpiParams) -> Self {
        let src: P = FromLocalParams::from_local_params(params);
        Self::new(&src)
    }
}
*/
