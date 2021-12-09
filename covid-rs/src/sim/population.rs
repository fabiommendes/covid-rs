use super::{HasAge, Id, RandomUpdate, World};
use crate::prelude::{Age, AgeDistribution10};
use crate::{prelude::AgeCount10, utils::random_ages};
use rand::prelude::Rng;
use std::collections::HashSet;

/// The population trait describes a collection of agents.
pub trait Population {
    type State;

    /** Initialization *******************************************************/

    /// Creates population from a sequence of agents.
    fn from_states<'a, I>(states: I) -> Self
    where
        Self: 'a,
        Self::State: Clone,
        I: Iterator<Item = &'a Self::State>;

    /// Creates population from a sequence of states for each individual.
    fn from_slice(states: &[Self::State]) -> Self
    where
        Self: Sized,
        Self::State: Clone,
    {
        return Self::from_states(states.iter());
    }

    /// Creates population with n copies of the given state.
    fn from_state(n: usize, state: Self::State) -> Self
    where
        Self: Sized,
        Self::State: Clone,
    {
        return Population::from_states([state].iter().cycle().take(n));
    }

    /// Creates population with n copies of the default state.
    fn from_default(n: usize) -> Self
    where
        Self: Sized,
        Self::State: Default + Clone,
    {
        return Population::from_state(n, Self::State::default());
    }

    /** Conversions and information ******************************************/

    /// Enumerate all individual states in population.
    fn to_states(&self) -> Vec<Self::State>
    where
        Self::State: Clone,
    {
        let mut vec = vec![];
        self.each_agent(&mut |_, a: &Self::State| vec.push(a.clone()));
        return vec;
    }

    /// Count the population size.
    fn count(&self) -> usize;

    /** Extract/modify individual agents *************************************/

    /// Get an agent by id.
    fn get_agent(&self, id: Id) -> Option<&Self::State>;

    /// Get mutable reference to agent by id.
    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State>;

    /// Get a pair of agents by id.
    fn get_pair(&self, i: Id, j: Id) -> Option<(&Self::State, &Self::State)> {
        match (self.get_agent(i), self.get_agent(j)) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    /// Get a pair of agents by id.
    ///
    /// The option returns a reference to both agents. If the ids are the same, always
    /// return None.
    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)>;

    /// Get agents by ids. If you need to fetch multiple agents, it can be more
    /// convenient to use this.
    fn get_agents(&self, ids: impl IntoIterator<Item = Id>) -> Vec<(Id, &Self::State)> {
        let mut out = Vec::new();
        for id in ids.into_iter() {
            self.get_agent(id).map(|a| out.push((id, a)));
        }
        return out;
    }

    /// Set an agent state by id.
    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self
    where
        Self::State: Clone,
    {
        self.get_agent_mut(id).map(|st| *st = state.clone());
        return self;
    }

    /// Map function to agent.
    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.get_agent(id).map(f)
    }

    /// Map function to agent, mutating it.
    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.get_agent_mut(id).map(f)
    }

    /// Set multiple agent states by ids. If you need to update multiple
    /// agents, it can be more convenient to use this.
    fn set_agents(&mut self, updates: &[(Id, &Self::State)]) -> &mut Self
    where
        Self::State: Clone,
    {
        for &(id, state) in updates {
            self.set_agent(id, state);
        }
        return self;
    }

    /// Apply function to all agents of population. Function receives the Id
    /// and reference to State.
    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State);

    /// Apply function to all agents of population. Function receives the Id
    /// and mutable reference to State.
    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State));

    /// Select a random id using random number generator.
    fn random_id<R: Rng>(&self, rng: &mut R) -> Id {
        rng.gen_range(0..self.count())
    }

    /// Select a random agent using random number generator.
    fn random<R: Rng>(&self, rng: &mut R) -> (Id, &Self::State)
    where
        Self::State: Clone,
    {
        let i = self.random_id(rng);
        let ag = self.get_agent(i).unwrap();
        return (i, ag);
    }

    /// Select a random agent using random number generator.
    fn random_mut<R: Rng>(&mut self, rng: &mut R) -> (Id, &mut Self::State)
    where
        Self::State: Clone,
    {
        let i = self.random_id(rng);
        let ag = self.get_agent_mut(i).unwrap();
        return (i, ag);
    }

    /// Select many distinct random agents.
    ///
    /// Return a vector of (Id, State) pairs.
    fn randoms<R: Rng>(&self, count: usize, rng: &mut R) -> Vec<(Id, Self::State)>
    where
        Self::State: Clone,
    {
        let mut out = Vec::new();
        self.map_randoms(count, rng, |i, st| out.push((i, st.clone())));
        return out;
    }

    /// Select many distinct random agents.
    fn map_randoms<R, F>(&self, count: usize, rng: &mut R, f: F)
    where
        R: Rng,
        F: FnMut(usize, &Self::State),
        Self::State: Clone,
    {
        let mut ids = HashSet::new();
        let mut missing = count;
        let mut g = f;
        while missing > 0 {
            let (id, state) = self.random(rng);
            if !ids.contains(&id) {
                ids.insert(id);
                g(id, &state);
                missing -= 1;
            }
        }
    }

    /// A mutable version of map_randoms
    fn map_randoms_mut<R, F>(&mut self, count: usize, rng: &mut R, f: F)
    where
        R: Rng,
        F: FnMut(usize, &mut Self::State),
        Self::State: Clone,
    {
        let mut ids = HashSet::new();
        let mut missing = count;
        let mut g = f;
        while missing > 0 {
            let (id, state) = self.random_mut(rng);
            if !ids.contains(&id) {
                ids.insert(id);
                g(id, state);
                missing -= 1;
            }
        }
    }

    /** Change population state and run simulation ***************************/

    /// Apply random_update to each element of population
    fn random_update<W, R>(&mut self, world: &W, rng: &mut R)
    where
        R: Rng,
        W: World,
        <Self as Population>::State: RandomUpdate<W>,
    {
        self.each_agent_mut(&mut |_, st: &mut Self::State| st.random_update(world, rng));
    }

    /// Set ages of all agents acording to distribution.
    fn set_age_distribution<R>(&mut self, n: usize, distrib: AgeDistribution10, rng: &mut R)
    where
        Self::State: HasAge,
        R: Rng,
    {
        let ages = random_ages(n, rng, distrib);
        self.each_agent_mut(|i, st: &mut Self::State| {
            st.set_age(ages[i]);
        });
    }

    /// Set ages from age counts. This method fills the age bins sequentially and stops
    /// once all age counts were set.
    ///
    /// This usually means that the age counts should sum to the population size.
    fn set_age_counts<R>(&mut self, counts: AgeCount10, rng: &mut R)
    where
        Self::State: HasAge,
        R: Rng,
    {
        let mut i = 0;
        let mut n = counts.get(i).map(|x| *x).unwrap_or(0);

        self.each_agent_mut(|_, st| {
            if n <= 0 {
                i += 1;
                n = counts.get(i).map(|x| *x).unwrap_or(0);
            }
            if n > 0 {
                let start = i * 10;
                st.set_age(rng.gen_range(start..start + 10) as Age);
                n -= 1;
            }
        });
    }
}

/// Simple trait for implementations that store states in a Vec of states.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait OwnsStateSlice {
    type Elem;

    /// Create from an owned vector of agents
    fn from_agent_vec(agents: Vec<Self::Elem>) -> Self;

    /// Return an immutable slice of agents
    fn as_state_slice(&self) -> &[Self::Elem];

    /// Return a mutable slice of agents
    fn as_state_mut_slice(&mut self) -> &mut [Self::Elem];
}

/////////////////////////////////////////////////////////////////////////////
// Implementations
/////////////////////////////////////////////////////////////////////////////
impl<P> Population for P
where
    P: OwnsStateSlice + Sized,
    P::Elem: Sized,
{
    type State = P::Elem;

    fn from_states<'a, I>(states: I) -> Self
    where
        Self: 'a,
        I: Iterator<Item = &'a Self::State>,
        Self::State: Clone,
    {
        let mut agents = vec![];
        for st in states {
            agents.push(st.clone());
        }
        Self::from_agent_vec(agents)
    }

    fn count(&self) -> usize {
        self.as_state_slice().len()
    }

    fn get_agent(&self, id: Id) -> Option<&Self::State> {
        self.as_state_slice().get(id)
    }

    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State> {
        self.as_state_mut_slice().get_mut(id)
    }

    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State),
    {
        for (id, st) in self.as_state_slice().iter().enumerate() {
            f(id, st);
        }
    }

    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State)) {
        let mut g = f;
        for (id, st) in self.as_state_mut_slice().iter_mut().enumerate() {
            g(id, st);
        }
    }

    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)> {
        let slice = self.as_state_mut_slice();
        let n = slice.len();
        if i == j || i >= n || j >= n {
            return None;
        } else {
            // Safety: we can have two mutable borrows to elements of the slice
            // since the previous line guarantees that elements are not the same
            unsafe {
                let a = &mut *(slice.get_unchecked_mut(i) as *mut _);
                let b = &mut *(slice.get_unchecked_mut(j) as *mut _);
                return Some((a, b));
            }
        }
    }
}

impl<S> OwnsStateSlice for Vec<S>
where
    S: Clone,
{
    type Elem = S;

    fn from_agent_vec(states: Vec<S>) -> Self {
        return states;
    }

    fn as_state_slice(&self) -> &[S] {
        self.as_slice()
    }

    fn as_state_mut_slice(&mut self) -> &mut [S] {
        self.as_mut_slice()
    }
}
