use super::{
    epi_local_params::EpiParamsLocalT, epi_params::EpiParamsT, EpiParamsCached, EpiParamsClinical,
    EpiParamsData, EpiParamsMin, ForBind, FromLocalParams, MultiComponent,
};
use crate::{epi_param_method, epi_param_methods, prelude::Real};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Compose { epidemic: EpidemicSEIRParams, clinical: EpiParamsClinical }.
///
/// The two fields can be accessed directly.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EpiParamsFull<T: Default> {
    // FIXME: we only added "Default" to be able to implement Deserialize
    pub epidemic: EpiParamsMin<T>,
    pub clinical: EpiParamsClinical<T>,
}

impl<T: Default> EpiParamsFull<T> {
    pub fn new(epidemic: EpiParamsMin<T>, clinical: EpiParamsClinical<T>) -> Self {
        EpiParamsFull { epidemic, clinical }
    }

    pub fn map<S: Default>(&self, f: impl Fn(&T) -> S) -> EpiParamsFull<S> {
        EpiParamsFull::new(self.epidemic.map(&f), self.clinical.map(&f))
    }

    /// Return a cached version of param set
    pub fn cached(&self) -> EpiParamsCached<Self, T>
    where
        // P: SEIRParamsData<T>,
        T: MultiComponent<Elem = Real> + Clone,
    {
        EpiParamsCached::new(self)
    }
}

impl<T, S> EpiParamsT<S> for EpiParamsFull<T>
where
    T: MultiComponent<Elem = Real> + ForBind<S, Output = Real> + Default,
{
    // Epidemic methods
    epi_param_method!(incubation_period[S], delegate = epidemic);
    epi_param_method!(infectious_period[S], delegate = epidemic);
    epi_param_method!(asymptomatic_infectiousness[S], delegate = epidemic);
    epi_param_method!(prob_asymptomatic[S], delegate = epidemic);
    epi_param_method!(case_fatality_ratio[S], delegate = epidemic);
    epi_param_method!(incubation_transition_prob[S], delegate = epidemic);
    epi_param_method!(infectious_transition_prob[S], delegate = epidemic);

    // Clinical methods
    epi_param_method!(severe_period[S], delegate = clinical);
    epi_param_method!(critical_period[S], delegate = clinical);
    epi_param_method!(prob_severe[S], delegate = clinical);
    epi_param_method!(prob_critical[S], delegate = clinical);
    epi_param_method!(severe_transition_prob[S], delegate = clinical);
    epi_param_method!(critical_transition_prob[S], delegate = clinical);

    fn prob_protect(&self, _obj: &S) -> Real {
        return 0.0;
    }
}

impl EpiParamsLocalT for EpiParamsFull<Real> {
    // Epidemic methods
    epi_param_methods!(
        delegate[epidemic]: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            prob_protect,
            case_fatality_ratio,
            incubation_transition_prob,
            infectious_transition_prob,
        }
        forward[clinical]: {
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    );
}

impl<T: Default> EpiParamsData<T> for EpiParamsFull<T>
where
    T: MultiComponent<Elem = Real>,
{
    epi_param_method!(data = incubation_period[T], delegate = epidemic);
    epi_param_method!(data = infectious_period[T], delegate = epidemic);
    epi_param_method!(data = severe_period[T], delegate = clinical);
    epi_param_method!(data = critical_period[T], delegate = clinical);
}

impl FromLocalParams for EpiParamsFull<Real> {
    fn from_local_params(params: &impl EpiParamsLocalT) -> Self {
        Self::new(
            FromLocalParams::from_local_params(params),
            FromLocalParams::from_local_params(params),
        )
    }
}
