use crate::{
    params::{EpiParams, ParamSet},
    prelude::{Time, EpiModel},
    sampler::PopulationSampler,
    sim::{Population, RandomUpdate},
};
use getset::{CopyGetters, Getters, MutGetters};
use rand::{prelude::SmallRng, Rng};
use std::fmt::Debug;

/// The bare bones simulation state.
///
/// This struct stores all agents, global parameters necessary and an internal random
/// number generator necessary execute the simulation. It differs from Engine, which is
/// also responsible to trigger events, schedule tasks and track the overall
/// execution state.
#[derive(Debug, Clone, Getters, CopyGetters, MutGetters)]
pub struct Simulation<P: Clone, ST: Clone> {
    #[getset(get_copy = "pub")]
    pub time: Time,

    #[getset(get = "pub", get_mut = "pub")]
    pub population: Vec<ST>,

    #[getset(get = "pub", get_mut = "pub")]
    pub params: P,

    #[getset(get = "pub", get_mut = "pub")]
    pub rng: SmallRng,
}

impl<P, ST> Simulation<P, ST>
where
    P: ParamSet<ST>,
    ST: RandomUpdate<P::BoundParams> + EpiModel + Clone,
    P::BoundParams: EpiParams,
{
    /// Advance a single simulation step.
    ///
    /// Infection pairs are produced by the sampler object.
    ///
    /// The callback function `cb` is executed for every new infection pair.
    pub fn step<S, F>(&mut self, sampler: &S, cb: F) -> usize
    where
        S: PopulationSampler<Vec<ST>>,
        F: FnMut(usize, usize),
    {
        self.time += 1;

        // Natural evolution of each agent.
        for obj in &mut self.population {
            obj.random_update(&self.params.bind(obj), &mut self.rng);
        }

        // Simulate agent interactions, allowing new infections to occur.
        let mut cases = 0usize;
        let mut on_infection = cb;

        for (i, j) in sampler.sample_infection_pairs(&self.population, &mut self.rng) {
            if i == j {
                continue;
            }
            if let Some((src, dest)) = self.population.get_pair_mut(i, j) {
                if !self.rng.gen_bool(self.params.bind(src).prob_protect())
                    && dest.contaminate_from(src)
                {
                    cases += 1;
                    on_infection(i, j);
                }
            }
        }
        return cases;
    }
}
