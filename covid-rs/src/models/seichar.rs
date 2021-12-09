use rand::Rng;

use crate::{
    epidemic::{EpiModel, SEICHARLike, SEIRLike},
    params::EpiParamsLocalT,
    prelude::Real,
    sim::RandomUpdate,
};

/// Enumeration used internally to distinguish Exposed, Infectious, Asymptomatic
/// Critical and Severe in SEICHAR.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SEICHAR<C> {
    Susceptible,
    Exposed(C),
    Infectious(C),
    Critical(C),
    Severe(C),
    Asymptomatic(C),
    Recovered(C),
    Dead(C),
}

impl<C> SEICHAR<C> {
    pub fn clinical(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            Self::Susceptible => None,
            Self::Exposed(c)
            | Self::Infectious(c)
            | Self::Critical(c)
            | Self::Severe(c)
            | Self::Asymptomatic(c)
            | Self::Recovered(c)
            | Self::Dead(c) => Some(c.clone()),
        }
    }
}

impl<C> Default for SEICHAR<C> {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl<C: Clone> EpiModel for SEICHAR<C> {
    const CARDINALITY: usize = 8;
    const CSV_HEADER: &'static str = "S,E,I,C,H,A,R,D";
    const S: usize = 0;
    const D: usize = 7;

    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed(_) => Self::E,
            Self::Infectious(_) => Self::I,
            Self::Critical(_) => Self::C,
            Self::Severe(_) => Self::H,
            Self::Asymptomatic(_) => Self::A,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn new_infectious_with(clinical: &Self::Clinical) -> Self {
        Self::Infectious(clinical.clone())
    }

    fn force_infectious(&mut self, force_dead: bool) -> bool {
        match self {
            Self::Susceptible => false,
            Self::Exposed(c)
            | Self::Critical(c)
            | Self::Severe(c)
            | Self::Asymptomatic(c)
            | Self::Infectious(c)
            | Self::Recovered(c) => {
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

    fn contagion_odds(&self) -> Real {
        match self {
            Self::Infectious(_) => 1.0,
            Self::Asymptomatic(_) => 0.42,
            Self::Severe(_) => 0.1,
            Self::Critical(_) => 0.1,
            _ => 0.0,
        }
    }

    fn is_recovered(&self) -> bool {
        self.index() == Self::R
    }

    fn transfer_contamination_from(&mut self, other: &Self) -> bool {
        other.clinical().map(|c| *self = Self::Exposed(c)).is_some()
    }
}

impl<C: Clone> SEIRLike for SEICHAR<C> {
    const E: usize = 1;
    const I: usize = 2;
    const R: usize = 6;

    fn is_exposed(&self) -> bool {
        self.index() == Self::E
    }

    fn expose(&mut self, with: &Self::Clinical) {
        *self = Self::Exposed(with.clone())
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = Self::Infectious(with.clone())
    }
}

impl<C: Clone> SEICHARLike for SEICHAR<C> {
    const C: usize = 3;
    const H: usize = 4;
    const A: usize = 5;
}

impl<C: Clone, P> RandomUpdate<P> for SEICHAR<C>
where
    P: EpiParamsLocalT,
{
    fn random_update<R: Rng>(&mut self, params: &P, rng: &mut R) {
        match self {
            Self::Exposed(c) => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    if rng.gen_bool(params.prob_asymptomatic()) {
                        *self = Self::Asymptomatic(c.clone())
                    } else {
                        *self = Self::Infectious(c.clone())
                    }
                }
            }
            Self::Asymptomatic(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered(c.clone())
                }
            }
            Self::Infectious(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.prob_severe()) {
                        *self = Self::Severe(c.clone())
                    } else {
                        *self = Self::Recovered(c.clone());
                    }
                }
            }
            Self::Severe(c) => {
                if rng.gen_bool(params.severe_transition_prob()) {
                    if rng.gen_bool(params.prob_critical()) {
                        *self = Self::Critical(c.clone())
                    } else {
                        *self = Self::Recovered(c.clone());
                    }
                }
            }
            Self::Critical(c) => {
                if rng.gen_bool(params.critical_transition_prob()) {
                    if rng.gen_bool(params.prob_death()) {
                        *self = Self::Dead(c.clone());
                    } else {
                        *self = Self::Recovered(c.clone());
                    }
                }
            }
            _ => (),
        }
    }
}
