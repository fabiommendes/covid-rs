use super::{EpiParamsGlobal, EpiParamsLocalT, EpiParamsT, LocalBind, MultiComponent};
use crate::{
    models::SimpleAgent,
    prelude::{Age, Real},
    sim::HasAge,
};
use getset::*;

#[derive(Debug, Clone, Copy, CopyGetters, Setters, PartialEq)]
pub struct VaccineParams {
    protect_death: Real,
    protect_contamination: Real,
}

impl Default for VaccineParams {
    fn default() -> Self {
        return VaccineParams {
            protect_death: 1.0,
            protect_contamination: 1.0,
        };
    }
}

type Vaccine = Option<VaccineParams>;

/// This simple struct binds a group of parameters by age and vaccine. We assume
/// that parameters depend only on age, and vaccine affect other probabilities
/// and parameters in an age-dependent universal way.
#[derive(Debug, Clone, Copy, Getters, Setters, Default, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct BindVaccine<P> {
    params: P,
    age: Age,
    vaccine: Vaccine,
}

impl<M, D> LocalBind<SimpleAgent<M, VaccineParams>> for BindVaccine<EpiParamsGlobal<D>>
where
    D: MultiComponent<Elem = Real> + Default,
{
    type Local = Self;
    type World = EpiParamsGlobal<D>;
    type Bind = (Age, Vaccine);

    fn bind(&mut self, bind: (Age, Vaccine)) {
        self.age = bind.0;
        self.vaccine = bind.1;
    }

    fn local(&self) -> &Self::Local {
        self
    }

    fn world(&self) -> &Self::World {
        self.params()
    }

    fn world_mut(&mut self) -> &mut Self::World {
        &mut self.params
    }

    fn bind_to_object(&mut self, obj: &SimpleAgent<M, VaccineParams>) {
        let age = obj.age();
        let vaccine = obj.vaccine().clone();
        let bind = (age, vaccine);
        <BindVaccine<EpiParamsGlobal<D>> as LocalBind<SimpleAgent<(), VaccineParams>>>::bind(
            self, bind,
        );
    }
}

macro_rules! methods {
    (unaffected: { $($name:ident),* $(,)? }) => {
        $(
            fn $name(&self) -> Real {
                self.params.$name(&self.age)
            }
        )*
    };
}

impl<P> EpiParamsLocalT for BindVaccine<P>
where
    P: EpiParamsT<Age>,
{
    methods!(
        unaffected: {
            incubation_period,
            infectious_period,
            severe_period,
            critical_period,

            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,

            asymptomatic_infectiousness,
            prob_asymptomatic,
            prob_critical,
            prob_severe,
        }
    );
    fn prob_protect(&self) -> Real {
        if let Some(v) = self.vaccine {
            return v.protect_contamination;
        } else {
            return 0.0;
        }
    }
    fn case_fatality_ratio(&self) -> Real {
        let cfr = self.params.case_fatality_ratio(&self.age);

        if let Some(v) = self.vaccine {
            return cfr * (1.0 - v.protect_death);
        } else {
            return cfr;
        }
    }
}

impl<P> From<P> for BindVaccine<P> {
    fn from(params: P) -> Self {
        BindVaccine {
            params,
            age: 0,
            vaccine: Default::default(),
        }
    }
}
