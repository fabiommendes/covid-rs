use super::{
    EpidemicSimulationMsg, EventHandler, InfectionPairsTracker, InfectionsPerStepTracker, Msg,
    StepDurationTracker,
};
use crate::{
    sim::Id,
    utils::{Accumulator, Sampler},
};
use std::any::Any;

pub struct EventDispatcher<E>
where
    E: Msg,
{
    listeners: Vec<Vec<Box<dyn EventHandler<E>>>>,
}

impl<E> EventDispatcher<E>
where
    E: Msg,
{
    /// Create a new event dispatcher
    pub fn new() -> Self {
        let mut events = Vec::with_capacity(E::EVENT_TYPES_COUNT);
        for _ in 0..E::EVENT_TYPES_COUNT {
            events.push(Vec::new());
        }
        return EventDispatcher { listeners: events };
    }

    /// Trigger all handlers for the given event.
    pub fn trigger(&mut self, event: &E) {
        for ev in &mut self.listeners[event.id()] {
            ev.handle(event)
        }
    }

    /// Register event handler
    pub fn register(&mut self, handler: Box<dyn EventHandler<E>>) {
        let id = handler.handle_id();
        self.listeners[id].push(handler);
    }
}

impl EventDispatcher<EpidemicSimulationMsg> {
    /// Initialize the default EpiEvent listeners.
    pub fn new_with_default_listeners() -> Self {
        let mut new = Self::new();
        let sampler = Accumulator::empty();

        new.register(Box::new(InfectionPairsTracker::new()));
        new.register(Box::new(InfectionsPerStepTracker::new()));
        new.register(Box::new(StepDurationTracker::new_from_sampler(sampler)));
        return new;
    }

    /// Return a slice with all detected infection pairs
    pub fn infection_pairs(&self) -> Option<&[(Id, Id)]> {
        let id = EpidemicSimulationMsg::NEW_INFECTION_ID;
        for h in &self.listeners[id] {
            if let Some(h) = <dyn Any>::downcast_ref::<InfectionPairsTracker>(h) {
                return Some(h.infection_pairs());
            }
        }
        return None;
    }

    /// Return a slice with the infection for each step so far.  
    pub fn infection_counts(&self) -> Option<&[usize]> {
        let id = EpidemicSimulationMsg::END_STEP_ID;
        for h in &self.listeners[id] {
            if let Some(h) = <dyn Any>::downcast_ref::<InfectionsPerStepTracker>(h) {
                return Some(h.infection_counts());
            }
        }
        return None;
    }
}
