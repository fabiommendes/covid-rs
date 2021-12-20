use std::error::Error;
use std::fs;
use std::path::Path;

// use crate::params::{BindVaccine, EpiParamsBindVaccine, EpiParamsCached, VaccineParams};
use crate::{
    models::{SimpleAgent, SimpleAgentPopulationExt, SEICHAR},
    params::{EpiParamsData},
    prelude::*,
    sim::{HasAge, Population, Simulation},
    utils::default_rng,
};
use chrono::{DateTime, Local};
use getset::{CopyGetters, Getters, Setters};
use rand::Rng;
use serde::{Deserialize, Serialize};

type Params = EpiParamsData<Real>;

/**
 *  Supported simulation types
 */
pub type SeicharAgent = SimpleAgent<SEICHAR<()>, ()>;
pub type WorldParams = EpiParamsData<Real>; //BindVaccine<EpiParamsCached<EpiParamsFull<AgeParam>, AgeParam>>;
pub type Sampler = SimpleSampler;
pub type SeicharSimulation = Simulation<SeicharAgent, WorldParams, Sampler>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Epicurve {
    data: Vec<Real>,
}

/// Enumeration that describes the vaccination strategy used to initialize population.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum VaccinePlan {
    NoVaccine,
    ByChance(Real),
    ByMinAge(Real, Age),
    ByMaxAge(Real, Age),
    ByAgeRange(Real, Age, Age),
}

impl Default for VaccinePlan {
    fn default() -> Self {
        VaccinePlan::NoVaccine
    }
}

impl VaccinePlan {
    /// Vaccinate population with the given vaccine according to the vaccination plan.
    pub fn vaccinate_population<P, M, V>(&self, pop: &mut P, vaccine: V, rng: &mut impl Rng)
    where
        P: SimpleAgentPopulationExt<M, V>,
        V: Clone,
    {
        match self {
            VaccinePlan::NoVaccine => {
                return;
            }
            &VaccinePlan::ByChance(prob) => {
                pop.vaccinate_random(vaccine, prob, rng);
            }
            &VaccinePlan::ByMinAge(prob, age) => {
                pop.vaccinate_random_if(vaccine, prob, rng, |ag| ag.age() >= age);
            }
            &VaccinePlan::ByMaxAge(prob, age) => {
                pop.vaccinate_random_if(vaccine, prob, rng, |ag| ag.age() <= age);
            }
            &VaccinePlan::ByAgeRange(prob, min, max) => {
                pop.vaccinate_random_if(vaccine, prob, rng, |ag| {
                    ag.age() >= min && ag.age() <= max
                });
            }
        }
    }
}

/// Struct that holds configuration data to initialize a simulation.
///
/// It uses Serde to persist configuration in TOML files.
#[derive(Deserialize, Serialize, Debug, Clone, Getters, CopyGetters, Setters)]
#[serde(default)]
pub struct Config {
    // Generic options
    #[getset(get = "pub")]
    name: String,

    #[getset(get_copy = "pub")]
    verbose: bool,

    seed: Option<String>,

    #[getset(get_copy = "pub")]
    timestamp: DateTime<Local>,

    #[getset(get_copy = "pub", set = "pub")]
    write_outputs: bool,

    // Population and age distribution
    #[getset(set = "pub")]
    pop_size: usize,

    #[getset(set = "pub")]
    age_distribution: Option<AgeDistribution10>,

    #[getset(get_copy = "pub", set = "pub")]
    initial_infections: usize,

    #[getset(get_copy = "pub", set = "pub")]
    n_contacts: Real,

    #[getset(get_copy = "pub", set = "pub")]
    prob_infection: Real,

    #[getset(get_copy = "pub", set = "pub")]
    max_iter: usize,

    params: Option<Params>,
    epicurve: Option<Epicurve>,

    #[getset(get = "pub", set = "pub")]
    vaccine_plan: VaccinePlan,
}

impl Config {
    /// Return the population size.
    ///
    /// Population size can be given implicitly from age distribution or explicitly
    /// via a parameter.
    pub fn pop_size(&self) -> usize {
        if self.pop_size != 0 {
            return self.pop_size;
        } else if let Some(distrib) = self.age_distribution {
            let sum: usize = distrib.iter().fold(0.0, |x, &y| x + y) as usize;
            if sum > 0 {
                return sum;
            }
        }
        return 1_000;
    }

    /// Return the normalized age_distribution.
    pub fn age_distribution(&self) -> AgeDistribution10 {
        if let Some(mut distrib) = self.age_distribution {
            let sum: Real = distrib.iter().sum();
            distrib.iter_mut().for_each(|x| *x = (*x) / sum);
            return distrib;
        }
        return [1.0 / 9.0; 9];
    }

    /// Update empty names using the configuration file path.
    pub fn update_name_from_path(&mut self, path: &str) {
        if !self.name.len() == 0 {
            let a = path.rfind("/").unwrap_or(0);
            let b = if path.ends_with(".toml") {
                path.rfind(".").unwrap_or(path.len())
            } else {
                path.len()
            };
            self.name.push_str(&path[a..b]);
        }
    }

    /// Return the output folder name
    pub fn output_folder_name(&self) -> String {
        return format!("{}-{}", self.name, self.timestamp).replace(" ", "-");
    }

    /// Return the output folder name and initialize it with
    /// the configuration file at conf.toml.
    pub fn create_output_folder(&self) -> Result<(), Box<dyn Error>> {
        if self.write_outputs {
            let path = self.output_folder_name();
            let conf_data = toml::to_string(self)?;

            fs::create_dir(&path)?;
            return self.write_ouptut("conf.toml", &conf_data);
        }
        return Ok(());
    }

    /// Return the output folder name and initialize it with
    /// the configuration file at conf.toml.
    pub fn write_ouptut(&self, path: &str, data: &str) -> Result<(), Box<dyn Error>> {
        if self.write_outputs {
            let base = self.output_folder_name();
            let dest = Path::new(&base).join(path);

            fs::write(&dest, data)?;
        }
        return Ok(());
    }

    /// Return the expected age frequencies. The sum of all frequencies is always
    /// equal to the pop_size().
    pub fn age_counts(&self) -> AgeCount10 {
        let mut n = self.pop_size() as u32;
        let mut fcounts = self.age_distribution().map(|x| x * n as Real);
        let mut counts = fcounts.map(|x| x as u32);

        for (i, &y) in counts.iter().enumerate() {
            fcounts[i] -= y as Real;
            n -= y;
        }

        while n > 0 {
            let i = {
                let mut argmax = 0;
                let mut maxvalue = fcounts[0];

                for (i, &x) in fcounts.iter().enumerate() {
                    if x > maxvalue {
                        maxvalue = x;
                        argmax = i;
                    }
                }
                argmax
            };
            fcounts[i] = 0.0;
            counts[i] += 1;
            n -= 1;
        }

        return counts;
    }

    /// Return the seed for the RNG
    pub fn seed(&self) -> &Option<String> {
        &self.seed
    }

    /// Set random seed string
    pub fn set_seed(&mut self, seed: &str) {
        if seed.is_empty() {
            self.seed = None;
        } else {
            self.seed = Some(seed.to_string());
        }
    }

    /// Return a sampler from configuration
    pub fn simple_sampler(&self) -> SimpleSampler {
        SimpleSampler::new(self.n_contacts, self.prob_infection)
    }

    /// Return a population vector from configuration
    pub fn population<R, T>(&self, rng: &mut R) -> Vec<T>
    where
        R: Rng,
        T: Default + Clone + HasAge,
    {
        let mut pop: Vec<T> = Population::from_default(self.pop_size());
        pop.set_age_counts(self.age_counts(), rng);
        return pop;
    }

    pub fn seichar_simulation(&self) -> SeicharSimulation {
        let mut rng = default_rng();
        let population: Vec<SeicharAgent> = self.population(&mut rng);
        let sampler = self.simple_sampler();

        // Initialize simulation
        // let params: EpiParamsMin<Real> = self.params.unwrap_or_default().cached().into();
        // let params: EpiParamsMin<Real> = self.params.unwrap_or_default().into();
        let params: EpiParamsData<Real> = EpiParamsData::default_from_scalars();
        let mut sim: Simulation<_, _, _> = Simulation::new(params, population, sampler);
        println!("{:?}", params);

        if let Some(seed) = &self.seed {
            sim.seed_from_string(seed);
        }

        // Configure simulation
        return sim;
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "".to_string(),
            verbose: true,
            timestamp: Local::now(),
            write_outputs: true,
            seed: None,

            pop_size: 0,
            age_distribution: Some([1.0; 9]),

            initial_infections: 1,
            n_contacts: 4.5,
            prob_infection: 0.1,
            max_iter: 30,
            params: Some(Default::default()),
            epicurve: None,
            vaccine_plan: Default::default(),
        }
    }
}
