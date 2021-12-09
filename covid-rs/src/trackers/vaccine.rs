use std::marker::PhantomData;

use crate::{
    models::{SimpleAgentPopulationExt},
    sim::HasAge,
};

use super::TrackerMut;

pub enum VaccineSupply {
    Empty,
    Curve(Vec<usize>),
    Constant(usize),
}

pub struct VaccinationStrategy<M, V> {
    supply: VaccineSupply,
    vaccine: V,
    _phantom: PhantomData<M>,
}

impl<M, V> VaccinationStrategy<M, V> {
    fn apply_doses<P>(&mut self, population: &mut P)
    where
        V: Clone,
        P: SimpleAgentPopulationExt<M, V>,
    {
        let n = match &mut self.supply {
            VaccineSupply::Empty => 0,
            VaccineSupply::Constant(n) => *n,
            VaccineSupply::Curve(v) => {
                let n = v.pop().unwrap_or(0);
                if v.is_empty() {
                    self.supply = VaccineSupply::Constant(n)
                }
                n
            }
        };
        if n > 0 {
            population.distribute_vaccines(n, self.vaccine.clone(), |ag| ag.age());
        }
    }
}

impl<M, V, P> TrackerMut<P> for VaccinationStrategy<M, V>
where
    V: Clone,
    P: SimpleAgentPopulationExt<M, V>,
{
    fn track_mut(&mut self, pop: &mut P) {
        self.apply_doses(pop)
    }
}
