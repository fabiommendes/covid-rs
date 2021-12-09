//! This module declares parameters for the covid-rs crate.  
//!
//! Managing and abstracting parameters is a good part of a real-world facing
//! simulation. We try to provide an interface that is at the same time efficient
//! (no boxed data or vtables), flexible and easy to use. Those goals are obviously
//! in conflict and sometimes some sacrifices were necessary.
mod bind;
mod constants;
mod epi_local_params;
mod epi_param_cached;
mod epi_params;
mod epi_params_clinical;
mod epi_params_full;
mod epi_params_min;
mod macros;
mod vaccine_simple;

pub use bind::*;
pub use constants::*;
pub use epi_local_params::*;
pub use epi_param_cached::*;
pub use epi_params::*;
pub use epi_params_clinical::*;
pub use epi_params_full::*;
pub use epi_params_min::*;
pub use vaccine_simple::*;

use crate::{prelude::{Age, AgeParam, ForAge, Real}, sim::HasAge};

///////////////////////////////////////////////////////////////////////////////
// Basic public traits
///////////////////////////////////////////////////////////////////////////////

/// This trait binds global parameters to a local view seen by each agent.
///
/// This view can depend on the agent's state or (more typically) on parts of
/// the agent's state. A typical example is a set of parameters that depend on
/// age can be bound to some agent's age before passing to its update functions.
/// The bound parameters would only show values pertinent to the agent's specific
/// age.
///
/// The local bind must also expose a mutable set of world parameters. Those
/// parameters are exposed as references for efficiency and thus must be owned.
pub trait LocalBind<S> {
    type Local;
    type World;
    type Bind;

    /// Bind parameters to value
    fn bind(&mut self, bind: Self::Bind);

    /// Return a copy bound to the given bind state
    fn local_clone(&self, bind: Self::Bind) -> Self::Local
    where
        Self::Local: Clone,
        Self: Clone,
    {
        let mut new = self.clone();
        new.bind(bind);
        return new.local().clone();
    }

    /// Return local parameters for current bind
    fn local(&self) -> &Self::Local;

    /// Return a reference to the world parameters
    fn world(&self) -> &Self::World;

    /// Return a mutable reference to the world parameters
    fn world_mut(&mut self) -> &mut Self::World;

    /// Just a convenience function that extract bind data and then binds
    fn bind_to_object(&mut self, _: &S);

    /// Return a copy bound to the given bind state
    fn clone_to_object(&self, obj: &S) -> Self::Local
    where
        Self::Local: Clone,
        Self: Clone,
    {
        let mut new = self.clone();
        new.bind_to_object(obj);
        return new.local().clone();
    }
}

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

/// Trait for types that can be created from a EpiLocalParams implementation
pub trait FromLocalParams {
    /// Create new instances from an UniversalSEIRParams implementation
    fn from_local_params(params: &impl EpiParamsLocalT) -> Self;
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
