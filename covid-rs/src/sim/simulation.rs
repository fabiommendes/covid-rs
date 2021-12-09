use super::{population::Population, state::RandomUpdate, HasEpiModel};
use crate::{
    epidemic::*,
    events::{EpidemicSimulationMsg, EventDispatcher},
    params::{EpiParamsFull, EpiParamsLocalT, FromLocalParams, LocalBind},
    prelude::*,
    trackers::{EpiTracker, Tracker},
};
use getset::{Getters, MutGetters};
use log::{debug, trace};
use rand::{
    prelude::{SeedableRng, SmallRng},
    Rng,
};
use std::{cell::RefCell, fmt::Debug};

/// Simulation stores a population of agents and some objects responsible for
/// controlling the dynamics of those Agents.
#[derive(Getters, MutGetters)]
pub struct Simulation<ST, P, SP> {
    #[getset(get = "pub", get_mut = "pub")]
    population: Vec<ST>,

    #[getset(get = "pub", get_mut = "pub")]
    params: RefCell<P>,

    #[getset(get = "pub", get_mut = "pub")]
    sampler: SP,

    rng: RefCell<SmallRng>,
    dispatcher: EventDispatcher<EpidemicSimulationMsg>,
    reporter: EpiTracker<Vec<ST>>,
    time: Time,
}

impl<'a, ST, P, SP> Simulation<ST, P, SP>
where
    P: LocalBind<ST>,
    P::Local: EpiParamsLocalT,
    ST: EpiModel + RandomUpdate<P::Local> + Debug,
    SP: PopulationSampler<Vec<ST>>,
{
    /// Create new simulation from population and sampler.
    pub fn new(params: P, population: Vec<ST>, sampler: SP) -> Self {
        Simulation {
            reporter: EpiTracker::new(&population),
            population,
            params: RefCell::new(params),
            dispatcher: EventDispatcher::new_with_default_listeners(),
            rng: RefCell::new(SmallRng::from_entropy()),
            time: 0,
            sampler,
        }
    }

    /// Return a copy of simulation ignoring local reporters and update
    /// functions
    pub fn copy(&self) -> Self
    where
        P: Clone,
        SP: Clone,
    {
        Simulation {
            population: self.population.clone(),
            params: self.params.clone(),
            sampler: self.sampler.clone(),
            reporter: self.reporter.copy(),
            rng: self.rng.clone(),
            time: self.time,
            dispatcher: EventDispatcher::new_with_default_listeners(),
        }
    }

    /** Builder API for simulation configuration *****************************/

    /// Set seed for random number generator
    pub fn seed_from_u64(&mut self, seed: u64) -> &mut Self {
        self.rng.replace(SmallRng::seed_from_u64(seed));
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

        self.rng.replace(SmallRng::from_seed(seed));
        return self;
    }

    /// Set seed for random number generator
    pub fn seed_from(&mut self, rng: &SmallRng) -> &mut Self {
        self.rng.replace(rng.clone());
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
        self.with_state(|rng, _, pop| {
            pop.contaminate_at_random(n, only_susceptible, rng);
        });
        return self;
    }

    /// Initialize simulation and calibrate sampler from a curve of cases.
    ///
    /// This is a somewhat simplistic view on model calibration. We just run
    /// the simulation normally but at each step we recalibrate the sampler
    /// to produce the same number of infections as expected from the epidemic
    /// curve.
    pub fn calibrate_sampler_from_cases(&mut self, cases: &[Real]) -> &mut Self
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
            let estimate = self.sampler.expected_infection_pairs(&self.population);
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
                self.population
                    .contaminate_at_random(n, true, &mut *self.rng.borrow_mut());
                acc_cases += n as Real;
                excess = acc_target - acc_cases as Real;
            }

            trace!(target: "calibrate_sample_cases", "iter {}, coeff: {:.2} ({:.2})\n  - target: {} ({}); cases: {} (~ {:.1}); excess: {}", n_iter, c2, c_mean, raw_target, target, n_cases, estimate, excess);
        }
        self.sampler.set_contacts(c_mean);
        debug!(target: "calibrate_sample_cases", "final contacts: {}, {} iterations", c_mean, n_iter);
        return self;
    }

    /// Like steps, but return Self, rather then the number of cases. This is
    /// useful to use in builder-like APIs.
    #[inline]
    pub fn run(&mut self, n_steps: usize) -> &mut Self {
        self.steps(n_steps);
        return self;
    }

    /** Run simulation *******************************************************/

    /// Run simulation for the given number of steps and return the number of
    /// new cases.
    pub fn steps(&mut self, n_steps: usize) -> usize {
        let mut cases = 0;
        for _ in 0..n_steps {
            // Default updates
            self.time += 1;
            self.update_agents();
            cases += self.update_pairs();
            self.reporter.track(&self.population);
        }

        return cases;
    }

    /// Self-update agents. Resolve the natural evolution of all agents.
    fn update_agents(&mut self) {
        self.with_state(|rng, params, population| {
            for obj in population.iter_mut() {
                params.bind_to_object(obj);
                obj.random_update(params.local(), rng);
            }
        })
    }

    /// Simulate agent interactions, allowing new infections to occur.
    fn update_pairs(&mut self) -> usize {
        let mut cases = 0usize;
        let rng = &mut *self.rng.borrow_mut();
        let params = &mut *self.params.borrow_mut();

        for (i, j) in self.sampler.sample_infection_pairs(&self.population, rng) {
            if i == j {
                continue;
            }
            if let Some((src, dest)) = self.population.get_pair_mut(i, j) {
                params.bind_to_object(dest);

                if !rng.gen_bool(params.local().prob_protect()) && dest.contaminate_from(src) {
                    cases += 1;
                    self.dispatcher
                        .trigger(&EpidemicSimulationMsg::NewInfection(i, j));
                }
            }
        }

        self.dispatcher
            .trigger(&EpidemicSimulationMsg::EndStep(self.time, cases));

        return cases;
    }

    /// Return a sample of n agents
    pub fn sample(&self, n: usize) -> Vec<ST> {
        let rng = &mut *self.rng.borrow_mut();
        let mut sample = Vec::with_capacity(n);
        for (_, ag) in self.population.randoms(n, rng) {
            sample.push(ag.clone());
        }
        return sample;
    }

    /// Population size
    pub fn count(&self) -> usize {
        self.population.len()
    }

    /// Return the tip of the epicurve
    pub fn epistate(&self, normalize: bool) -> Vec<Real> {
        let factor = self._normalization_factor(normalize);
        self.reporter
            .tip()
            .iter()
            .map(|a| *a as Real * factor)
            .collect()
    }

    /// Return curve for the n-th component of epicurve.
    ///
    /// If normalized, results are divided by population size.
    pub fn get_epicurve(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        self.reporter.col(n).map(|data| {
            let mut vec = Vec::with_capacity(data.len());
            let factor = self._normalization_factor(normalize);
            for x in data {
                vec.push(x as Real * factor);
            }
            return vec;
        })
    }

    /// Get epistate at a given iteration
    pub fn get_epistate(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let factor = self._normalization_factor(normalize);
        let row = self.reporter.row(n)?;
        return Some(row.iter().map(|x| *x as Real * factor).collect());
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self) -> String {
        let mut infections = vec![0];
        if let Some(counts) = self.dispatcher.infection_counts() {
            infections.extend(counts.iter());
        }
        return self
            .reporter
            .epicurves()
            .clone()
            .add_column("cases", infections.iter().cloned(), true)
            .render_csv( ',');
    }

    /// Used internally to normalize (or not) results
    fn _normalization_factor(&self, normalize: bool) -> Real {
        if normalize {
            1.0 / self.count() as Real
        } else {
            1.0
        }
    }

    /// Get epidemiological params for given agent
    ///
    /// Return Some(FullSEIRParams<f64>) if agent exists.
    pub fn get_local_epiparams(&self, i: usize) -> Option<EpiParamsFull<f64>>
    where
        ST: EpiModel,
        P::Local: EpiParamsLocalT,
    {
        let ag = self.population.get(i)?;
        let mut params = self.params.borrow_mut();
        params.bind_to_object(ag);
        Some(FromLocalParams::from_local_params(params.local()))
    }

    /// Work with mutable references to the internal RNG and population.
    pub fn with_rng_population<R>(
        &mut self,
        f: impl FnOnce(&mut SmallRng, &mut Vec<ST>) -> R,
    ) -> R {
        let rng = &mut *self.rng.borrow_mut();
        f(rng, &mut self.population)
    }

    /// Work with a mutable reference to the internal RNG, parameters and population.
    pub fn with_state<R>(&mut self, f: impl FnOnce(&mut SmallRng, &mut P, &mut Vec<ST>) -> R) -> R {
        let rng = &mut *self.rng.borrow_mut();
        let params = &mut *self.params.borrow_mut();
        return f(rng, params, &mut self.population);
    }
}

impl<P, ST> Simulation<ST, P, SimpleSampler>
where
    P: LocalBind<ST>,
    ST: RandomUpdate<P::Local> + EpiModel + Debug,
    P::Local: EpiParamsLocalT,
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
