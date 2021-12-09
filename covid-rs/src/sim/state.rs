use rand::Rng;

use crate::{
    prelude::{Age, AgeDistribution10, EpiModel},
    utils::random_ages,
};
use std::fmt::Debug;

use super::Population;

/// Minimal bounds on agent States.
pub trait State: Send + Clone + PartialEq + Debug + Default {} // Sync?
pub trait World: Send + Clone + PartialEq + Debug + Default {} // Sync?

/// A trait for agents that have an age field. Many epidemiological models are
/// age-sensitive and it is useful to isolate this property as trait to implement
/// generic functionality associated with processing ages.
pub trait HasAge {
    /// Agent's age.
    fn age(&self) -> Age;

    /// Set age with given value.
    fn set_age(&mut self, value: Age) -> &mut Self;
}

pub trait HasAgePopulationExt: Population
where
    Self::State: HasAge,
{
    /// Set age of all agents to the given value
    fn set_ages(&mut self, value: Age) -> &mut Self {
        self.each_agent_mut(|_, ag| {
            ag.set_age(value);
        });
        return self;
    }

    /// Rewrite age of all agents according to the given age distribution.
    fn distrib_ages<R: Rng>(&mut self, distrib: AgeDistribution10, rng: &mut R) -> &mut Self {
        let ages = random_ages(self.count(), rng, distrib);
        let mut i = 0;
        self.each_agent_mut(move |_, ag| {
            ag.set_age(ages[i]);
            i += 1;
        });

        return self;
    }
}

impl<P> HasAgePopulationExt for P
where
    P: Population,
    P::State: HasAge,
{
}

/// A trait for objects that have an compartment field with a SIR value.
pub trait HasEpiModel {
    type Model: EpiModel;

    /// Return the epidemiological compartment.
    fn epimodel(&self) -> &Self::Model;

    /// Return the epidemiological compartment.
    fn epimodel_mut(&mut self) -> &mut Self::Model;

    /// Set value of the epidemiological state.
    fn set_epimodel(&mut self, value: Self::Model) -> &mut Self {
        let model = self.epimodel_mut();
        *model = value;
        return self;
    }

    /// Apply random_update to the inner stochastic model. This usually is part
    /// of the implementation of a StochasticUpdate<W> trait for the parent
    /// model.
    fn epimodel_random_update<R: Rng, W>(&mut self, params: &W, rng: &mut R)
    where
        Self::Model: RandomUpdate<W>,
    {
        self.epimodel_mut().random_update(params, rng);
    }
}

/// A trait for objects that can be deterministically updated in the given
/// World.
pub trait DeterministicUpdate<W> {
    fn deterministic_update(&mut self, world: &W);
}

/// A trait for objects that can be stochastically updated in the given
/// World. Users must pass a random number generator in order to update
// this object
pub trait RandomUpdate<W> {
    fn random_update<R: Rng>(&mut self, world: &W, rng: &mut R);
}

/////////////////////////////////////////////////////////////////////////////
// Default trait implementations
/////////////////////////////////////////////////////////////////////////////

impl State for () {}

impl<W> DeterministicUpdate<W> for () {
    fn deterministic_update(&mut self, _world: &W) {}
}

impl<W> RandomUpdate<W> for () {
    fn random_update<R: Rng>(&mut self, _world: &W, _: &mut R) {}
}
