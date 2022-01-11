use getset::{CopyGetters, Getters};
use rand::Rng;

use crate::{
    epidemic::EpiModel,
    prelude::{Age, Real, Time},
    sim::{HasAge, HasEpiModel, Population, RandomUpdate},
};

/// A simple agent with an age, epidemic model and vaccine model.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Getters, CopyGetters)]
pub struct SimpleAgent<M, V> {
    age: Age,

    #[getset(get = "pub")]
    epimodel: M,

    #[getset(get_copy = "pub")]
    epimodel_t: Time,

    #[getset(get = "pub")]
    vaccine: Option<V>,

    #[getset(get_copy = "pub")]
    vaccine_t: Time,
}

impl<M, V: Clone> SimpleAgent<M, V> {
    /// Return true if agent is vaccinated
    pub fn is_vaccinated(&self) -> bool {
        self.vaccine.is_some()
    }
    
    /// Vaccinate agent with vaccine
    pub fn vaccinate(&mut self, vaccine: &V) {
        self.vaccine = Some(vaccine.clone());
        self.vaccine_t = 0;
    }

    /// Remove vaccine from agent
    pub fn unvaccinate(&mut self) {
        self.vaccine = None;
        self.vaccine_t = 0;
    }
}

impl<M, V> HasAge for SimpleAgent<M, V> {
    fn age(&self) -> Age {
        return self.age;
    }

    fn set_age(&mut self, value: Age) -> &mut Self {
        self.age = value;
        return self;
    }
}

impl<M: EpiModel, V> HasEpiModel for SimpleAgent<M, V> {
    type Model = M;

    fn epimodel(&self) -> &Self::Model {
        &self.epimodel
    }

    fn epimodel_mut(&mut self) -> &mut Self::Model {
        &mut self.epimodel
    }

    fn set_epimodel(&mut self, value: Self::Model) -> &mut Self {
        self.epimodel = value;
        return self;
    }
}

impl<M, V, W> RandomUpdate<W> for SimpleAgent<M, V>
where
    Self: HasEpiModel<Model = M>,
    M: RandomUpdate<W> + EpiModel,
{
    default fn random_update<R: Rng>(&mut self, world: &W, rng: &mut R) {
        self.epimodel_random_update(world, rng);
        self.epimodel_t += 1;
        self.vaccine_t += 1;
    }
}

///////////////////////////////////////////////////////////////////////////////
// Extend population with trait
///////////////////////////////////////////////////////////////////////////////
pub trait SimpleAgentPopulationExt<M, V>: Population<State = SimpleAgent<M, V>>
where
    V: Clone,
{
    /// Set vaccines of all agents to the given value
    fn set_vaccines(&mut self, value: V) -> &mut Self {
        self.each_agent_mut(|_, ag| ag.vaccinate(&value));
        return self;
    }

    /// Vaccinate all individuals that pass predicate.
    fn vaccinate_if(&mut self, value: V, f: impl FnMut(&mut Self::State) -> bool) -> &mut Self
    where
        Self::State: HasAge,
    {
        let mut pred = f;
        self.each_agent_mut(|_, ag| {
            if pred(ag) {
                ag.vaccinate(&value)
            }
        });
        return self;
    }

    /// Vaccinate at most n individuals with the given vaccine prioritizing with a score function.
    fn distribute_vaccines<F, C>(&mut self, n: usize, vaccine: V, score_fn: F) -> &mut Self
    where
        F: FnMut(&Self::State) -> C,
        C: Ord,
    {
        let mut score = score_fn;
        let mut m = n;
        let mut keys = Vec::with_capacity(self.count());
        self.each_agent(&mut |i, ag| keys.push((score(ag), i)));
        keys.sort_unstable();

        while m > 0 {
            match keys.pop() {
                Some((_, id)) => {
                    let ag = self.get_agent_mut(id).unwrap();
                    ag.vaccinate(&vaccine);
                    m -= 1;
                }
                _ => break,
            }
        }
        return self;
    }

    /// Vaccinate all individuals with the given vaccine and uniform
    /// probability
    fn vaccinate_random<R: Rng>(&mut self, value: V, prob: Real, rng: &mut R) -> &mut Self {
        self.vaccinate_if(value, |_| rng.gen_bool(prob))
    }

    /// Vaccinate all individuals older than the given age.
    fn vaccinate_elderly_random(
        &mut self,
        value: V,
        age: Age,
        prob: Real,
        rng: &mut impl Rng,
    ) -> &mut Self
    where
        Self::State: HasAge,
    {
        self.vaccinate_if(value, |ag| ag.age() >= age && rng.gen_bool(prob))
    }

    /// Vaccinate all individuals older than the given age.
    fn vaccinate_elderly(&mut self, value: V, age: Age) -> &mut Self
    where
        Self::State: HasAge,
    {
        self.vaccinate_if(value, |ag| ag.age() >= age)
    }

    /// Vaccinate all individuals that pass predicate.
    fn vaccinate_random_if(
        &mut self,
        value: V,
        prob: Real,
        rng: &mut impl Rng,
        f: impl FnMut(&mut Self::State) -> bool,
    ) -> &mut Self
    where
        Self::State: HasAge,
    {
        let mut pred = f;
        self.vaccinate_if(value, |ag| pred(ag) && rng.gen_bool(prob))
    }
}

impl<P, M, V> SimpleAgentPopulationExt<M, V> for P
where
    P: Population<State = SimpleAgent<M, V>>,
    V: Clone,
{
}
