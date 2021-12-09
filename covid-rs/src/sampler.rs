use crate::{
    prelude::{EpiModel, Real},
    sim::{HasAge, Population},
};
use getset::*;
use ndarray::prelude::*;
use rand::prelude::*;

pub trait Sampler {
    /// Baseline probability of infection. Different samplers may interpret this
    /// value in slightly different ways. This is a global probability that
    /// can be used to tweak other infection probabilities for specific
    /// contacts.
    fn prob_infection(&self) -> Real;

    /// Set prob infection to value.
    fn set_prob_infection(&mut self, value: Real) -> &mut Self;

    /// Average number of contacts per individual per day.
    fn contacts(&self) -> Real;

    /// Recalibrate to obtain a new average number of contacts.
    fn set_contacts(&mut self, value: Real) -> &mut Self;
}

/// The sampler trait defines how new pairs of infected individuals are selected
/// from population.
pub trait PopulationSampler<P>: Sampler
where
    P: Population,
{
    /// Receive a list of agents and return the vector of indexes of all new
    /// infections ocurred in a simulation step.
    ///
    /// This function is called once per step in a simulation and if the sampler
    /// requires any state  
    fn sample_infection_pairs(&self, population: &P, rng: &mut impl Rng) -> Vec<(usize, usize)>;

    /// Return the expected number of infection pairs for population.
    fn expected_infection_pairs(&self, population: &P) -> Real {
        let mut rng = SmallRng::from_entropy();
        self.sample_infection_pairs(population, &mut rng).len() as Real
    }

    /// Update any necessary internal state from the initial list of agents.
    /// This is called everytime the sampler is registered in a simulation.
    /// The sampler may modify  
    fn init(&mut self, _population: &mut P) {}

    /// Update population of EpiModels by sampling pairs and than contaminating
    /// each pair using the model.contaminate_from() method.
    ///
    /// Return the number of successful new infections.
    fn update_epimodel_population(&self, population: &mut P, rng: &mut impl Rng) -> usize
    where
        P::State: EpiModel,
    {
        let mut cases = 0;
        self.update_epimodel_population_with(population, rng, |_, src, _, dest| {
            // trace!(target: "sampler", "contamination pair: (from: {:?}, to: {:?})", src, dest);
            if dest.contaminate_from(src) {
                cases += 1
            }
        });
        return cases;
    }

    /// A more versatile version of update_epimodel_population which calls a
    /// closure to decide how to proceed with contamination of each possible
    /// infection pairs.
    fn update_epimodel_population_with(
        &self,
        population: &mut P,
        rng: &mut impl Rng,
        f: impl FnMut(usize, &mut P::State, usize, &mut P::State),
    ) where
        P::State: EpiModel,
    {
        let mut g = f;

        for (i, j) in self.sample_infection_pairs(population, rng) {
            if i == j {
                continue;
            }
            if let Some((src, dest)) = population.get_pair_mut(i, j) {
                g(i, src, j, dest)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// CONCRETE IMPLEMENTATIONS
////////////////////////////////////////////////////////////////////////////////

/// A NoOP sampler. This is actually just a type alias to unit.
pub type NoOpSampler = ();

impl Sampler for NoOpSampler {
    fn prob_infection(&self) -> Real {
        0.0
    }

    fn set_prob_infection(&mut self, _value: Real) -> &mut Self {
        return self;
    }

    fn contacts(&self) -> Real {
        0.0
    }

    fn set_contacts(&mut self, _: Real) -> &mut Self {
        self
    }
}

impl<P: Population> PopulationSampler<P> for NoOpSampler {
    fn sample_infection_pairs(&self, _pool: &P, _rng: &mut impl Rng) -> Vec<(usize, usize)> {
        vec![]
    }
}

/// A simple sampling strategy that picks up a fixed number of contacts per
/// infectious individual and infect randomly in population using the given
/// probability of infection.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct SimpleSampler {
    contacts: Real,
    prob_infection: Real,
}

impl SimpleSampler {
    pub fn new(contacts: Real, prob_infection: Real) -> Self {
        SimpleSampler {
            contacts,
            prob_infection,
        }
    }

    fn each_infection_pair<P, R, F>(&self, pop: &P, rng: &mut R, f: F)
    where
        F: FnMut(usize, usize),
        R: Rng,
        P: Population,
        P::State: EpiModel,
    {
        let n = pop.count();
        let mut action = f;

        pop.each_agent(&mut |i, st| {
            let odds = st.contagion_odds();
            if odds > 0.0 {
                let mut m = round_probabilistically(self.contacts, rng);
                while m > 0 {
                    if rng.gen_bool((self.prob_infection * odds).min(1.0)) {
                        let j = rng.gen_range(0..n);
                        if i == j {
                            continue;
                        } else if pop.map_agent(j, |ag| ag.is_susceptible()) == Some(true) {
                            action(i, j);
                        }
                    }
                    m -= 1;
                }
            }
        });
    }
}

impl Sampler for SimpleSampler {
    fn prob_infection(&self) -> Real {
        self.prob_infection
    }

    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        self.prob_infection = value;
        return self;
    }

    fn contacts(&self) -> Real {
        self.contacts
    }

    fn set_contacts(&mut self, value: Real) -> &mut Self {
        self.contacts = value;
        return self;
    }
}

impl<P> PopulationSampler<P> for SimpleSampler
where
    P: Population,
    P::State: EpiModel,
{
    fn sample_infection_pairs(&self, pop: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        self.each_infection_pair(pop, rng, |i, j| pairs.push((i, j)));
        return pairs;
    }

    fn expected_infection_pairs(&self, population: &P) -> Real {
        use crate::utils::default_rng;

        let mut total = 0;
        let mut rng = default_rng();
        self.each_infection_pair(population, &mut rng, |_, _| total += 1);
        return total as Real;
    }

    // fn expected_infection_pairs(&self, pop: &P) -> Real {
    //     let mut total = 0.0;
    //     let prob = self.prob_infection;
    //     let mut s = 0;

    //     pop.each_agent(&mut |_, st| {
    //         let odds = st.contagion_odds();
    //         s += st.is_susceptible() as usize;
    //         total += (prob * odds).min(1.0);
    //     });

    //     return total * self.contacts * (s as Real / pop.count() as Real);
    // }
}

/// A simple sampling strategy that picks up a fixed number of contacts per
/// infectious individual and infect randomly in population using the given
/// probability of infection.
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters, Setters)]
pub struct ContactMatrixSampler {
    /// Age range in each bin. Usually 10 years.
    age_range: u8,

    /// Store the index of all agents in each age group.
    ///
    /// age_groups = vec![  // list of vectors
    ///      vec![1, 3, 10, 20, 50, ...],  // individuals in age group 0
    ///      vec![2, 6, 11, 19, 30, ...],  // individuals in age group 1
    ///      vec![0, 5, 21, 51, 99, ...],  // individuals in age group 2
    ///      ...
    /// ];
    ///
    /// The vectors do not have the same length and the number of vectors is
    /// equal to the number of age groups.
    age_groups: Vec<Vec<usize>>,

    /// The contact matrix C[i, j] determines the average number of daily
    /// contacts an individual in age group i does with an individual in age
    /// group j.
    #[getset(get = "pub", set = "pub")]
    contact_matrix: Array2<Real>,
    n_contacts: Real,

    /// Probability of infection for a single contact
    prob_infection: Real,
}

impl ContactMatrixSampler {
    pub fn new(bin_size: u8, contact_matrix: Array2<Real>, prob_infection: Real) -> Self {
        assert!(contact_matrix.is_square(), "Contact matrix must be square");
        ContactMatrixSampler {
            age_range: bin_size,
            contact_matrix,
            prob_infection,
            age_groups: vec![],
            n_contacts: 0.0,
        }
    }

    pub fn n_bins(&self) -> usize {
        self.contact_matrix.nrows()
    }

    /// Return the average number of contacts per individual
    fn n_contacts(&self) -> Real {
        let mut count = 0.0;
        let pop_sizes: Vec<_> = self.age_groups.iter().map(|v| v.len() as Real).collect();
        for i in 0..self.contact_matrix.nrows() {
            let num_i = pop_sizes[i];
            for j in 0..self.contact_matrix.ncols() {
                let n = self.contact_matrix[(i, j)];
                count += n * num_i;
            }
        }
        return count / pop_sizes.iter().fold(0.0, |x, y| x + y);
    }

    fn age_group(&self, age: u8) -> usize {
        Self::age_group_static(age, self.age_range, self.n_bins())
    }

    fn age_group_static(age: u8, bin_size: u8, n: usize) -> usize {
        ((age / bin_size) as usize).clamp(0, n - 1)
    }
}

impl Sampler for ContactMatrixSampler {
    fn prob_infection(&self) -> Real {
        self.prob_infection
    }

    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        self.prob_infection = value;
        return self;
    }

    fn contacts(&self) -> Real {
        self.n_contacts
    }

    fn set_contacts(&mut self, value: Real) -> &mut Self {
        let ratio = value / self.n_contacts();
        self.contact_matrix *= ratio;
        self.n_contacts = value;
        return self;
    }
}

impl<P> PopulationSampler<P> for ContactMatrixSampler
where
    P: Population,
    P::State: HasAge + EpiModel,
{
    fn init(&mut self, pop: &mut P) {
        let nbins = self.n_bins();
        let bin_size = self.age_range;

        for _ in 0..(255 / self.age_range) {
            self.age_groups.push(vec![]);
        }

        pop.each_agent(&mut |i, st| {
            let k = Self::age_group_static(st.age(), bin_size, nbins);
            self.age_groups[k].push(i);
        });

        self.n_contacts = self.n_contacts();
    }

    fn sample_infection_pairs(&self, pop: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        pop.each_agent(&mut |i, st| {
            let odds = st.contagion_odds();
            if odds > 0.0 {
                let u = self.age_group(st.age());
                for v in 0..self.n_bins() {
                    let mut m = round_probabilistically(self.contact_matrix[(u, v)], rng);
                    let group = &self.age_groups[v];
                    while m > 0 {
                        if rng.gen_bool((self.prob_infection * odds).min(1.0)) {
                            let j = group[rng.gen_range(0..group.len())];
                            if i == j {
                                continue;
                            }
                            pairs.push((i, j));
                        }
                        m -= 1;
                    }
                }
            }
        });
        return pairs;
    }

    fn expected_infection_pairs(&self, population: &P) -> Real {
        let sampler = SimpleSampler::new(self.n_contacts(), self.prob_infection);
        return sampler.expected_infection_pairs(population);
    }
}

fn round_probabilistically(f: Real, rng: &mut impl Rng) -> usize {
    let int = f as usize;
    if rng.gen_bool(f - (int as Real)) {
        return int + 1;
    }
    return int;
}

/// TODO: impl PythonSampler and use dyn to make this go away!
#[derive(Debug, Clone, PartialEq)]
pub enum AnySampler {
    Simple(SimpleSampler),
    ContactMatrix(ContactMatrixSampler),
}

impl Sampler for AnySampler {
    fn prob_infection(&self) -> Real {
        match self {
            AnySampler::Simple(s) => s.prob_infection(),
            AnySampler::ContactMatrix(s) => s.prob_infection(),
        }
    }

    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        match *self {
            AnySampler::Simple(ref mut s) => {
                s.set_prob_infection(value);
            }
            AnySampler::ContactMatrix(ref mut s) => {
                s.set_prob_infection(value);
            }
        }
        return self;
    }

    fn contacts(&self) -> Real {
        match self {
            AnySampler::Simple(s) => s.contacts(),
            AnySampler::ContactMatrix(s) => s.contacts(),
        }
    }

    fn set_contacts(&mut self, value: Real) -> &mut Self {
        match self {
            AnySampler::Simple(s) => {
                s.set_contacts(value);
            }
            AnySampler::ContactMatrix(s) => {
                s.set_contacts(value);
            }
        };
        return self;
    }
}

impl<P> PopulationSampler<P> for AnySampler
where
    P: Population,
    P::State: HasAge + EpiModel,
{
    fn sample_infection_pairs(&self, pool: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        match self {
            AnySampler::Simple(s) => s.sample_infection_pairs(pool, rng),
            AnySampler::ContactMatrix(s) => s.sample_infection_pairs(pool, rng),
        }
    }

    fn init(&mut self, pool: &mut P) {
        match self {
            AnySampler::Simple(s) => s.init(pool),
            AnySampler::ContactMatrix(s) => s.init(pool),
        }
    }
}

impl From<SimpleSampler> for AnySampler {
    fn from(sampler: SimpleSampler) -> AnySampler {
        AnySampler::Simple(sampler)
    }
}

impl From<ContactMatrixSampler> for AnySampler {
    fn from(sampler: ContactMatrixSampler) -> AnySampler {
        AnySampler::ContactMatrix(sampler)
    }
}
