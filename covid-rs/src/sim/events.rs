use std::any::Any;

use super::Id;

/** EVENTS *******************************************************************/

/// The Event describes a minimal structure that Event enums must have to
/// be used in an efficient event dispatcher struct  
pub trait Event {
    /// Must return a unique id for event type.
    fn id(&self) -> usize;

    /// Maximum id for all events
    const EVENT_TYPES_COUNT: usize;
}

/// Event
#[derive(Debug, Clone)]
pub enum EpiEvent {
    StartStep,
    EndStep(usize),
    NewInfection(Id, Id),
}

impl EpiEvent {
    const START_STEP_ID: usize = 0;
    const END_STEP_ID: usize = 1;
    const NEW_INFECTION_ID: usize = 2;
}

impl Event for EpiEvent {
    const EVENT_TYPES_COUNT: usize = 3;

    fn id(&self) -> usize {
        match self {
            Self::StartStep => Self::START_STEP_ID,
            Self::EndStep(_) => Self::END_STEP_ID,
            Self::NewInfection(_, _) => Self::NEW_INFECTION_ID,
        }
    }
}

/** EVENT HANDLERS ***********************************************************/

/// The EventHandler<E> trait describes an object that can process some
/// event of type E (usually an enum type) in a given context Ctx (usually a
/// simulation object).
///
/// Event handlers are useful to collect statistics, and generate reports.
pub trait EventHandler<E>
where
    E: Event,
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

impl EventHandler<EpiEvent> for InfectionPairsTracker {
    fn handle(&mut self, event: &EpiEvent) {
        if let &EpiEvent::NewInfection(a, b) = event {
            self.data.push((a, b))
        }
    }

    fn handle_id(&self) -> usize {
        return EpiEvent::NewInfection(0, 0).id();
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

impl EventHandler<EpiEvent> for InfectionsPerStepTracker {
    fn handle(&mut self, event: &EpiEvent) {
        if let &EpiEvent::EndStep(n) = event {
            self.data.push(n)
        }
    }

    fn handle_id(&self) -> usize {
        return EpiEvent::EndStep(0).id();
    }

    fn report(&self) -> String {
        return "".to_string();
    }
}

/** EVENT DISPATCHER *********************************************************/
pub struct EventDispatcher<E>
where
    E: Event,
{
    listeners: Vec<Vec<Box<dyn EventHandler<E>>>>,
}

impl<E> EventDispatcher<E>
where
    E: Event,
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

impl EventDispatcher<EpiEvent> {
    /// Initialize the default EpiEvent listeners.
    pub fn new_with_default_listeners() -> Self {
        let mut new = Self::new();
        new.register(Box::new(InfectionPairsTracker::new()));
        new.register(Box::new(InfectionsPerStepTracker::new()));
        return new;
    }

    /// Return a slice with all detected infection pairs
    pub fn infection_pairs(&self) -> Option<&[(Id, Id)]> {
        let id = EpiEvent::NewInfection(0, 0).id();
        for h in &self.listeners[id] {
            if let Some(h) = <dyn Any>::downcast_ref::<InfectionPairsTracker>(h) {
                return Some(h.infection_pairs());
            }
        }
        return None;
    }

    /// Return a slice with the infection for each step so far.  
    pub fn infection_counts(&self) -> Option<&[usize]> {
        let id = EpiEvent::EndStep(0).id();
        for h in &self.listeners[id] {
            if let Some(h) = <dyn Any>::downcast_ref::<InfectionsPerStepTracker>(h) {
                return Some(h.infection_counts());
            }
        }
        return None;
    }
}
