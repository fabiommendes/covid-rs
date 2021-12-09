use crate::{
    epidemic::{EpiModel, SEIRLike},
    params::EpiParamsLocalT,
    prelude::Real,
    sim::RandomUpdate,
};
use rand::Rng;

/// Concrete implementation of the SIR model. This model is generic over a
/// clinical parameter type C. If no distinction should be made between different
/// clinical states besides being in ant of the Susceptible, Infectious, Recovered
/// states, C can be safely set to ().
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SIR<C> {
    Susceptible,
    Infectious(C),
    Recovered(C),
    Dead(C),
}

impl<C> SIR<C> {
    pub fn clinical(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            Self::Susceptible => None,
            Self::Infectious(c) | Self::Recovered(c) | Self::Dead(c) => Some(c.clone()),
        }
    }
}

impl<C> Default for SIR<C> {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl<C: Clone> EpiModel for SIR<C> {
    const CARDINALITY: usize = 4;
    const CSV_HEADER: &'static str = "S,I,R,D";
    const S: usize = 0;
    const D: usize = 3;
    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Infectious(_) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn force_infectious(&mut self, force_dead: bool) -> bool {
        match self {
            Self::Susceptible => false,
            Self::Infectious(c) | Self::Recovered(c) => {
                *self = Self::Infectious(c.clone());
                return true;
            }
            Self::Dead(c) => {
                if force_dead {
                    *self = Self::Infectious(c.clone());
                    return true;
                }
                return false;
            }
        }
    }

    fn is_recovered(&self) -> bool {
        self.index() == Self::R
    }

    fn new_infectious_with(clinical: &Self::Clinical) -> Self {
        Self::Infectious(clinical.clone())
    }

    fn contagion_odds(&self) -> Real {
        match self {
            Self::Infectious(_) => 1.0,
            _ => 0.0,
        }
    }

    fn transfer_contamination_from(&mut self, other: &Self) -> bool {
        other
            .clinical()
            .map(|c| *self = Self::Infectious(c))
            .is_some()
    }
}

impl<C: Clone> SEIRLike for SIR<C> {
    const E: usize = 1;
    const I: usize = 1;
    const R: usize = 2;

    fn is_exposed(&self) -> bool {
        self.is_infectious()
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = Self::Infectious(with.clone())
    }
}

impl<C: Clone, P> RandomUpdate<P> for SIR<C>
where
    P: EpiParamsLocalT,
{
    fn random_update<R: Rng>(&mut self, params: &P, rng: &mut R) {
        match self {
            Self::Infectious(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.infection_fatality_ratio()) {
                        *self = Self::Dead(c.clone())
                    } else {
                        *self = Self::Recovered(c.clone())
                    }
                }
            }
            _ => (),
        }
    }
}
