use super::{population::Population, state::RandomUpdate, HasEpiModel};
use crate::{
    epidemic::*,
    events::{EpidemicSimulationMsg, EventDispatcher},
    // params::{EpiParamsFull, EpiParams, FromLocalParams, LocalBind},
    params::{EpiParams, ParamSet},
    prelude::*,
    scheduler::Scheduler,
    utils::Table,
};
use getset::{Getters, MutGetters};
use log;
use rand::{
    prelude::{SeedableRng, SmallRng},
    Rng,
};
use rayon::prelude::*;
use std::{fmt::Debug};

/// Simulation state
#[derive(Debug, Clone)]
pub struct SimulationState<P: Clone, ST: Clone> {
    pub time: Time,
    pub population: Vec<ST>,
    pub params: P,
    pub rng: SmallRng,
}

impl<P, ST> SimulationState<P, ST>
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

/// Simulation stores a population of agents and some objects responsible for
/// controlling the dynamics of those Agents.
#[derive(Getters, MutGetters, Clone)]
pub struct Simulation<ST: Clone, P: Clone, SP: Clone> {
    state: SimulationState<P, ST>,

    // #[getset(get = "pub", get_mut = "pub")]
    // params: P,
    #[getset(get = "pub", get_mut = "pub")]
    sampler: SP,
    dispatcher: EventDispatcher<EpidemicSimulationMsg>,
    scheduler: Scheduler<SimulationState<P, ST>>,
    epicurves: Option<Table<usize>>,
}

impl<'a, ST: Clone, P: Clone, SP: Clone> Simulation<ST, P, SP>
where
    P: ParamSet<ST>,
    P::BoundParams: EpiParams,
    ST: EpiModel + RandomUpdate<P::BoundParams> + Debug,
    SP: PopulationSampler<Vec<ST>>,
{
    /// Create new simulation from population and sampler.
    pub fn new(params: P, population: Vec<ST>, sampler: SP) -> Self {
        Simulation {
            epicurves: Some(Table::new(ST::column_names())),
            state: SimulationState {
                population,
                time: 0,
                params: params,
                rng: SmallRng::from_entropy(),
            },
            dispatcher: EventDispatcher::new_with_default_listeners(),
            scheduler: Scheduler::new(),
            sampler,
        }
    }

    /// Reference to the population vector
    pub fn population(&self) -> &Vec<ST> {
        return &self.state.population;
    }

    /// Mutable reference to the population vector
    pub fn population_mut(&mut self) -> &mut Vec<ST> {
        return &mut self.state.population;
    }

    /// Return the current simulation time
    pub fn time(&self) -> Time {
        return self.state.time;
    }

    /** Builder API for simulation configuration *****************************/

    /// Set seed for random number generator
    pub fn seed_from_u64(&mut self, seed: u64) -> &mut Self {
        self.state.rng = SmallRng::seed_from_u64(seed);
        return self;
    }

    /// Set seed for random number generator
    pub fn seed_from_string(&mut self, data: &str) -> &mut Self {
        let mut seed = <SmallRng as SeedableRng>::Seed::default();
        let dest = seed.as_mut();
        let src = data.as_bytes();

        for i in 0..(dest.len().min(src.len())) {
            dest[i] += src[i];
        }

        self.state.rng = SmallRng::from_seed(seed);
        return self;
    }

    /// Set seed for random number generator
    pub fn seed_from(&mut self, rng: &SmallRng) -> &mut Self {
        self.state.rng = rng.clone();
        return self;
    }

    /// Increment the RNG. Useful to avoid repeated runs for methods that do not affect the RNG.
    pub fn rng_next(&mut self) -> &mut Self {
        self.state.rng.gen_bool(0.5);
        return self;
    }

    /// Contaminate n individuals at random.
    ///
    /// If only_susceptible is true, only contaminate susceptible individuals.
    pub fn contaminate_at_random(&mut self, n: usize, only_susceptible: bool) -> &mut Self
    where
        ST: HasEpiModel,
        ST::Clinical: Default,
    {
        self.with_parts(|pop, _, rng| {
            pop.contaminate_at_random(n, only_susceptible, rng);
        });
        return self;
    }

    /*
    /// Initialize simulation and calibrate sampler from a curve of cases.
    ///
    /// This is a somewhat simplistic view on model calibration. We just run
    /// the simulation normally but at each step we recalibrate the sampler
    /// to produce the same number of infections as expected from the epidemic
    /// curve.
    pub fn calibrate_sampler_from_cases(&'a mut self, cases: &[Real]) -> &mut Self
    where
        ST::Clinical: Default,
    {
        // TODO: create calibrator struct
        let alpha = 0.5;
        let min_contacts = 0.0;
        let max_contacts = 10.0;
        let min_scale = 1.0 / 1.5;
        let max_scale = 1.33;
        let e_ratio = 0.25;
        let r = 0.85;

        let mut n_iter = 0;
        let mut excess = 0.0;
        let mut acc_cases = 0.0;
        let mut acc_target = 0.0;
        let mut c_mean = self.sampler.contacts();

        for raw_target in cases {
            n_iter += 1;
            acc_target += raw_target;

            let target = (raw_target + e_ratio * excess).max(0.0);
            let estimate = self
                .sampler
                .expected_infection_pairs(&self.state.population);
            let grow = ((target + alpha) / (estimate + alpha)).clamp(min_scale, max_scale);

            // Calibrate contacts. Other implementations might calibrate different
            // coefficients, but we do not have any way to generalize it yet.
            let c1 = self.sampler.contacts();
            let c2 = (c1 * grow).clamp(min_contacts, max_contacts);
            c_mean = c_mean * r + c2 * (1.0 - r);
            self.sampler.set_contacts(c2);

            // Run and register the number of cases
            let n_cases = self.steps(1);
            acc_cases += n_cases as Real;
            excess = acc_target - acc_cases as Real;

            // If excess is very large (very negative), we might want to create
            // artificial infections to quickstart an infection
            if excess > 0.25 * (acc_target + alpha) {
                let n = (excess * 0.25) as usize;
                self.state
                    .population
                    .contaminate_at_random(n, true, &mut self.state.rng);
                acc_cases += n as Real;
                excess = acc_target - acc_cases as Real;
            }

            trace!(target: "calibrate_sample_cases", "iter {}, coeff: {:.2} ({:.2})\n  - target: {} ({}); cases: {} (~ {:.1}); excess: {}", n_iter, c2, c_mean, raw_target, target, n_cases, estimate, excess);
        }
        self.sampler.set_contacts(c_mean);
        debug!(target: "calibrate_sample_cases", "final contacts: {}, {} iterations", c_mean, n_iter);
        return self;
    }
    */

    /// Like steps, but return Self, rather then the number of cases. This is
    /// useful to use in builder-like APIs.
    #[inline]
    pub fn run(&'a mut self, n_steps: usize) -> &'a mut Self {
        for _ in 0..n_steps {
            self.step();
        }
        return self;
    }

    /// Create n copies of simulation and run them for n_steps in parallel.
    pub fn run_parallel(&self, n: usize, n_steps: usize) -> Vec<Self>
    where
        P: Send + Sync,
        SP: Send + Sync,
        ST: Send + Sync,
    {
        let mut result = self.copies(n);
        result.par_iter_mut().for_each(|sim| {
            sim.run(n_steps);
        });
        return result;
    }

    /** Run simulation *******************************************************/

    /// Run a single simulation step;
    pub fn step(&mut self) -> usize {
        let mut cases = 0;
        let start_time = self.state.time;
        let dispatcher = &mut self.dispatcher;
        let scheduler = &mut self.scheduler;
        let sampler = &self.sampler;
        let state = &mut self.state;
        let cb = |i, j| dispatcher.trigger(&EpidemicSimulationMsg::NewInfection(i, j));

        log::debug!("running step: {}", start_time + 1);
        scheduler.before_step(state);
        cases += state.step(sampler, cb);
        dispatcher.trigger(&EpidemicSimulationMsg::EndStep(start_time + 1, cases));
        scheduler.after_step(state);

        let population = &state.population;
        if let Some(table) = self.epicurves.as_mut() {
            table.count_epidemic_compartments(population, true);
        }

        return cases;
    }

    /// Return a sample of n agents
    ///
    /// This method does not advance the random number generator and thus all samples taken
    /// in succession will be identical. If this behavior is not desired, execute
    /// simulation.rng_next() to increment the RNG.
    pub fn sample(&self, n: usize) -> Vec<ST> {
        let mut sample = Vec::with_capacity(n);
        let mut rng = self.state.rng.clone();

        for (_, ag) in self.state.population.randoms(n, &mut rng) {
            sample.push(ag.clone());
        }
        return sample;
    }

    /// Create n copies of self.
    ///
    /// RNG is initialized from entropy in each copy.
    pub fn copies(&self, n: usize) -> Vec<Self> {
        return (1..n)
            .map(|_| {
                let mut new = self.clone();
                new.seed_from(&mut SmallRng::from_entropy());
                new
            })
            .collect();
    }

    /// Population size
    pub fn count(&self) -> usize {
        self.state.population.len()
    }

    /// Return the tip of the epicurve
    pub fn epistate(&self, normalize: bool) -> Vec<Real> {
        let factor = self._normalization_factor(normalize);
        if let Some(table) = self.epicurves.as_ref() {
            return table.tip().iter().map(|a| *a as Real * factor).collect();
        } else {
            return vec![0.0; ST::CARDINALITY];
        }
    }

    /// Return curve for the n-th component of epicurve.
    ///
    /// If normalized, results are divided by population size.
    pub fn get_epicurve(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let data = self.epicurves.as_ref()?.col(n)?;
        let mut vec = Vec::with_capacity(data.len());
        let factor = self._normalization_factor(normalize);
        for x in data {
            vec.push(x as Real * factor);
        }
        return Some(vec);
    }

    /// Get epistate at a given iteration
    pub fn get_epistate(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let factor = self._normalization_factor(normalize);
        let row = self.epicurves.as_ref()?.row(n)?;
        return Some(row.iter().map(|x| *x as Real * factor).collect());
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self) -> String {
        let mut infections = vec![0];
        if let Some(counts) = self.dispatcher.infection_counts() {
            infections.extend(counts.iter());
        }

        if let Some(table) = &self.epicurves {
            return table
                .clone()
                .add_column("cases", infections.iter().cloned(), true)
                .render_csv(',');
        } else {
            return "".to_string();
        }
    }

    /// Used internally to normalize (or not) results
    fn _normalization_factor(&self, normalize: bool) -> Real {
        if normalize {
            1.0 / self.count() as Real
        } else {
            1.0
        }
    }

    /*
    /// Get epidemiological params for given agent
    ///
    /// Return Some(FullSEIRParams<f64>) if agent exists.
    pub fn get_local_epiparams(&self, i: usize) -> Option<P::LocalParams>
    where
    ST: EpiModel,
    P::LocalParams: EpiParams,
    {
        let ag = self.state.population.get(i)?;
        let params = self.params.local_params(ag);
        return Some(params);
    }
    */

    /// Work with mutable references to the internal population, parameters and RNG.
    pub fn with_parts<R>(&mut self, f: impl FnOnce(&mut Vec<ST>, &mut P, &mut SmallRng) -> R) -> R {
        return f(
            &mut self.state.population,
            &mut self.state.params,
            &mut self.state.rng,
        );
    }

    /// Work with a mutable reference to the internal RNG, parameters and population.
    pub fn with_state_mut<R>(&mut self, f: impl FnOnce(&mut SimulationState<P, ST>) -> R) -> R {
        return f(&mut self.state);
    }

    /// Work with a mutable reference to the internal RNG, parameters and population.
    pub fn with_state<R>(&self, f: impl FnOnce(&SimulationState<P, ST>) -> R) -> R {
        return f(&self.state);
    }
}

impl<'a, P, ST> Simulation<ST, P, SimpleSampler>
where
    P: ParamSet<ST>,
    ST: RandomUpdate<P::BoundParams> + EpiModel + Debug,
    P::BoundParams: EpiParams,
{
    /// Create a new simulation from a simple sampler
    pub fn new_simple(
        params: P,
        population: Vec<ST>,
        n_contacts: Real,
        prob_infection: Real,
    ) -> Self {
        let sampler = SimpleSampler::new(n_contacts, prob_infection);
        return Self::new(params, population, sampler);
    }
}

// impl<W, S, PS> OwnsStateSlice for Simulation<W, S, PS>
// where
//     PS: PopulationSampler<Vec<S>> + Default,
//     W: LocalBind<S> + Default,
//     S: EpiModel + RandomUpdate<W::Local> + Debug,
// {
//     type Elem = S;

//     fn from_agent_vec(agents: Vec<Self::Elem>) -> Self {
//         Simulation::new_simple(W::def, agents, n_contacts, prob_infection)
//     }
//     fn as_state_slice(&self) -> &[S] {
//         self.population.as_slice()
//     }

//     fn as_state_mut_slice(&mut self) -> &mut [S] {
//         self.population.as_mut_slice()
//     }
// }
