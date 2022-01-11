// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
extern crate clap;
use clap::{App, Arg, ArgMatches};
use covid::{
    config::{Config, SeicharSimulation, VaccinePlan},
    prelude::{Age, Real},
    utils::plot_vbars,
};
use simple_logger::SimpleLogger;
use std::{error::Error, fs};
use toml;

type Fallible<T> = Result<T, Box<dyn Error>>;

fn main() -> Fallible<()> {
    SimpleLogger::new().init().unwrap();

    let matches = App::new("Covid-rs")
        .version("0.1")
        .author("Fábio Mendes <fabiomacedomendes@gmail.com>")
        .about("Simulate epidemiological scenarios for COVID-19 and other diseases.")
        .arg(
            Arg::with_name("config")
                .help("Sets a custom config file")
                .required(false)
                .index(1),
        )
        // Generic options
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Sets a custom simulation name")
                .takes_value(true),
        )
        // Simulation parameters
        .arg(
            Arg::with_name("pop_size")
                .short("s")
                .long("pop-size")
                .help("Change the population size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dry_run")
                .short("d")
                .long("dry-run")
                .help("Run simulation without writing output files")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("max_iter")
                .short("m")
                .long("max-iter")
                .help("Maximum number of iterations")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("infections")
                .short("i")
                .long("infections")
                .help("Start simulation infecting this given number of individuals")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("prob_infections")
                .long("prob-infections")
                .help("Probability of a single infection")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_contacts")
                .long("n-contacts")
                .help("Average number of contacts in the simple sampler")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vaccine_age")
                .long("vaccine-age")
                .help("Age range used to apply vaccine")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vaccine_coverage")
                .long("vaccine-coverage")
                .help("Probability of vaccinated population")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .long("seed")
                .help("Random string that initializes the RNG")
                .takes_value(true),
        )
        .get_matches();

    // Read configuration
    let (config_path, has_config) = if let Some(path) = matches.value_of("config") {
        (path, true)
    } else {
        ("settings.toml", false)
    };

    let mut config: Config = if let Ok(data) = fs::read_to_string(config_path) {
        toml::from_str(&data)?
    } else if has_config {
        Err(format!("config file '{}' not found!", config_path))?
    } else {
        Config::default()
    };

    update_configuration(&matches, &mut config, config_path, has_config)?;
    config.create_output_folder()?;
    println!("foo");

    // Initialize according to params
    let mut sim = config.seichar_simulation();
    log::info!("SEICHAR simulation started");

    initialize_simulation(&mut sim, &config)?;
    run_simulation(&mut sim, &config)?;
    save_simulation_results(&sim, &config)?;

    return Ok(());
}

fn update_configuration(
    matches: &ArgMatches,
    config: &mut Config,
    config_path: &str,
    has_config: bool,
) -> Fallible<()> {
    // name
    let name = matches.value_of("name");
    if let Some(name) = name {
        config.set_name(name.to_string());
    } else if has_config {
        config.update_name_from_path(config_path);
    } else {
        config.set_name("sim".to_string());
    }
    log::debug!("name: {:?}", config.name());

    // write_outputs
    config.set_write_outputs(!matches.is_present("dry_run"));

    // max_iter
    if let Some(max_iter) = matches.value_of("max_iter").map(|x| x.parse()) {
        config.set_max_iter(max_iter.map_err(|_| "'max_iter' must be an integer")?);
    }

    // pop_size
    if let Some(pop_size) = matches.value_of("pop_size").map(|x| x.parse()) {
        config.set_pop_size(pop_size.map_err(|_| "'pop_size' must be an integer")?);
    }

    // seed
    if let Some(seed) = matches.value_of("seed") {
        config.set_seed(seed);
    }

    // Preparing parameters ///////////////////////////////////////////////////

    // initial_infections
    if let Some(infections) = matches.value_of("infections").map(|x| x.parse()) {
        config.set_initial_infections(infections.map_err(|_| "'infections' must be an integer")?);
    }

    // prob_infections
    if let Some(p) = matches.value_of("prob_infections").map(|x| x.parse()) {
        config.set_prob_infection(p.map_err(|_| "'p' must be float between 0 and 1")?);
    }

    // initial_infections
    if let Some(n) = matches.value_of("n_contacts").map(|x| x.parse()) {
        config.set_n_contacts(n.map_err(|_| "'n_contacts' must be a positive float")?);
    }

    // vaccine_plan
    {
        let prob: Real = matches
            .value_of("vaccine_coverage")
            .unwrap_or("1.0")
            .parse()?;

        if let Some(age_range) = matches.value_of("vaccine_age") {
            if age_range.ends_with("+") {
                let min_age: Age = age_range
                    .strip_suffix("+")
                    .unwrap()
                    .parse()
                    .map_err(|_| "Age must be an integer")?;
                config.set_vaccine_plan(VaccinePlan::ByMinAge { prob, min_age });
            } else if age_range.ends_with("-") {
                let max_age: Age = age_range
                    .strip_suffix("-")
                    .unwrap()
                    .parse()
                    .map_err(|_| "Age must be an integer")?;
                config.set_vaccine_plan(VaccinePlan::ByMaxAge { prob, max_age });
            } else if let Some((astr, bstr)) = age_range.split_once('-') {
                let min_age: Age = astr.parse()?;
                let max_age: Age = bstr.parse()?;
                config.set_vaccine_plan(VaccinePlan::ByAgeRange {
                    prob,
                    min_age,
                    max_age,
                });
            } else {
                println!("invalid range for 'vaccine_age': {}", age_range);
                panic!();
            }
        } else {
            config.set_vaccine_plan(VaccinePlan::ByChance { prob });
        }
    }

    // finish
    log::info!("Config file successfully loaded");
    Ok(())
}

fn initialize_simulation(sim: &mut SeicharSimulation, config: &Config) -> Fallible<()> {
    // apply vaccines
    sim.with_state_mut(|st| {
        config.vaccine_plan().vaccinate_population(
            &mut st.population,
            Default::default(),
            &mut st.rng,
        )
    });

    // infect seed
    if config.initial_infections() != 0 {
        sim.contaminate_at_random(config.initial_infections(), true);
    }

    Ok(())
}

fn run_simulation(sim: &mut SeicharSimulation, config: &Config) -> Fallible<()> {
    // Execute simulation plan
    sim.run(config.max_iter());
    log::info!("Run {} iterations of simulation", config.max_iter());
    Ok(())
}

fn save_simulation_results(sim: &SeicharSimulation, config: &Config) -> Fallible<()> {
    // Write output
    let csv_data = sim.render_epicurve_csv();
    config.write_ouptut("epicurve.csv", &csv_data)?;

    println!("\nNUMBER OF INFECTIONS");
    println!("--------------------");
    plot_vbars(&sim.get_epicurve(2, true).unwrap(), 20);
    // plot_hbars(&sim.get_epicurve(2, true).unwrap(), 80);

    println!("Epistate: {:?}", sim.epistate(true));

    let n = sim.population().len();
    for i in (n - 10)..n {
        println!("{:?}", sim.population()[i]);
    }

    Ok(())
}
