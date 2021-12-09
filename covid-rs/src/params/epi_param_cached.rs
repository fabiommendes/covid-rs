use super::{
    epi_local_params::EpiParamsLocalT,
    epi_params::{daily_probability, EpiParamsData, EpiParamsT},
    EpiParamsFull, ForBind, FromLocalParams, LocalBind, MultiComponent,
};
use crate::{epi_param_method, epi_param_methods, prelude::Real};
use getset::Getters;
use paste::paste;
use std::fmt::Debug;

/// A cached params take a params impl and caches all transition probability
/// values. This avoids some potentially expensive computations involving
/// exponentials by paying a fixed cost upfront when writing data for each
/// corresponding transition period.
#[derive(Copy, Clone, Debug, PartialEq, Getters)]
pub struct EpiParamsCached<P, T> {
    #[getset(get = "pub")]
    params: P,
    incubation_transition_prob: T,
    infectious_transition_prob: T,
    severe_transition_prob: T,
    critical_transition_prob: T,
}

impl<P, T> EpiParamsCached<P, T>
where
    P: EpiParamsData<T> + Clone,
    T: MultiComponent<Elem = Real>,
{
    pub fn new(params: &P) -> Self {
        EpiParamsCached {
            incubation_transition_prob: params
                .with_incubation_period_data(|xs| xs.map_components(daily_probability)),
            infectious_transition_prob: params
                .with_infectious_period_data(|xs| xs.map_components(daily_probability)),
            severe_transition_prob: params
                .with_severe_period_data(|xs| xs.map_components(daily_probability)),
            critical_transition_prob: params
                .with_critical_period_data(|xs| xs.map_components(daily_probability)),
            params: params.clone(),
        }
    }
}

impl<P, T> Default for EpiParamsCached<P, T>
where
    P: EpiParamsData<T> + Default + Clone,
    T: MultiComponent<Elem = Real>,
{
    default fn default() -> Self {
        let params = P::default();
        Self::new(&params)
    }
}

impl<P, T> From<P> for EpiParamsCached<P, T>
where
    P: EpiParamsData<T> + Clone,
    T: MultiComponent<Elem = Real>,
{
    fn from(params: P) -> Self {
        Self::new(&params)
    }
}

impl<P, T, S> EpiParamsT<S> for EpiParamsCached<P, T>
where
    P: EpiParamsT<S>,
    T: ForBind<S, Output = Real>,
{
    // Delegate to attributes
    epi_param_method!(incubation_period[S], delegate = params);
    epi_param_method!(infectious_period[S], delegate = params);
    epi_param_method!(severe_period[S], delegate = params);
    epi_param_method!(critical_period[S], delegate = params);
    epi_param_method!(asymptomatic_infectiousness[S], delegate = params);
    epi_param_method!(prob_asymptomatic[S], delegate = params);
    epi_param_method!(prob_severe[S], delegate = params);
    epi_param_method!(prob_critical[S], delegate = params);
    epi_param_method!(prob_death[S], delegate = params);
    epi_param_method!(case_fatality_ratio[S], delegate = params);
    epi_param_method!(infection_fatality_ratio[S], delegate = params);

    // Read directly from attributes
    epi_param_methods!(
       by_field[S]: {
            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,
        }
    );

    fn prob_protect(&self, _obj: &S) -> Real {
        return 0.0;
    }
}

impl<P> EpiParamsLocalT for EpiParamsCached<P, Real>
where
    P: EpiParamsLocalT,
{
    // Epidemic methods
    epi_param_methods!(
        delegate[params]: {
            // Epidemic
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
            infection_fatality_ratio,
            prob_death,
            prob_protect,

            // Clinical
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    );

    // Read directly from attributes
    epi_param_methods!(
       by_field: {
            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,
        }
    );
}

impl<P> FromLocalParams for EpiParamsCached<P, Real>
where
    P: FromLocalParams + EpiParamsData<Real> + Clone,
{
    fn from_local_params(params: &impl EpiParamsLocalT) -> Self {
        let src: P = FromLocalParams::from_local_params(params);
        Self::new(&src)
    }
}

impl<S> LocalBind<S> for EpiParamsCached<EpiParamsFull<Real>, Real> {
    type Local = Self;
    type World = Self;
    type Bind = S;

    fn bind(&mut self, _: Self::Bind) {}

    fn bind_to_object(&mut self, _: &S) {}

    fn local(&self) -> &Self::Local {
        self
    }

    fn world(&self) -> &Self::World {
        self
    }

    fn world_mut(&mut self) -> &mut Self::World {
        self
    }
}
