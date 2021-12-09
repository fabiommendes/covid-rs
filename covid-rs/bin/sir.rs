use covid::{models::*, params::*, prelude::*, sim::*, utils::*};
use std::iter;

pub fn main() {
    /*
    // type T = models::SEICHAR<()>;
    // type T = models::SEAIR<()>;
    type T = SeirAgent<bool>;
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let mut pop: Vec<T> = new_population(10_000);
    let rng = &mut default_rng();

    pop.distrib_ages(AGE_DISTRIBUTION_BRAZIL, rng)
        // .vaccinate_random(true, 0.9, rng)
        // .distribute_vaccines(2_000, true, move |_| score(&mut rngb))
        // .distribute_vaccines(2_000, true, |ag| ag.age())
        .contaminate_at_random(10, rng)
        .force_infectious(false);

    let params: EpiParamsBindVaccine<AgeParam> = Default::default();
    // let mut params = AgeDependentSEIR::<AgeParam>::default();
    let mut sim: Simulation<_, _, _> = Simulation::new_simple(params.clone(), pop, 4.5, 0.095);

    sim.seed_from(rng);

    let cases = {
        let mut data = Vec::new();
        data.extend(iter::successors(Some(1.0), |x| Some(1.1 * x)).take(30));
        let last = *data.last().unwrap();
        data.resize_with(90, || last);
        data
    };

    sim.calibrate_sampler_from_cases(&cases);
    let prob = sim.sampler().prob_infection() * 1.25;
    sim.sampler_mut().set_prob_infection(prob);
    sim.run(120);

    // println!("{:?}", cases);
    println!("{}", sim.render_epicurve_csv(T::CSV_HEADER));

    plot_vbars(&sim.get_epicurve(2, true).unwrap(), 20);
    plot_hbars(&sim.get_epicurve(2, true).unwrap(), 80);

    // println!("{:#?}", sim.sample(20));

    // for a in 0..10 {
    //     let obj = sim.get_agent_mut(0).unwrap();
    //     obj.set_age(1 + a * 5);
    //     obj.vaccinate(&false);
    //     println!("{:#?}", obj);

    //     let pset = params.clone_to_object(obj);
    //     let p = FullSEIRParams::<Real>::from_universal_params(&pset);
    //     println!("{:#?}", p);
    //     // println!("{:#?}", pset);
    // }
    */
}
