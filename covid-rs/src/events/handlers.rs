use super::{EpidemicSimulationMsg, Msg};
use crate::sim::Id;
use crate::{prelude::Real, utils::Sampler};
use dyn_clone::DynClone;
use getset::{CopyGetters, Getters, Setters};
use std::{
    fmt::Debug,
    thread::sleep,
    time::{Duration, Instant},
};

/// The EventHandler<E> trait describes an object that can process some
/// event of type E (usually an enum type) in a given context Ctx (usually a
/// simulation object).
///
/// Event handlers are useful to collect statistics, and generate reports.
pub trait EventHandler<E>: DynClone
where
    E: Msg,
{
    /// Handle event in the given context. Usually, the context is a reference
    /// to a simulation object. The event handler cannot affect the context.
    fn handle(&mut self, event: &E);

    /// In order to make dispatch more efficient, event handlers are registered
    /// to separate lanes depend on event type. This usually corresponds to a
    /// different Id for each case of a enum type.
    ///
    /// This method should return the handle id for the event handler.
    fn handle_id(&self) -> usize;
}

/** TRACK INFECTION PAIRS  ***************************************************/

/// Event listener that keeps track of all pairs of infections.
///
/// It listens to the NewInfection(a, b) message and stores the pair (a, b)
/// in an internal list.
#[derive(Debug, Clone, Default)]
pub struct InfectionPairsTracker {
    data: Vec<(Id, Id)>,
}

impl InfectionPairsTracker {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Count infections for the given id
    pub fn count_new_infections_by(&self, id: Id) -> usize {
        self.data.iter().filter(|(a, _)| id == *a).count()
    }

    /// Expose a slice with all infection pairs
    pub fn infection_pairs(&self) -> &[(Id, Id)] {
        return &self.data;
    }

    /// Push new event that agent a infects agent b.
    ///
    /// Agents are tracked by id.
    pub fn push(&mut self, a: Id, b: Id) {
        self.data.push((a, b));
    }
}

impl EventHandler<EpidemicSimulationMsg> for InfectionPairsTracker {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::NewInfection(a, b) = event {
            self.push(a, b)
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::NewInfection(0, 0).id();
    }
}

/** TRACK NEW INFECTION COUNTS ***********************************************/

/// Count the number of infections per step.
///
/// Listen to the EndStep(num_infections) event
#[derive(Debug, Clone, Default)]
pub struct InfectionsPerStepTracker {
    data: Vec<usize>,
}

impl InfectionsPerStepTracker {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Push new value.
    pub fn push(&mut self, n: usize) {
        self.data.push(n);
    }

    /// Expose a slice with all infection counts
    pub fn infection_counts(&self) -> &[usize] {
        return &self.data;
    }
}

impl EventHandler<EpidemicSimulationMsg> for InfectionsPerStepTracker {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::EndStep(_, n) = event {
            self.push(n)
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::END_STEP_ID;
    }
}

/** TRACK ITERATION TIMES ****************************************************/

/// A simple clock that count the duration of each iteration.
#[derive(Getters, CopyGetters, Debug, Clone)]
pub struct StepDurationTracker<S: Sampler + Clone> {
    #[getset(get_copy = "pub")]
    instant: Instant,

    #[getset(get = "pub")]
    sampler: S,
}

impl<S: Sampler + Clone> StepDurationTracker<S> {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::new_from_sampler(S::empty())
    }

    /// Create a new tracker from a sampler object.
    pub fn new_from_sampler(sampler: S) -> Self {
        StepDurationTracker {
            instant: Instant::now(),
            sampler: sampler,
        }
    }
}

impl<S: Default + Sampler + Clone> Default for StepDurationTracker<S> {
    fn default() -> Self {
        StepDurationTracker {
            instant: Instant::now(),
            sampler: S::default(),
        }
    }
}

impl<S: Sampler + Clone> EventHandler<EpidemicSimulationMsg> for StepDurationTracker<S> {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::EndStep(_, _) = event {
            let now = Instant::now();
            let delta = now.duration_since(self.instant);
            self.instant = now;
            self.sampler.observe(delta.as_secs_f64())
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::END_STEP_ID;
    }
}

/** THROTTLE SIMULATION ******************************************************/

/// A simple throttle that limit the frequency of steps to some specified
/// amount.
#[derive(Getters, CopyGetters, Setters, Debug, Clone)]
pub struct Throttle {
    #[getset(get_copy = "pub")]
    instant: Instant,

    #[getset(get = "pub", set = "pub")]
    duration: Duration,
}

impl Throttle {
    pub fn new(duration: Duration) -> Self {
        Throttle {
            instant: Instant::now(),
            duration,
        }
    }

    pub fn new_sec(dt: Real) -> Self {
        let sec = dt as u64;
        let nano = (1e9 * (dt - sec as Real)) as u32;
        Self::new(Duration::new(sec, nano))
    }
}

impl EventHandler<EpidemicSimulationMsg> for Throttle {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::EndStep(_, _) = event {
            let now = Instant::now();
            let elapsed = now.duration_since(self.instant);
            self.instant = now;
            sleep(self.duration - elapsed)
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::END_STEP_ID;
    }
}

