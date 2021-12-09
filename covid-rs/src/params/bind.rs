use super::{
    epi_local_params::EpiParamsLocalT, epi_params::EpiParamsT, EpiParamsGlobal, LocalBind,
    MultiComponent,
};
use crate::{
    prelude::{Age, Real},
    sim::HasAge,
};
use getset::{Getters, Setters};

/// A parameter set bound to a given state.
///
/// Impls should provide an API that mimics the wrapped Param Set omitting the
/// state argument which is passed implicitly from the state stored in this
/// struct.
#[derive(Debug, Clone, Copy, Getters, Setters, Default, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct Bind<P, S> {
    params: P,
    bind: S,
}

impl<P, S> Bind<P, S> {
    /// Binds a param set to state
    pub fn new(params: P, bind: S) -> Self {
        Bind { params, bind }
    }

    /// Return mutable reference to params.
    pub fn params_mut(&mut self) -> &mut P {
        &mut self.params
    }
}

/// A parameter set bound to a given state.
///
/// This is similar to BoundParams, but uses references instead of owning the
/// param set and state. Using this struct is potentially much more efficient
/// since it avoids unnecessary copies, but may lead to some lifetime issues
/// in some situations.
#[derive(Debug)]
pub struct BindRef<'a, P, B> {
    params: &'a P,
    bind: B,
}

impl<'a, P, S> BindRef<'a, P, S> {
    /// Binds a reference to a param set to state
    pub fn new(params: &'a P, bind: S) -> Self {
        BindRef { params, bind }
    }
}

/// Implement boilerplate for bound SEIRParam methods
macro_rules! bound_methods {
    ($($id:ident),* $(,)?) => {
        $(
            fn $id(&self) -> Real {
                self.params.$id(&self.bind)
            }
        )*
    };
    ($(& $id:ident),* $(,)?) => {
        $(
            fn $id(&self) -> Real {
                self.params.$id(&self.bind)
            }
        )*
    };
}

impl<P: EpiParamsT<S>, S> EpiParamsLocalT for Bind<P, S> {
    bound_methods!(
        incubation_period,
        infectious_period,
        severe_period,
        critical_period,
        asymptomatic_infectiousness,
        prob_asymptomatic,
        prob_severe,
        prob_critical,
        prob_death,
        prob_protect,
        case_fatality_ratio,
        infection_fatality_ratio,
        incubation_transition_prob,
        infectious_transition_prob,
        severe_transition_prob,
        critical_transition_prob,
    );
}

impl<'a, P: EpiParamsT<S>, S> EpiParamsLocalT for BindRef<'a, P, S> {
    bound_methods!(
        &incubation_period,
        &infectious_period,
        &severe_period,
        &critical_period,
        &asymptomatic_infectiousness,
        &prob_asymptomatic,
        &prob_severe,
        &prob_critical,
        &prob_death,
        &prob_protect,
        &case_fatality_ratio,
        &infection_fatality_ratio,
        &incubation_transition_prob,
        &infectious_transition_prob,
        &severe_transition_prob,
        &critical_transition_prob,
    );
}

impl<T, D> LocalBind<T> for Bind<EpiParamsGlobal<D>, Age>
where
    T: HasAge,
    D: MultiComponent<Elem = Real> + Default,
{
    type Local = Self;
    type World = EpiParamsGlobal<D>;
    type Bind = Age;

    default fn bind(&mut self, bind: Age) {
        self.set_bind(bind);
    }

    default fn local(&self) -> &Self::Local {
        self
    }

    default fn bind_to_object(&mut self, obj: &T) {
        let age = obj.age();
        <Self as LocalBind<T>>::bind(self, age);
    }

    default fn world(&self) -> &Self::World {
        self.params()
    }

    default fn world_mut(&mut self) -> &mut Self::World {
        self.params_mut()
    }
}
