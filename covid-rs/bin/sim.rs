use std::fs;

use covid::{
    params::{EpiParamsFull},
    prelude::*,
    sim::*,
    utils::*,
};
use csv::*;
use serde::{Deserialize, Serialize};

type Params = EpiParamsFull<AgeParam>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Epicurve {
    data: Vec<Real>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pop_size: usize,
    initial_infections: usize,
    n_contacts: Real,
    prob_infection: Real,
    num_iter: usize,
    verbose: bool,
    params: Option<Params>,
    pop_distrib: Option<AgeDistribution10>,
    pop_counts: Option<AgeCount10>,
    epicurve: Option<Epicurve>,
}

impl Config {
    pub fn write_data(&self, data: &String, name: &str) {
        fs::write(name, data).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pop_size: 1_000,
            initial_infections: 10,
            n_contacts: 4.5,
            prob_infection: 0.1,
            num_iter: 30,
            verbose: true,
            params: Some(Default::default()),
            pop_distrib: Some([1.0; 9]),
            pop_counts: None,
            epicurve: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TableRow {
    age: Age,
    cfr: Real,
    ifr: Real,
    severe: Real,
    critical: Real,
}

pub fn read_params_table(path: &str, params: &mut Params) -> Result<()> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut prob_severe: AgeDistribution10 = [0.0; 9];
    let mut prob_asymptomatic: AgeDistribution10 = [0.0; 9];
    let mut prob_critical: AgeDistribution10 = [0.0; 9];
    let mut cfr: AgeDistribution10 = [0.0; 9];

    for (i, res) in reader.deserialize().enumerate() {
        let row: TableRow = res?;
        prob_asymptomatic[i] = 1.0 - row.ifr / row.cfr;
        prob_severe[i] = row.severe;
        prob_critical[i] = row.critical / row.severe;
        cfr[i] = row.cfr;
    }

    params
        .epidemic
        .set_prob_asymptomatic(prob_asymptomatic.into());
    params.clinical.set_prob_severe(prob_severe.into());
    params.clinical.set_prob_critical(prob_critical.into());
    params.epidemic.set_case_fatality_ratio(cfr.into());
    return Ok(());
}

pub fn main() {
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let cfg_data = fs::read_to_string("conf.toml").unwrap();
    let mut cfg: Config = toml::from_str(&cfg_data).unwrap();
    let mut params = cfg.params.unwrap_or_default();

    match read_params_table("params.csv", &mut params) {
        Ok(_) => {
            println!("Using distributions from params.csv");
        }
        _ => {}
    }
    cfg.params = Some(params);

    if cfg.verbose {
        println!("{:#?}", cfg);
    }
    simple_simulation(cfg);
}

pub fn simple_simulation(cfg: Config) {
    /*
    type T = SimpleAgent<SEICHAR<()>, bool>;
    let sampler = SimpleSampler::new(cfg.n_contacts, cfg.prob_infection);
    let population: Vec<T>;
    let mut rng = default_rng();

    // Should we construct from distribution or pop_counts?
    if let Some(ns) = cfg.pop_counts {
        population = new_population_from_ages(ns, &mut rng);
    } else if let Some(distrib) = cfg.pop_distrib {
        population = new_population_from_distribution(cfg.pop_size, distrib, &mut rng);
    } else {
        population = new_population(cfg.pop_size);
    }

    // Initialize simulation
    let params: EpiParamsBindVaccine<AgeParam> = cfg.params.unwrap_or_default().cached().into();
    let mut sim: Simulation<_, _, _> = Simulation::new(params, population, sampler);

    // Should we infect from epicurve?
    if let Some(epi) = cfg.epicurve.clone() {
        sim.calibrate_sampler_from_cases(epi.data.as_slice());
        println!("{:?}", sim.epistate(true));
        println!("{:?}", sim.epistate(false));
    } else {
        sim.contaminate_at_random(cfg.initial_infections, &mut rng);
    }

    // Configure simulation
    sim.run(cfg.num_iter);

    // Write output
    let csv = sim.render_epicurve_csv(T::CSV_HEADER);
    cfg.write_data(&csv, "epicurve.csv");
    println!("{}", &csv);
    */
}
