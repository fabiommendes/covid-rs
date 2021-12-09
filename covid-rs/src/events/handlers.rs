use crate::sim::Id;

use super::{Msg, EpidemicSimulationMsg};


/// The EventHandler<E> trait describes an object that can process some
/// event of type E (usually an enum type) in a given context Ctx (usually a
/// simulation object).
///
/// Event handlers are useful to collect statistics, and generate reports.
pub trait EventHandler<E>
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

    /// Event handlers may produce reports for the user with information
    /// about the handler state. This is usually used in event handlers that collect
    /// statistics about the simulation.
    fn report(&self) -> String;
}

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

    /// Push new infection from agent a to agent b.
    ///
    /// Agents are tracked by id.
    pub fn add(&mut self, a: Id, b: Id) {
        self.data.push((a, b));
    }
}

impl EventHandler<EpidemicSimulationMsg> for InfectionPairsTracker {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::NewInfection(a, b) = event {
            self.data.push((a, b))
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::NewInfection(0, 0).id();
    }

    fn report(&self) -> String {
        return "".to_string();
    }
}

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

    /// Expose a slice with all infection counts
    pub fn infection_counts(&self) -> &[usize] {
        return &self.data;
    }
}

impl EventHandler<EpidemicSimulationMsg> for InfectionsPerStepTracker {
    fn handle(&mut self, event: &EpidemicSimulationMsg) {
        if let &EpidemicSimulationMsg::EndStep(_, n) = event {
            self.data.push(n)
        }
    }

    fn handle_id(&self) -> usize {
        return EpidemicSimulationMsg::END_STEP_ID;
    }

    fn report(&self) -> String {
        return "".to_string();
    }
}