//! This module declares parameters for the covid-rs crate.  
//!
//! Managing and abstracting parameters is a good part of a real-world facing
//! simulation. We try to provide an interface that is at the same time efficient
//! (no boxed data or vtables), flexible and easy to use. Those goals are obviously
//! in conflict and sometimes some sacrifices were necessary.
// mod bind;
mod constants;
mod epi_param_cached;
mod epi_params;
mod epi_params_basic;
mod vaccines;
// mod vaccine_simple;

// pub use bind::*;
pub use constants::*;
pub use epi_param_cached::*;
pub use epi_params::*;
pub use epi_params_basic::*;
pub use vaccines::*;
// pub use vaccine_simple::*;

use crate::prelude::{AgeDistribution10, AgeParam, Real};
use constants as cte;

pub trait ParamSet<ST>
where
    Self: Clone,
{
    type BoundParams: EpiParams;
    // type GlobalParams: EpiParams;

    /// Return the local set of parameters bound to the given state.
    fn bind(&self, st: &ST) -> Self::BoundParams;

    // /// Return an object representing global parameters.
    // fn global_params(&self) -> Self::GlobalParams;
}

///////////////////////////////////////////////////////////////////////////////
// Basic public traits
///////////////////////////////////////////////////////////////////////////////

/*
/// A trait that maps Self with the expected output of a ParamSet after receiving
/// some bind value B as argument.
///
/// A simple example: Self might be an array of reals representing a probability
/// distribution, B can be an age and Output is the value corresponding to each
/// age group.
///
/// In this scenario, for_state(age) maps ages to values T extracted from the
/// Self array [T].
pub trait ForBind<B> {
    type Output;

    /// Maps a value of obj to the desired Output value.
    fn for_state(&self, value: &B) -> Self::Output;
}

impl<T, S> ForBind<S> for T
where
T: ForAge,
S: HasAge,
{
    type Output = T::Output;

    #[inline]
    default fn for_state(&self, obj: &S) -> T::Output {
        self.for_age(obj.age())
    }
}

impl<T: ForAge<Output = Real>> ForBind<Age> for T {
    type Output = Real;

    #[inline]
    default fn for_state(&self, age: &Age) -> Real {
        self.for_age(*age)
    }
}
*/

/// A trait related to ForState, which allows transformation of the inner data
/// by some mapping in the expected elements.
///
/// This trait is natural for types that somehow store a collection of elements
/// that can be retrieved by some key S using the ForState<S, Output=Elem> trait.
pub trait MultiComponent
where
    Self: Sized,
{
    type Elem;

    /// Uses f(x) to transform self internally and return a mapped result.
    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self;

    /// Create data from single element, possibly replicating it for all keys.
    fn from_component(x: Self::Elem) -> Self;
}

impl MultiComponent for Real {
    type Elem = Real;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        f(*self)
    }

    fn from_component(x: Self::Elem) -> Self {
        x
    }
}

impl<T, const N: usize> MultiComponent for [T; N]
where
    T: Sized + Copy,
{
    type Elem = T;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        (&self).map(f)
    }

    fn from_component(x: Self::Elem) -> Self {
        [x; N]
    }
}

impl MultiComponent for AgeParam {
    type Elem = Real;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        self.map(f)
    }

    fn from_component(x: Self::Elem) -> Self {
        AgeParam::Scalar(x)
    }
}

/*
/// Trait for types that can be created from a EpiLocalParams implementation
pub trait FromLocalParams {
    /// Create new instances from an UniversalSEIRParams implementation
    fn from_local_params(params: &impl EpiParams) -> Self;
}

///////////////////////////////////////////////////////////////////////////////
// Type aliases
///////////////////////////////////////////////////////////////////////////////

/// The recommended type to hold epidemiological params.
pub type EpiParamsGlobal<T> = EpiParamsCached<EpiParamsFull<T>, T>;

/// A type that is usable as an universal global param set.
pub type EpiParamsLocal = EpiParamsGlobal<Real>;

/// A type alias for vaccine-dependent models
pub type EpiParamsBindVaccine<T> = BindVaccine<EpiParamsGlobal<T>>;

/// A type alias for bound age-dependent SEIR params that implements the
/// LocalBind trait.
pub type EpiParamsBindAge<T> = Bind<EpiParamsGlobal<T>, Age>;
*/

/// The most simple param
pub fn simple_params() -> EpiParamsCached<Real> {
    return EpiParamsCached::new(EpiParamsData::default_from_scalars());
}

/// The most simple param
pub fn age_dependent_params() -> EpiParamsCached<AgeDistribution10> {
    let base = EpiParamsData {
        incubation_period: cte::INCUBATION_PERIOD_DISTRIBUTION,
        infectious_period: cte::INFECTIOUS_PERIOD_DISTRIBUTION,
        asymptomatic_infectiousness: cte::ASYMPTOMATIC_INFECTIOUSNESS_DISTRIBUTION,
        prob_asymptomatic: cte::PROB_ASYMPTOMATIC_DISTRIBUTION,
        case_fatality_ratio: cte::CASE_FATALITY_RATIO_DISTRIBUTION,
        severe_period: cte::SEVERE_PERIOD_DISTRIBUTION,
        critical_period: cte::CRITICAL_PERIOD_DISTRIBUTION,
        prob_severe: cte::PROB_SEVERE_DISTRIBUTION,
        prob_critical: cte::PROB_CRITICAL_DISTRIBUTION,
    };
    return EpiParamsCached::new(base);
}
